use sysinfo::{
    CpuRefreshKind, MemoryRefreshKind, ProcessRefreshKind, ProcessesToUpdate, RefreshKind, System,
};

use crate::models::metrics::{ProcessMetric, SystemSnapshot};

/// Owns the long-lived sysinfo System instance.
///
/// Keeping this instance alive matters: sysinfo calculates some values, such as
/// CPU usage, by comparing the current sample with a previous sample.
pub struct SystemCollector {
    system: System,
}

impl SystemCollector {
    pub fn new() -> Result<Self, sysinfo::Error> {
        let refresh_plan = RefreshKind::nothing()
            .with_cpu(CpuRefreshKind::everything())
            .with_memory(MemoryRefreshKind::everything())
            .with_processes(
                ProcessRefreshKind::nothing()
                    .with_cpu()
                    .with_memory()
                    .without_tasks(),
            );
        let system = System::new_with_specifics(refresh_plan)?;
        Ok(Self { system })
    }
    pub fn refresh(&mut self) {
        self.system.refresh_cpu_usage();
        self.system.refresh_memory();
        self.system.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::nothing()
                .with_cpu()
                .with_memory()
                .without_tasks(),
        );
    }

    pub fn snapshot(&self) -> SystemSnapshot {
        let cpu_cores = self
            .system
            .cpus()
            .iter()
            .map(|cpu| cpu.usage())
            .collect();

        let processes: Vec<ProcessMetric> = self
            .system
            .processes()
            .iter()
            .map(|(pid, process)| ProcessMetric {
                pid: pid.as_u32(),
                name: process.name().to_string_lossy().into_owned(),
                cpu_percent: process.cpu_usage(),
                memory_bytes: process.memory(),
                parent_pid: process.parent().map(|parent| parent.as_u32()),
            })
            .collect();

        SystemSnapshot {
            hostname: System::host_name().unwrap_or_else(|_| "unknown-host".to_owned()),
            operating_system: std::env::consts::OS.to_owned(),
            uptime_seconds: System::uptime().unwrap_or_default(),
            cpu_percent: self.system.global_cpu_usage(),
            cpu_cores,
            memory_used_bytes: self.system.used_memory(),
            memory_total_bytes: self.system.total_memory(),
            swap_used_bytes: self.system.used_swap(),
            swap_total_bytes: self.system.total_swap(),
            process_count: processes.len(),
            processes,
        }
    }
}
