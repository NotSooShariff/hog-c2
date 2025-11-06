use crate::models::{AppLimit, AppUsage};
use crate::platform;
use chrono::{Local, Timelike};
use std::collections::HashMap;
use std::sync::Mutex;

pub struct TrackingService {
    usage_data: Mutex<HashMap<String, AppUsage>>,
    limits: Mutex<HashMap<String, AppLimit>>,
    warned_apps: Mutex<HashMap<String, bool>>, // Track which apps have been warned
}

impl TrackingService {
    pub fn new() -> Self {
        Self {
            usage_data: Mutex::new(HashMap::new()),
            limits: Mutex::new(HashMap::new()),
            warned_apps: Mutex::new(HashMap::new()),
        }
    }

    pub fn update_active_window(&self, app_name: String, window_title: String) {
        // Update usage data
        let current_duration = {
            let mut data = self.usage_data.lock().unwrap();

            data.entry(app_name.clone())
                .and_modify(|usage| {
                    usage.duration_seconds += 1;
                    usage.window_title = window_title.clone();
                })
                .or_insert(AppUsage {
                    app_name: app_name.clone(),
                    window_title,
                    duration_seconds: 1,
                });

            // Reset daily stats at midnight
            let now = Local::now();
            if now.hour() == 0 && now.minute() == 0 {
                data.clear();
            }

            data.get(&app_name).map(|u| u.duration_seconds).unwrap_or(0)
        };

        // Check if app has a limit and enforce it
        let should_terminate = {
            let limits = self.limits.lock().unwrap();
            let mut warned = self.warned_apps.lock().unwrap();

            if let Some(limit) = limits.get(&app_name) {
                if limit.enabled {
                    let duration_minutes = current_duration / 60;

                    // Check if max duration exceeded
                    if duration_minutes >= limit.max_duration_minutes {
                        log::warn!(
                            "App '{}' has exceeded max duration ({} minutes). Terminating...",
                            app_name,
                            limit.max_duration_minutes
                        );
                        true
                    } else if duration_minutes >= limit.notification_threshold_minutes
                        && !warned.get(&app_name).unwrap_or(&false)
                    {
                        // Mark as warned to avoid spam
                        warned.insert(app_name.clone(), true);
                        log::info!(
                            "App '{}' approaching limit ({}/{} minutes)",
                            app_name,
                            duration_minutes,
                            limit.max_duration_minutes
                        );
                        false
                    } else {
                        false
                    }
                } else {
                    false
                }
            } else {
                false
            }
        };

        // Terminate the process if limit exceeded
        if should_terminate {
            if let Err(e) = platform::terminate_process_by_name(&app_name) {
                log::error!("Failed to terminate process '{}': {}", app_name, e);
            } else {
                log::info!("Successfully blocked app: {}", app_name);
            }
        }
    }

    pub fn get_stats(&self) -> Vec<AppUsage> {
        let data = self.usage_data.lock().unwrap();
        data.values().cloned().collect()
    }

    pub fn get_daily_total(&self, app_name: &str) -> i64 {
        let data = self.usage_data.lock().unwrap();
        data.get(app_name)
            .map(|usage| usage.duration_seconds)
            .unwrap_or(0)
    }

    pub fn reset_daily_stats(&self) {
        let mut data = self.usage_data.lock().unwrap();
        data.clear();

        // Also reset warned apps
        let mut warned = self.warned_apps.lock().unwrap();
        warned.clear();
    }

    pub fn update_limits(&self, new_limits: Vec<AppLimit>) {
        let mut limits = self.limits.lock().unwrap();
        limits.clear();

        for limit in new_limits {
            limits.insert(limit.app_name.clone(), limit);
        }

        log::info!("Updated app limits. Total limits: {}", limits.len());
    }

    pub fn get_limits(&self) -> Vec<AppLimit> {
        let limits = self.limits.lock().unwrap();
        limits.values().cloned().collect()
    }

    pub fn get_active_window_info() -> Option<(String, String)> {
        platform::get_active_window_info()
    }
}

impl Default for TrackingService {
    fn default() -> Self {
        Self::new()
    }
}
