use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagDetail {
    pub name: String,
    pub digest: String,
    pub size_bytes: u64,
    pub created_at: Option<String>,
    pub architecture: Option<String>,
    pub os: Option<String>,
}

impl TagDetail {
    pub fn empty(name: String) -> Self {
        Self { name, digest: "n/a".into(), size_bytes: 0, created_at: None, architecture: None, os: None }
    }
    pub fn basic(name: String, digest: String, size_bytes: u64) -> Self {
        Self { name, digest, size_bytes, created_at: None, architecture: None, os: None }
    }
}
