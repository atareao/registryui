use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Catalog {
    pub repositories: Vec<String>,
}
