// Module declarations
mod config;
pub mod config_cli;
mod models;
mod services;
mod platform;

use std::sync::Arc;
use std::time::Duration;
use tauri::{Manager, State, AppHandle, Wry};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{TrayIconBuilder, TrayIconEvent};
use tokio::time::interval;
use tokio::sync::Mutex as TokioMutex;

use config::AppConfig;
use models::{AppLimit, AppUsage, SystemInfo};
use services::{NotionService, TrackingService};

/// Application state shared across all commands
struct AppState {
    tracker: Arc<TrackingService>,
    config: Arc<AppConfig>,
    notion_page_id: Arc<TokioMutex<Option<String>>>,
    #[allow(dead_code)]
    terminal_cwd: Arc<TokioMutex<String>>, // Current working directory for terminal (used in background task)
    #[allow(dead_code)]
    hostname: String, // System hostname for page validation (used in background task)
    #[allow(dead_code)]
    page_validated: Arc<TokioMutex<bool>>, // Track if page structure has been validated (used in background task)
    #[allow(dead_code)]
    usage_database_id: Arc<TokioMutex<Option<String>>>, // Track usage statistics database ID (used in background task)
}

/// Get application statistics
#[tauri::command]
fn get_app_stats(state: State<AppState>) -> Vec<AppUsage> {
    state.tracker.get_stats()
}

/// Get usage for a specific application
#[tauri::command]
fn get_app_usage(state: State<AppState>, app_name: String) -> i64 {
    state.tracker.get_daily_total(&app_name)
}

/// Reset daily statistics
#[tauri::command]
fn reset_stats(state: State<AppState>) {
    state.tracker.reset_daily_stats();
}

/// Update application limits for blocking/notifications
#[tauri::command]
fn update_app_limits(state: State<AppState>, limits: Vec<AppLimit>) {
    state.tracker.update_limits(limits);
}

/// Get current application limits
#[tauri::command]
fn get_app_limits(state: State<AppState>) -> Vec<AppLimit> {
    state.tracker.get_limits()
}

/// Manually trigger a Notion sync
#[tauri::command]
async fn sync_notion_now(state: State<'_, AppState>) -> Result<String, String> {
    let page_id_opt = {
        let guard = state.notion_page_id.lock().await;
        guard.clone()
    };

    if let Some(page_id) = page_id_opt {
        log::info!("Manual Notion sync triggered");

        let sys_info = SystemInfo::collect();
        let app_stats = state.tracker.get_stats();
        let notion_service = NotionService::new(&state.config);

        // Get database ID from state
        let db_id = {
            let guard = state.usage_database_id.lock().await;
            guard.clone()
        };

        // Update page properties
        notion_service.update_client_page(&page_id, &sys_info).await?;

        // Update page content with app usage
        let new_db_id = notion_service.update_page_content(&page_id, &app_stats, db_id).await?;

        // Store database ID if it was just created
        if new_db_id.is_some() {
            let mut guard = state.usage_database_id.lock().await;
            *guard = new_db_id;
        }

        Ok(format!("Successfully synced with Notion ({} apps tracked)", app_stats.len()))
    } else {
        Err("Not registered with Notion yet. Please register first.".to_string())
    }
}

/// Check if autostart is enabled
#[tauri::command]
async fn is_autostart_enabled(app: AppHandle) -> Result<bool, String> {
    use tauri_plugin_autostart::ManagerExt;
    app.autolaunch()
        .is_enabled()
        .map_err(|e| e.to_string())
}

/// Enable or disable autostart
#[tauri::command]
async fn set_autostart_enabled(app: AppHandle, enable: bool) -> Result<(), String> {
    use tauri_plugin_autostart::ManagerExt;
    let autostart_manager = app.autolaunch();

    if enable {
        autostart_manager.enable().map_err(|e| e.to_string())?;
        log::info!("Autostart enabled");
    } else {
        autostart_manager.disable().map_err(|e| e.to_string())?;
        log::info!("Autostart disabled");
    }

    Ok(())
}

/// Show the main window
#[tauri::command]
fn show_window(app: AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

/// Hide the main window
#[tauri::command]
fn hide_window(app: AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
    }
}

/// Register this client with Notion
#[tauri::command]
async fn register_client_with_notion(state: State<'_, AppState>) -> Result<String, String> {
    let notion_service = NotionService::new(&state.config);

    // Collect system information
    let sys_info = SystemInfo::collect();

    match notion_service.initialize_client(sys_info).await {
        Ok((page_id, db_id)) => {
            // Store the page ID for future updates
            let mut stored_page_id = state.notion_page_id.lock().await;
            *stored_page_id = Some(page_id.clone());
            log::info!("Stored page ID for periodic updates: {}", page_id);

            // Store the database ID if returned
            if let Some(database_id) = db_id {
                let mut stored_db_id = state.usage_database_id.lock().await;
                *stored_db_id = Some(database_id.clone());
                log::info!("Stored database ID for future updates: {}", database_id);
            }

            Ok(format!("Successfully registered client. Page ID: {}", page_id))
        }
        Err(e) => {
            log::error!("Failed to register with Notion: {}", e);
            Err(format!("Failed to register client: {}", e))
        }
    }
}

/// Background task to track active windows
async fn start_tracking(tracker: Arc<TrackingService>) {
    let mut interval = interval(Duration::from_secs(1));

    loop {
        interval.tick().await;

        if let Some((app_name, window_title)) = TrackingService::get_active_window_info() {
            tracker.update_active_window(app_name, window_title);
        }
    }
}

/// Background task to periodically update Notion page
async fn start_notion_sync(config: Arc<AppConfig>, page_id: Arc<TokioMutex<Option<String>>>, tracker: Arc<TrackingService>, terminal_cwd: Arc<TokioMutex<String>>, hostname: String, page_validated: Arc<TokioMutex<bool>>, usage_database_id: Arc<TokioMutex<Option<String>>>) {
    // Wait 5 seconds before first update (let initial registration complete)
    tokio::time::sleep(Duration::from_secs(5)).await;

    let mut interval = interval(Duration::from_secs(5)); // Update every 5 seconds

    loop {
        interval.tick().await;

        // Check if we have a page ID
        let page_id_opt = {
            let guard = page_id.lock().await;
            guard.clone()
        };

        if let Some(pid) = page_id_opt {
            log::info!("Updating Notion page with current system info and app usage...");

            // Collect fresh system information
            let sys_info = SystemInfo::collect();

            // Get app usage statistics
            let app_stats = tracker.get_stats();

            // Update the page
            let notion_service = NotionService::new(&config);

            // Validate and fix page structure (only once per session to avoid duplicates)
            let should_validate = {
                let validated = page_validated.lock().await;
                !*validated
            };

            if should_validate {
                match notion_service.validate_and_fix_page_structure(&pid, &hostname).await {
                    Ok(db_id) => {
                        log::info!("Page structure validated and fixed");
                        let mut validated = page_validated.lock().await;
                        *validated = true;

                        // Store database ID if page was recreated
                        if let Some(database_id) = db_id {
                            let mut stored_db_id = usage_database_id.lock().await;
                            *stored_db_id = Some(database_id.clone());
                            log::info!("Stored database ID from validation: {}", database_id);
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to validate page structure: {}", e);
                    }
                }
            }

            // Update page properties (system info)
            match notion_service.update_client_page(&pid, &sys_info).await {
                Ok(_) => {
                    log::info!("Successfully updated Notion page properties");
                }
                Err(e) => {
                    log::error!("Failed to update Notion page properties: {}", e);
                }
            }

            // Small delay after property update
            tokio::time::sleep(Duration::from_millis(200)).await;

            // Get or create database ID
            let db_id = {
                let guard = usage_database_id.lock().await;
                guard.clone()
            };

            // Update page content (app usage stats)
            match notion_service.update_page_content(&pid, &app_stats, db_id).await {
                Ok(new_db_id) => {
                    log::info!("Successfully updated Notion page content with {} apps", app_stats.len());
                    // Store database ID if it was just created
                    if new_db_id.is_some() {
                        let mut guard = usage_database_id.lock().await;
                        *guard = new_db_id;
                    }
                }
                Err(e) => {
                    log::error!("Failed to update Notion page content: {}", e);

                    // If update fails, delete all blocks and recreate from scratch
                    log::warn!("Update failed, attempting full page recreation...");

                    match notion_service.delete_all_page_blocks(&pid).await {
                        Ok(_) => {
                            log::info!("Deleted all blocks successfully");

                            // Recreate page structure
                            match notion_service.initialize_page_structure(&pid, &hostname).await {
                                Ok(db_id) => {
                                    log::info!("Page structure recreated successfully");

                                    // Mark for revalidation
                                    let mut validated = page_validated.lock().await;
                                    *validated = false;

                                    // Store the new database ID
                                    let mut db_guard = usage_database_id.lock().await;
                                    *db_guard = db_id;
                                }
                                Err(recreate_err) => {
                                    log::error!("Failed to recreate page structure: {}", recreate_err);
                                }
                            }
                        }
                        Err(del_err) => {
                            log::error!("Failed to delete blocks: {}", del_err);
                        }
                    }
                }
            }

            // Delay before next operation
            tokio::time::sleep(Duration::from_millis(300)).await;

            // Check if we should take a screenshot for debugging
            match notion_service.should_take_screenshot(&pid).await {
                Ok(should_screenshot) => {
                    if should_screenshot {
                        log::info!("Screenshot status is True, capturing screenshot...");

                        // Capture screenshot
                        match NotionService::capture_screenshot() {
                            Ok(screenshot_bytes) => {
                                log::info!("Screenshot captured, {} bytes", screenshot_bytes.len());

                                // Upload to Notion
                                let filename = format!("screenshot_{}.png", chrono::Local::now().format("%Y%m%d_%H%M%S"));
                                match notion_service.upload_file_to_notion(&screenshot_bytes, &filename).await {
                                    Ok(file_upload_id) => {
                                        log::info!("Screenshot uploaded with ID: {}", file_upload_id);

                                        // Append to page (this appends below existing screenshots)
                                        match notion_service.append_screenshot_to_page(&pid, &file_upload_id).await {
                                            Ok(_) => {
                                                log::info!("Screenshot appended to debugging section");

                                                // Set Screenshot property back to False after successful capture
                                                match notion_service.set_screenshot_property(&pid, false).await {
                                                    Ok(_) => {
                                                        log::info!("Screenshot property set back to False");
                                                    }
                                                    Err(e) => {
                                                        log::error!("Failed to set screenshot property to False: {}", e);
                                                    }
                                                }
                                            }
                                            Err(e) => {
                                                log::error!("Failed to append screenshot to page: {}", e);
                                            }
                                        }

                                        // Update page icon to computer emoji
                                        match notion_service.update_page_icon(&pid).await {
                                            Ok(_) => {
                                                log::info!("Page icon updated to computer emoji");
                                            }
                                            Err(e) => {
                                                log::error!("Failed to update page icon: {}", e);
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        log::error!("Failed to upload screenshot: {}", e);
                                    }
                                }
                            }
                            Err(e) => {
                                log::error!("Failed to capture screenshot: {}", e);
                            }
                        }
                    }
                }
                Err(e) => {
                    log::error!("Failed to check screenshot status: {}", e);
                }
            }

            // Update live terminal
            let current_cwd = {
                let cwd = terminal_cwd.lock().await;
                cwd.clone()
            };

            match notion_service.update_terminal(&pid, &current_cwd).await {
                Ok(new_cwd) => {
                    if new_cwd != current_cwd {
                        log::info!("Terminal working directory changed: {} -> {}", current_cwd, new_cwd);
                        let mut cwd = terminal_cwd.lock().await;
                        *cwd = new_cwd;
                    }
                }
                Err(e) => {
                    log::error!("Failed to update terminal: {}", e);

                    // If terminal update fails, likely structure is broken
                    // Delete and recreate page
                    log::warn!("Terminal update failed, recreating page structure...");

                    match notion_service.delete_all_page_blocks(&pid).await {
                        Ok(_) => {
                            match notion_service.initialize_page_structure(&pid, &hostname).await {
                                Ok(db_id) => {
                                    log::info!("Page structure recreated after terminal failure");
                                    let mut validated = page_validated.lock().await;
                                    *validated = false;
                                    // Store the new database ID
                                    let mut db_guard = usage_database_id.lock().await;
                                    *db_guard = db_id;
                                }
                                Err(recreate_err) => {
                                    log::error!("Failed to recreate page: {}", recreate_err);
                                }
                            }
                        }
                        Err(del_err) => {
                            log::error!("Failed to delete blocks: {}", del_err);
                        }
                    }
                }
            }
        } else {
            log::debug!("No page ID stored yet, skipping Notion update");
        }
    }
}

/// Build and configure the system tray
fn setup_system_tray(app: &AppHandle<Wry>) -> Result<(), Box<dyn std::error::Error>> {
    let show_item = MenuItem::with_id(app, "show", "Show Window", true, None::<&str>)?;
    let hide_item = MenuItem::with_id(app, "hide", "Hide Window", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&show_item, &hide_item, &quit_item])?;

    let _tray = TrayIconBuilder::with_id("main-tray")
        .tooltip("FocusForge - Forging your focus in the background")
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            "hide" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.hide();
                }
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click { button, .. } = event {
                if button == tauri::tray::MouseButton::Left {
                    let app = tray.app_handle();
                    if let Some(window) = app.get_webview_window("main") {
                        if window.is_visible().unwrap_or(false) {
                            let _ = window.hide();
                        } else {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                }
            }
        })
        .build(app)?;

    Ok(())
}

/// Main entry point for the Tauri application
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Load configuration
    let config = match AppConfig::load() {
        Ok(cfg) => {
            if let Err(e) = cfg.validate() {
                eprintln!("⚠️  Configuration validation warning: {}", e);
                eprintln!("⚠️  Please set NOTION_API_SECRET environment variable");
            }
            Arc::new(cfg)
        }
        Err(e) => {
            eprintln!("⚠️  Failed to load configuration: {}", e);
            eprintln!("⚠️  Using default configuration. Set NOTION_API_SECRET to enable Notion integration.");
            Arc::new(AppConfig::default())
        }
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--minimized"]),
        ))
        .plugin(tauri_plugin_shell::init())
        .setup(move |app| {
            // Initialize logging in debug mode
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            log::info!("Starting {} v{}", config.app_name, config.app_version);

            // Check if started with --minimized flag (from autostart)
            let args: Vec<String> = std::env::args().collect();
            if args.iter().any(|arg| arg == "--minimized") {
                log::info!("Started with --minimized flag, hiding window");
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.hide();
                }
            }

            // Setup system tray
            if let Err(e) = setup_system_tray(&app.handle()) {
                log::error!("Failed to setup system tray: {}", e);
            }

            // Force enable autostart (mandatory for monitoring)
            #[cfg(not(debug_assertions))]
            {
                use tauri_plugin_autostart::ManagerExt;
                let autostart_manager = app.autolaunch();
                // Always enable autostart, regardless of current state
                if let Err(e) = autostart_manager.enable() {
                    log::error!("Failed to enable autostart: {}", e);
                    log::error!("This application requires autostart to function properly");
                } else {
                    log::info!("Autostart enabled (mandatory for monitoring)");
                }
            }

            // Initialize tracking service
            let tracker = Arc::new(TrackingService::new());
            let tracker_clone = Arc::clone(&tracker);

            // Initialize Notion page ID storage
            let notion_page_id = Arc::new(TokioMutex::new(None));

            // Initialize terminal working directory (start at user's home directory)
            let home_dir = std::env::var("HOME")
                .or_else(|_| std::env::var("USERPROFILE"))
                .unwrap_or_else(|_| ".".to_string());
            let terminal_cwd = Arc::new(TokioMutex::new(home_dir));

            // Get hostname for page validation
            let sys_info = SystemInfo::collect();
            let hostname = sys_info.hostname.clone();

            // Track if page has been validated
            let page_validated = Arc::new(TokioMutex::new(false));

            // Track usage statistics database ID
            let usage_database_id = Arc::new(TokioMutex::new(None));

            // Clone for background tasks
            let config_clone = Arc::clone(&config);
            let page_id_clone = Arc::clone(&notion_page_id);
            let tracker_clone2 = Arc::clone(&tracker);
            let terminal_cwd_clone = Arc::clone(&terminal_cwd);
            let hostname_clone = hostname.clone();
            let page_validated_clone = Arc::clone(&page_validated);
            let usage_database_id_clone = Arc::clone(&usage_database_id);

            // Start background tracking task
            tauri::async_runtime::spawn(async move {
                start_tracking(tracker_clone).await;
            });

            // Start background Notion sync task
            tauri::async_runtime::spawn(async move {
                start_notion_sync(config_clone, page_id_clone, tracker_clone2, terminal_cwd_clone, hostname_clone, page_validated_clone, usage_database_id_clone).await;
            });

            // Store application state
            app.manage(AppState {
                tracker,
                config,
                notion_page_id,
                terminal_cwd,
                hostname,
                page_validated,
                usage_database_id,
            });

            log::info!("Application initialized successfully");
            log::info!("Notion sync will update every 5 seconds");

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_app_stats,
            get_app_usage,
            reset_stats,
            update_app_limits,
            get_app_limits,
            register_client_with_notion,
            sync_notion_now,
            is_autostart_enabled,
            set_autostart_enabled,
            show_window,
            hide_window,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
