pub fn format_size(bytes: u64) -> String {
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

pub fn format_speed(bytes_per_sec: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes_per_sec >= GB {
        format!("{:.2} GB/s", bytes_per_sec as f64 / GB as f64)
    } else if bytes_per_sec >= MB {
        format!("{:.1} MB/s", bytes_per_sec as f64 / MB as f64)
    } else if bytes_per_sec >= KB {
        format!("{:.0} KB/s", bytes_per_sec as f64 / KB as f64)
    } else {
        format!("{} B/s", bytes_per_sec)
    }
}

pub fn format_eta(seconds: u64) -> String {
    if seconds == 0 {
        return "Hesaplanıyor...".to_string();
    }

    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;

    if hours > 0 {
        format!("{}s {}dk {}sn", hours, minutes, secs)
    } else if minutes > 0 {
        format!("{}dk {}sn", minutes, secs)
    } else {
        format!("{}sn", secs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_size_bytes() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(500), "500 B");
        assert_eq!(format_size(1023), "1023 B");
    }

    #[test]
    fn test_format_size_kb() {
        assert_eq!(format_size(1024), "1.00 KB");
        assert_eq!(format_size(1536), "1.50 KB");
    }

    #[test]
    fn test_format_size_mb() {
        assert_eq!(format_size(1048576), "1.00 MB");
    }

    #[test]
    fn test_format_size_gb() {
        assert_eq!(format_size(1073741824), "1.00 GB");
    }

    #[test]
    fn test_format_size_tb() {
        assert_eq!(format_size(1099511627776), "1.00 TB");
    }

    #[test]
    fn test_format_speed_zero() {
        assert_eq!(format_speed(0), "0 B/s");
    }

    #[test]
    fn test_format_speed_kb() {
        assert_eq!(format_speed(2048), "2 KB/s");
    }

    #[test]
    fn test_format_speed_mb() {
        assert_eq!(format_speed(1048576), "1.0 MB/s");
    }

    #[test]
    fn test_format_speed_gb() {
        assert_eq!(format_speed(1073741824), "1.00 GB/s");
    }

    #[test]
    fn test_format_eta_zero() {
        assert_eq!(format_eta(0), "Hesaplanıyor...");
    }

    #[test]
    fn test_format_eta_seconds_only() {
        assert_eq!(format_eta(30), "30sn");
    }

    #[test]
    fn test_format_eta_minutes() {
        assert_eq!(format_eta(90), "1dk 30sn");
    }

    #[test]
    fn test_format_eta_hours() {
        assert_eq!(format_eta(3661), "1s 1dk 1sn");
    }
}
