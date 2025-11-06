use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub notion_api_secret: String,
    pub notion_database_name: String,
    pub app_name: String,
    pub app_version: String,
}

impl AppConfig {
    /// Load configuration from environment variables or defaults
    pub fn load() -> Result<Self, String> {
        // Try to load .env file in development (ignore if not found)
        #[cfg(debug_assertions)]
        {
            let _ = dotenvy::dotenv();
        }

        // Try to load from CLI-saved config first
        let saved_config = crate::config_cli::load_saved_config();

        // Try to load from environment first, then fall back to saved config, then compile-time defaults
        let notion_api_secret = env::var("NOTION_API_SECRET")
            .or_else(|_| {
                saved_config.as_ref()
                    .map(|(token, _)| token.clone())
                    .ok_or(())
            })
            .unwrap_or_else(|_| {
                // Fallback for development - should be overridden in production
                option_env!("NOTION_API_SECRET")
                    .unwrap_or("")
                    .to_string()
            });

        if notion_api_secret.is_empty() {
            return Err(
                "NOTION_API_SECRET not configured. Please run with --configure flag or set the environment variable".to_string()
            );
        }

        let notion_database_name = env::var("NOTION_DATABASE_NAME")
            .or_else(|_| {
                saved_config.as_ref()
                    .map(|(_, db)| db.clone())
                    .ok_or(())
            })
            .unwrap_or_else(|_| "All Clients".to_string());

        let app_name = env::var("APP_NAME")
            .unwrap_or_else(|_| "FocusForge".to_string());

        let app_version = env::var("APP_VERSION")
            .unwrap_or_else(|_| env!("CARGO_PKG_VERSION").to_string());

        // Log token info in debug mode (first 10 chars only for security)
        #[cfg(debug_assertions)]
        {
            if notion_api_secret.len() > 10 {
                log::info!("Loaded Notion token: {}...", &notion_api_secret[..10]);
            }
        }

        Ok(AppConfig {
            notion_api_secret,
            notion_database_name,
            app_name,
            app_version,
        })
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.notion_api_secret.is_empty() {
            return Err("Notion API secret is required".to_string());
        }

        if !self.notion_api_secret.starts_with("secret_") {
            return Err("Invalid Notion API secret format".to_string());
        }

        if self.notion_database_name.is_empty() {
            return Err("Notion database name is required".to_string());
        }

        Ok(())
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            notion_api_secret: String::new(),
            notion_database_name: "All Clients".to_string(),
            app_name: "FocusForge".to_string(),
            app_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_valid_config() {
        let config = AppConfig {
            notion_api_secret: "secret_validtoken123".to_string(),
            notion_database_name: "Test DB".to_string(),
            app_name: "Test App".to_string(),
            app_version: "1.0.0".to_string(),
        };

        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validate_invalid_secret() {
        let config = AppConfig {
            notion_api_secret: "invalid_token".to_string(),
            ..Default::default()
        };

        assert!(config.validate().is_err());
    }
}
