use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppLimit {
    pub app_name: String,
    pub max_duration_minutes: i64,
    pub notification_threshold_minutes: i64,
    pub enabled: bool,
}
