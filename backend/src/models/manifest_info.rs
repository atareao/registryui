use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct ManifestInfo {
    pub name: String,
    pub tag: String,
    pub digest: String,
}
