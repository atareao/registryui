use serde::{Deserialize, Serialize};

// =================================================================
// 1. ESTRUCTURAS DE DATOS (STRUCTS)
// =================================================================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub username: String,
    pub hashed_password: String,
}
