use serde::{Deserialize, Serialize};
use sysinfo::System;

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemInfo {
    pub os: String,
    pub os_version: String,
    pub total_ram_gb: f64,
    pub used_ram_gb: f64,
    pub ram_usage_percent: f64,
    pub total_disk_gb: f64,
    pub used_disk_gb: f64,
    pub disk_usage_percent: f64,
    pub cpu_count: usize,
    pub hostname: String,
}

impl SystemInfo {
    pub fn collect() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();

        // OS Information
        let os = System::name().unwrap_or_else(|| "Unknown".to_string());
        let os_version = System::os_version().unwrap_or_else(|| "Unknown".to_string());

        // RAM Information
        let total_ram = sys.total_memory() as f64 / 1_073_741_824.0; // Convert bytes to GB
        let used_ram = sys.used_memory() as f64 / 1_073_741_824.0;
        let ram_usage_percent = if total_ram > 0.0 {
            (used_ram / total_ram) * 100.0
        } else {
            0.0
        };

        // Disk Information
        let mut total_disk = 0u64;
        let mut available_disk = 0u64;

        // sysinfo 0.32+ uses Disks struct
        let disks = sysinfo::Disks::new_with_refreshed_list();
        for disk in &disks {
            total_disk += disk.total_space();
            available_disk += disk.available_space();
        }

        let total_disk_gb = total_disk as f64 / 1_073_741_824.0;
        let used_disk_gb = (total_disk - available_disk) as f64 / 1_073_741_824.0;
        let disk_usage_percent = if total_disk_gb > 0.0 {
            (used_disk_gb / total_disk_gb) * 100.0
        } else {
            0.0
        };

        // CPU Information
        let cpu_count = sys.cpus().len();

        // Hostname
        let hostname = System::host_name().unwrap_or_else(|| "Unknown".to_string());

        SystemInfo {
            os,
            os_version,
            total_ram_gb: (total_ram * 100.0).round() / 100.0,
            used_ram_gb: (used_ram * 100.0).round() / 100.0,
            ram_usage_percent: (ram_usage_percent * 100.0).round() / 100.0,
            total_disk_gb: (total_disk_gb * 100.0).round() / 100.0,
            used_disk_gb: (used_disk_gb * 100.0).round() / 100.0,
            disk_usage_percent: (disk_usage_percent * 100.0).round() / 100.0,
            cpu_count,
            hostname,
        }
    }
}
