// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config_cli;

use clap::Parser;
use config_cli::{CliArgs, save_config, show_current_config};

fn main() {
    // Parse CLI arguments
    let args = CliArgs::parse();

    // Handle --show-config flag
    if args.show_config {
        show_current_config();
        return;
    }

    // Handle --configure flag
    if args.configure {
        if let Some(ref token) = args.notion_token {
            if token.is_empty() {
                eprintln!("❌ Error: Notion token cannot be empty");
                std::process::exit(1);
            }

            match save_config(token, &args.notion_database) {
                Ok(_) => {
                    println!("\n✅ Configuration saved successfully!");
                    println!("You can now run the application normally.");
                    std::process::exit(0);
                }
                Err(e) => {
                    eprintln!("❌ Error: {}", e);
                    std::process::exit(1);
                }
            }
        } else {
            eprintln!("❌ Error: --configure requires --notion-token");
            eprintln!("\nUsage:");
            eprintln!("  {} --configure --notion-token YOUR_TOKEN",
                     std::env::current_exe()
                         .ok()
                         .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
                         .unwrap_or_else(|| "hog-c2".to_string()));
            std::process::exit(1);
        }
    }

    // Normal application startup
    app_lib::run();
}
