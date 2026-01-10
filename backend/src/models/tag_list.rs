use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct TagList {
    pub name: String,
    pub tags: Option<Vec<String>>,
}
