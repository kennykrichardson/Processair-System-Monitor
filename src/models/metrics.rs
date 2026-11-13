#[derive(Debug, Clone, PartialEq)]
pub struct ProcessMetric {
    pub pid: u32,
    pub name: String,
    pub cpu_percent: f32,
    pub memory_bytes: u64,
    pub parent_pid: Option<u32>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SystemSnapshot {
    pub hostname: String,
    pub operating_system: String,
    pub uptime_seconds: u64,
    pub cpu_percent: f32,
    pub cpu_cores: Vec<f32>,
    pub memory_used_bytes: u64,
    pub memory_total_bytes: u64,
    pub swap_used_bytes: u64,
    pub swap_total_bytes: u64,
    pub process_count: usize,
    pub processes: Vec<ProcessMetric>,
}

impl SystemSnapshot {

    pub fn memory_percent(&self) -> f64 {
        percentage(self.memory_used_bytes, self.memory_total_bytes)
    }

    pub fn swap_percent(&self) -> f64 {
        percentage(self.swap_used_bytes, self.swap_total_bytes)
    }
}

pub fn percentage(used: u64, total: u64) -> f64 {
    if total == 0 {
        return 0.0;
    }

    (used as f64 / total as f64 * 100.0).clamp(0.0, 100.0)
}

/// Format byte counts using binary units (KiB, MiB, GiB, TiB).
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];

    if bytes < 1024 {
        return format!("{bytes} B");
    }

    let mut value = bytes as f64;
    let mut unit_index = 0;

    while value >= 1024.0 && unit_index < UNITS.len() - 1 {
        value /= 1024.0;
        unit_index += 1;
    }

    format!("{value:.1} {}", UNITS[unit_index])
}

pub fn format_uptime(total_seconds: u64) -> String {
    let days = total_seconds / 86_400;
    let hours = (total_seconds % 86_400) / 3_600;
    let minutes = (total_seconds % 3_600) / 60;
    let seconds = total_seconds % 60;

    if days > 0 {
        format!("{days}d {hours:02}h {minutes:02}m")
    } else if hours > 0 {
        format!("{hours}h {minutes:02}m {seconds:02}s")
    } else {
        format!("{minutes}m {seconds:02}s")
    }
}

#[cfg(test)]
mod tests {
    use super::{format_bytes, format_uptime, percentage};

    #[test]
    fn formats_bytes_in_binary_units() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(1_536), "1.5 KiB");
        assert_eq!(format_bytes(1_073_741_824), "1.0 GiB");
    }

    #[test]
    fn percentage_handles_zero_and_caps_invalid_values() {
        assert_eq!(percentage(0, 0), 0.0);
        assert_eq!(percentage(25, 100), 25.0);
        assert_eq!(percentage(150, 100), 100.0);
    }

    #[test]
    fn formats_uptime_with_readable_units() {
        assert_eq!(format_uptime(59), "0m 59s");
        assert_eq!(format_uptime(3_661), "1h 01m 01s");
        assert_eq!(format_uptime(183_845), "2d 03h 04m");
    }
}
