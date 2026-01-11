use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct RepositoryInfo {
    pub name: String,
    pub last_push: Option<String>,
    pub tag_count: usize,
}
