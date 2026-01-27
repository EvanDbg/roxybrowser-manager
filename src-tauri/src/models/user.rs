use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub email: String,
    #[serde(rename = "displayName")]
    pub display_name: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(rename = "lastUsed")]
    pub last_used: String,
    #[serde(default)]
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub version: u32,
    #[serde(rename = "currentUser")]
    pub current_user: Option<String>,
    pub users: Vec<UserProfile>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            version: 1,
            current_user: None,
            users: Vec::new(),
        }
    }
}
