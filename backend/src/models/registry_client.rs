use super::ApiResponse;
use axum::{http::StatusCode, response::IntoResponse};
use reqwest::{Client, header};
use serde_json::Value;

pub struct RegistryClient {
    base_url: String,
    client: Client,
}

impl RegistryClient {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            client: Client::new(),
        }
    }

    // 1. Obtener todos los repositorios
    pub async fn get_catalog(&self, auth_header: &str) -> impl IntoResponse {
        let url = format!("{}/v2/_catalog", self.base_url);

        // Intentamos la operación completa
        self.fetch_registry_data(&url, auth_header)
            .await
            .map_or_else(
            |(status, msg)| ApiResponse::error(status, &msg),
            |data| ApiResponse::success("Repositorios obtenidos", Some(data)),
        )
    }

    // Función auxiliar interna para centralizar la lógica de reqwest
    async fn fetch_registry_data(
        &self,
        url: &str,
        auth: &str,
    ) -> Result<Value, (StatusCode, String)> {
        let resp = self
            .client
            .get(url)
            .header(header::AUTHORIZATION, auth)
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

    // 2. Obtener tags de un repositorio
    pub async fn get_tags(&self, repo: &str, auth_header: &str) -> impl IntoResponse {
        let url = format!("{}/v2/{}/tags/list", self.base_url, repo);
        self.fetch_registry_data(&url, auth_header)
        .await
        .map_or_else(
            |(status, msg)| ApiResponse::error(status, &msg),
            |data| ApiResponse::success(&format!("Tags de {} obtenidos", repo), Some(data)),
        )
    }

    // 3. Obtener el Digest (necesario para borrar)
    async fn get_manifest_digest(
        &self,
        repo: &str,
        tag: &str,
        auth: &str,
    ) -> Result<String, (StatusCode, String)> {
        let url = format!("{}/v2/{}/manifests/{}", self.base_url, repo, tag);

        let resp = self
            .client
            .head(&url)
            .header(header::AUTHORIZATION, auth)
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

    // 4. Eliminar un tag (usando su digest)
    pub async fn delete_tag(&self, repo: &str, tag: &str, auth_header: &str) -> impl IntoResponse {
        // 1. Primero necesitamos el Digest
        let digest = match self.get_manifest_digest(repo, tag, auth_header).await {
            Ok(d) => d,
            Err((status, msg)) => return ApiResponse::error(status, &msg).into_response(),
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
                ApiResponse::success("Tag eliminado correctamente", None)
                    .into_response()
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
}
