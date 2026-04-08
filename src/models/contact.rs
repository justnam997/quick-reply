use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct Contact {
    pub id: String,
    pub name: String,
    pub platform: String,
    pub profile_path: Option<String>,
}
