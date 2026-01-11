use super::ApiResponse;
use super::catalog::Catalog;
use super::manifest_v2::ManifestV2;
use super::repository_info::RepositoryInfo;
use super::tag_detail::TagDetail;
use super::tag_list::TagList;
use axum::{http::StatusCode, response::IntoResponse};
use dashmap::DashMap;
use reqwest::header::{ACCEPT, AUTHORIZATION, HeaderMap, HeaderValue};
use reqwest::{Client, header};
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::sync::Arc;
use tracing::debug;

#[derive(Clone)]
pub struct RegistryClient {
    base_url: String,
    basic_auth: String,
    client: Client,
    cache: Arc<DashMap<String, RepositoryInfo>>,
}

impl RegistryClient {
    pub fn new(base_url: String, encoded: String) -> Self {
        Self {
            base_url,
            basic_auth: format!("Basic {}", encoded),
            client: Client::new(),
            cache: Arc::new(DashMap::new()),
        }
    }

    async fn fetch_catalog_names(&self) -> Result<Catalog, (StatusCode, String)> {
        let url = format!("{}/v2/_catalog", self.base_url);
        self.fetch_from_registry(&url, None).await
    }

    async fn fetch_tags(&self, repo: &str) -> Result<TagList, (StatusCode, String)> {
        let url = format!("{}/v2/{}/tags/list", self.base_url, repo);
        self.fetch_from_registry(&url, None).await
    }

    pub async fn fetch_creation_date(
        &self,
        repo: &str,
        tag: &str,
    ) -> Result<String, (StatusCode, String)> {
        // 1. Obtener el manifiesto para el tag dado
        let manifest: ManifestV2 = self
            .fetch_from_registry(
                &format!("{}/v2/{}/manifests/{}", self.base_url, repo, tag),
                Some({
                    let mut headers = HeaderMap::new();
                    headers.insert(
                        ACCEPT,
                        HeaderValue::from_static(
                            "application/vnd.docker.distribution.manifest.v2+json",
                        ),
                    );
                    headers
                }),
            )
            .await?;

        // 2. Obtener el blob de configuración usando el digest del manifiesto
        let config_blob: Value = self
            .fetch_config_blob(repo, &manifest.config.digest)
            .await?;

        // 3. Extraer la fecha de creación del JSON del blob de configuración
        if let Some(created) = config_blob.get("created")
            && let Some(created_str) = created.as_str()
        {
            return Ok(created_str.to_string());
        }

        Err((
            StatusCode::BAD_REQUEST,
            "Fecha de creación no encontrada en el blob de configuración".to_string(),
        ))
    }

    pub async fn fetch_manifest_info(&self, repo: &str, tag: &str) -> impl IntoResponse {
        let url = format!("{}/v2/{}/manifests/{}", self.base_url, repo, tag);

        // Preparamos la cabecera específica para Manifiesto V2
        let mut headers = HeaderMap::new();
        headers.insert(
            ACCEPT,
            HeaderValue::from_static("application/vnd.docker.distribution.manifest.v2+json"),
        );

        // Llamada genérica pasando las cabeceras
        match self
            .fetch_from_registry::<ManifestV2>(&url, Some(headers))
            .await
        {
            Ok(manifest) => ApiResponse::success(
                "Manifiesto obtenido",
                Some(serde_json::to_value(manifest).unwrap()),
            ),
            Err((status, msg)) => ApiResponse::error(status, &msg),
        }
        .into_response()
    }

    async fn fetch_config_blob(
        &self,
        repo: &str,
        digest: &str,
    ) -> Result<Value, (StatusCode, String)> {
        let url = format!("{}/v2/{}/blobs/{}", self.base_url, repo, digest);
        self.fetch_from_registry::<Value>(&url, None).await
    }

    async fn fetch_from_registry<T: DeserializeOwned>(
        &self,
        url: &str,
        extra_headers: Option<HeaderMap>,
    ) -> Result<T, (StatusCode, String)> {
        let mut request = self
            .client
            .get(url)
            .header(AUTHORIZATION, self.basic_auth.clone());
        if let Some(headers) = extra_headers {
            request = request.headers(headers);
        }
        let resp = request.send().await.map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Error de red: {}", e),
            )
        })?;

        // Si el Registry devuelve 401 o cualquier error, lo mapeamos
        if !resp.status().is_success() {
            return Err((
                resp.status(),
                format!("Error al obtener el catálogo: {}", resp.status()),
            ));
        }

        // Parseamos el JSON a nuestra estructura Catalog definida previamente
        let body_text = resp.text().await.map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Error leyendo cuerpo: {}", e),
            )
        })?;

        // Intentar deserializar manualmente para capturar el error exacto de Serde
        serde_json::from_str::<T>(&body_text).map_err(|e| {
            tracing::error!("Cuerpo recibido que falló: {}", body_text);
            (
                StatusCode::BAD_REQUEST,
                format!("Error de parseo en {}: {}. Cuerpo: {}", url, e, body_text),
            )
        })
    }

    pub async fn get_catalog(self) -> impl IntoResponse {
        let catalog = match self.fetch_catalog_names().await {
            Ok(c) => c,
            Err((s, m)) => return ApiResponse::error(s, &m).into_response(),
        };

        // Usamos Arc para poder compartir el cliente en los hilos asíncronos
        let client_arc = Arc::new(self);

        let futures = catalog.repositories.into_iter().map(|repo_name| {
            let client = client_arc.clone();

            async move {
                // 1. Acceso correcto a la caché
                // DashMap devuelve un Ref; usamos .value() para llegar al RepositoryInfo
                if let Some(cached_ref) = client.cache.get(&repo_name) {
                    return cached_ref.value().clone();
                }

                // 2. Trabajo pesado
                let tags = client.fetch_tags(&repo_name).await.ok();
                let count = tags.as_ref().map(|t| t.tags.len()).unwrap_or(0);
                let mut last_date = None;

                if let Some(t_list) = tags
                    && let Some(last_tag) = t_list.tags.last()
                {
                    // Usamos la auth almacenada en self (client)
                    last_date = client.fetch_creation_date(&repo_name, last_tag).await.ok();
                }

                let info = RepositoryInfo {
                    name: repo_name.clone(),
                    last_push: last_date,
                    tag_count: count,
                };

                // 3. Insertar en caché
                client.cache.insert(repo_name, info.clone());
                info
            }
        });

        let enriched_data = futures::future::join_all(futures).await;

        ApiResponse::success("Catálogo obtenido", Some(enriched_data)).into_response()
    }

    // Función auxiliar interna para centralizar la lógica de reqwest
    async fn fetch_registry_data(&self, url: &str) -> Result<Value, (StatusCode, String)> {
        let resp = self
            .client
            .get(url)
            .header(header::AUTHORIZATION, &self.basic_auth)
            .send()
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Error de red: {}", e),
                )
            })?;

        if !resp.status().is_success() {
            return Err((
                resp.status(),
                format!("Error del Registry: {}", resp.status()),
            ));
        }

        resp.json::<Value>().await.map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                format!("Error al parsear JSON: {}", e),
            )
        })
    }

    // 3. Obtener el Digest (necesario para borrar)
    async fn get_manifest_digest(
        &self,
        repo: &str,
        tag: &str,
    ) -> Result<String, (StatusCode, String)> {
        let url = format!("{}/v2/{}/manifests/{}", self.base_url, repo, tag);

        let resp = self
            .client
            .head(&url)
            .header(header::AUTHORIZATION, &self.basic_auth)
            // IMPORTANTE: Sin esta cabecera, el Registry puede devolverte el digest v1 en lugar del v2
            .header(
                header::ACCEPT,
                "application/vnd.docker.distribution.manifest.v2+json",
            )
            .send()
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Error de red: {}", e),
                )
            })?;

        if !resp.status().is_success() {
            return Err((
                resp.status(),
                format!("No se pudo obtener el manifiesto de {}", tag),
            ));
        }

        // El Digest viene en esta cabecera específica
        let digest = resp
            .headers()
            .get("Docker-Content-Digest")
            .and_then(|h| h.to_str().ok())
            .ok_or((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Cabecera Docker-Content-Digest ausente".to_string(),
            ))?;

        Ok(digest.to_string())
    }

    pub async fn delete_tag(&self, repo: &str, tag: &str, auth_header: &str) -> impl IntoResponse {
        // 1. Primero necesitamos el Digest
        let digest = match self.get_manifest_digest(repo, tag).await {
            Ok(d) => d,
            Err((status, msg)) => return ApiResponse::<Value>::error(status, &msg).into_response(),
        };

        // 2. Ahora ejecutamos el borrado real usando el digest
        let url = format!("{}/v2/{}/manifests/{}", self.base_url, repo, digest);

        let resp_result = self
            .client
            .delete(&url)
            .header(header::AUTHORIZATION, auth_header)
            .send()
            .await;

        match resp_result {
            Ok(resp) if resp.status().is_success() => {
                ApiResponse::<Value>::success("Tag eliminado correctamente", None).into_response()
            }
            Ok(resp) => ApiResponse::error(
                resp.status(),
                "El Registry no permitió el borrado (¿está habilitado?)",
            )
            .into_response(),
            Err(e) => ApiResponse::error(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
                .into_response(),
        }
    }

    pub async fn get_tags(&self, repo: &str) -> impl IntoResponse {
        // 1. Obtener la lista básica de tags
        let tag_list = match self.fetch_tags(repo).await {
            Ok(t) => t,
            Err((s, m)) => return ApiResponse::<Value>::error(s, &m).into_response(),
        };

        let repo_name = repo.to_string();
        let futures = tag_list.tags.into_iter().map(|tag_name| {
            // Clonamos el cliente directamente (ya tiene Arc interno para la caché)
            let client = self.clone();
            let repo = repo_name.clone();

            async move {
                let url = format!("{}/v2/{}/manifests/{}", client.base_url, repo, tag_name);

                // Construimos las cabeceras de forma más segura
                let mut headers = HeaderMap::new();
                headers.insert(
                    header::ACCEPT,
                    HeaderValue::from_static(
                        "application/vnd.docker.distribution.manifest.v2+json",
                    ),
                );

                // Intentamos obtener el manifiesto
                match client
                    .fetch_from_registry::<ManifestV2>(&url, Some(headers))
                    .await
                {
                    Ok(m) => {
                        // Si el manifiesto funciona, intentamos el config blob para la fecha
                        let size: u64 =
                            m.layers.iter().map(|l| l.size).sum::<u64>() + m.config.size;

                        // Aquí llamamos a fetch_config_blob que ya tienes en tu registry_client.rs
                        let config_blob = client.fetch_config_blob(&repo, &m.config.digest).await;
                        debug!("Config blob para {}: {:?}", tag_name, config_blob);

                        match config_blob {
                            Ok(c) => TagDetail {
                                name: tag_name,
                                digest: m.config.digest,
                                size_bytes: size,
                                created_at: c
                                    .get("created")
                                    .and_then(|v| v.as_str())
                                    .map(|s| s.to_string()),
                                architecture: c
                                    .get("architecture")
                                    .and_then(|v| v.as_str())
                                    .map(|s| s.to_string()),
                                os: c.get("os").and_then(|v| v.as_str()).map(|s| s.to_string()),
                            },
                            Err(e) => {
                                debug!("Error obteniendo config blob para {}: {}", tag_name, e.1);
                                TagDetail::basic(tag_name, m.config.digest, size)
                            }
                        }
                    }
                    Err(e) => {
                        // ESTO ES LO QUE ESTÁ PASANDO: El fetch del manifiesto falla
                        tracing::error!("Error fetch_manifest para {}: {}", tag_name, e.1);
                        TagDetail::empty(tag_name)
                    }
                }
            }
        });

        let enriched_tags = futures::future::join_all(futures).await;

        ApiResponse::success(
            &format!("Tags de {} obtenidos", repo_name),
            Some(enriched_tags),
        )
        .into_response()
    }

    // Implementación de apoyo para limpiar el código anterior
}
