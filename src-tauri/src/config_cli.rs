use clap::Parser;
use std::fs;
use std::path::PathBuf;
use dirs;

#[derive(Parser, Debug)]
#[command(name = "hog-c2")]
#[command(version, about = "Remote monitoring and management tool", long_about = None)]
pub struct CliArgs {
    /// Notion API integration token
    #[arg(long, env = "NOTION_INTEGRATION_TOKEN")]
    pub notion_token: Option<String>,

    /// Notion database name to use
    #[arg(long, env = "NOTION_DATABASE_NAME", default_value = "🖥 Remote Systems")]
    pub notion_database: String,

    /// Configure and save settings without starting the GUI
    #[arg(long)]
    pub configure: bool,

    /// Show current configuration
    #[arg(long)]
    pub show_config: bool,
}

/// Get the config file path
pub fn get_config_path() -> PathBuf {
    let config_dir = dirs::config_dir()
        .or_else(|| dirs::home_dir().map(|h| h.join(".config")))
        .expect("Failed to get config directory");

    let app_config_dir = config_dir.join("hog-c2");
    fs::create_dir_all(&app_config_dir).ok();

    app_config_dir.join("config.env")
}

/// Save configuration to file
pub fn save_config(notion_token: &str, notion_database: &str) -> Result<(), String> {
    let config_path = get_config_path();
    let config_content = format!(
        "NOTION_INTEGRATION_TOKEN={}\nNOTION_DATABASE_NAME={}\n",
        notion_token, notion_database
    );

    fs::write(&config_path, config_content)
        .map_err(|e| format!("Failed to save config: {}", e))?;

    println!("✅ Configuration saved to: {}", config_path.display());
    Ok(())
}

/// Load configuration from file
pub fn load_saved_config() -> Option<(String, String)> {
    let config_path = get_config_path();

    if !config_path.exists() {
        return None;
    }

    let content = fs::read_to_string(&config_path).ok()?;

    let mut token = None;
    let mut database = None;

    for line in content.lines() {
        if line.starts_with("NOTION_INTEGRATION_TOKEN=") {
            token = Some(line.trim_start_matches("NOTION_INTEGRATION_TOKEN=").to_string());
        } else if line.starts_with("NOTION_DATABASE_NAME=") {
            database = Some(line.trim_start_matches("NOTION_DATABASE_NAME=").to_string());
        }
    }

    match (token, database) {
        (Some(t), Some(d)) => Some((t, d)),
        _ => None,
    }
}

/// Display current configuration
pub fn show_current_config() {
    let config_path = get_config_path();

    println!("📁 Config file location: {}", config_path.display());

    if let Some((token, database)) = load_saved_config() {
        println!("✓ Configuration found:");
        println!("  Notion Database: {}", database);
        println!("  Notion Token: {}...{}",
                 &token[..8.min(token.len())],
                 if token.len() > 16 { &token[token.len()-8..] } else { "" });
    } else {
        println!("❌ No saved configuration found");
        println!("\nTo configure, run:");
        println!("  {} --configure --notion-token YOUR_TOKEN --notion-database \"Database Name\"",
                 std::env::current_exe()
                     .ok()
                     .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
                     .unwrap_or_else(|| "hog-c2".to_string()));
    }
}
