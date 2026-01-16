use sysinfo::{System, Networks, Disks};
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_usage: f32,
    pub memory: u64,
    pub disk_usage: u64,
}

#[derive(Debug, Clone)]
pub struct CpuInfo {
    pub usage: f32,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct MemoryInfo {
    pub total: u64,
    pub used: u64,
    pub available: u64,
}

#[derive(Debug, Clone)]
pub struct DiskInfo {
    pub name: String,
    pub total: u64,
    pub available: u64,
}

#[derive(Debug, Clone)]
pub struct NetworkInfo {
    pub interface: String,
    pub received: u64,
    pub transmitted: u64,
}

#[derive(Debug, Clone)]
pub struct SystemSnapshot {
    pub timestamp: Instant,
    pub processes: Vec<ProcessInfo>,
    pub cpus: Vec<CpuInfo>,
    pub memory: MemoryInfo,
    pub disks: Vec<DiskInfo>,
    pub networks: Vec<NetworkInfo>,
    pub global_cpu_usage: f32,
}

pub struct SystemMonitor {
    sys: System,
    networks: Networks,
    disks: Disks,
    last_update: Instant,
    update_interval: Duration,
}

impl SystemMonitor {
    pub fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();

        Self {
            sys,
            networks: Networks::new_with_refreshed_list(),
            disks: Disks::new_with_refreshed_list(),
            last_update: Instant::now(),
            update_interval: Duration::from_millis(1000),
        }
    }

    pub fn update(&mut self) {
        if self.last_update.elapsed() < self.update_interval {
            return;
        }

        self.sys.refresh_all();
        self.networks.refresh(true);
        self.disks.refresh(true);
        self.last_update = Instant::now();
    }

    pub fn snapshot(&self) -> SystemSnapshot {
        let processes = self.sys.processes()
            .iter()
            .map(|(pid, process)| {
                ProcessInfo {
                    pid: pid.as_u32(),
                    name: process.name().to_string_lossy().to_string(),
                    cpu_usage: process.cpu_usage(),
                    memory: process.memory(),
                    disk_usage: process.disk_usage().written_bytes,
                }
            })
            .collect();

        let cpus = self.sys.cpus()
            .iter()
            .map(|cpu| CpuInfo {
                usage: cpu.cpu_usage(),
                name: cpu.name().to_string(),
            })
            .collect();

        let memory = MemoryInfo {
            total: self.sys.total_memory(),
            used: self.sys.used_memory(),
            available: self.sys.available_memory(),
        };

        let disks = self.disks.iter()
            .map(|disk| DiskInfo {
                name: disk.name().to_string_lossy().to_string(),
                total: disk.total_space(),
                available: disk.available_space(),
            })
            .collect();

        let networks = self.networks.iter()
            .map(|(interface, data)| NetworkInfo {
                interface: interface.to_string(),
                received: data.total_received(),
                transmitted: data.total_transmitted(),
            })
            .collect();

        SystemSnapshot {
            timestamp: Instant::now(),
            processes,
            cpus,
            memory,
            disks,
            networks,
            global_cpu_usage: self.sys.global_cpu_usage(),
        }
    }

    pub fn get_process_count(&self) -> usize {
        self.sys.processes().len()
    }

    pub fn get_cpu_count(&self) -> usize {
        self.sys.cpus().len()
    }
}

impl Default for SystemMonitor {
    fn default() -> Self {
        Self::new()
    }
}

pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if bytes >= TB {
        format!("{:.2} TB", bytes as f64 / TB as f64)
    } else if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
