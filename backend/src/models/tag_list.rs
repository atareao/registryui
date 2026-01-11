use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct TagList {
    pub tags: Vec<String>,
}
