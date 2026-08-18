use std::io::{self, Write};
use std::path::Path;
use std::sync::atomic::{AtomicUsize, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use colored::*;

use crate::format::{format_eta, format_size, format_speed};

pub struct ProgressTracker {
    pub processed_files: Arc<AtomicUsize>,
    pub processed_bytes: Arc<AtomicU64>,
    pub current_file: Arc<Mutex<String>>,
    pub total_files: usize,
    pub total_size: u64,
    pub start_time: Instant,
}

impl ProgressTracker {
    pub fn new(total_files: usize, total_size: u64) -> Self {
        Self {
            processed_files: Arc::new(AtomicUsize::new(0)),
            processed_bytes: Arc::new(AtomicU64::new(0)),
            current_file: Arc::new(Mutex::new(String::new())),
            total_files,
            total_size,
            start_time: Instant::now(),
        }
    }

    pub fn update_current_file(&self, path: &Path) {
        if let Ok(mut current) = self.current_file.lock() {
            *current = path.to_string_lossy().to_string();
        }
    }

    pub fn increment(&self, bytes: u64) {
        self.processed_files.fetch_add(1, Ordering::Relaxed);
        self.processed_bytes.fetch_add(bytes, Ordering::Relaxed);
    }

    pub fn spawn_progress_thread(&self) -> std::thread::JoinHandle<()> {
        let processed_files = Arc::clone(&self.processed_files);
        let processed_bytes = Arc::clone(&self.processed_bytes);
        let current_file = Arc::clone(&self.current_file);
        let total_files = self.total_files;
        let total_size = self.total_size;
        let start_time = self.start_time;

        std::thread::spawn(move || {
            let mut last_bytes = 0u64;
            let mut last_time = Instant::now();

            loop {
                let files_done = processed_files.load(Ordering::Relaxed);
                let bytes_done = processed_bytes.load(Ordering::Relaxed);
                let current = current_file
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .clone();

                if files_done >= total_files {
                    break;
                }

                let now = Instant::now();
                let elapsed_instant = now.duration_since(last_time).as_secs_f64();
                let bytes_diff = bytes_done.saturating_sub(last_bytes);
                let instant_speed = if elapsed_instant > 0.5 {
                    last_bytes = bytes_done;
                    last_time = now;
                    (bytes_diff as f64 / elapsed_instant) as u64
                } else {
                    0
                };

                let total_elapsed = start_time.elapsed().as_secs_f64();
                let avg_speed = if total_elapsed > 0.0 {
                    (bytes_done as f64 / total_elapsed) as u64
                } else {
                    0
                };

                let remaining_bytes = total_size.saturating_sub(bytes_done);
                let eta_seconds = remaining_bytes.checked_div(avg_speed).unwrap_or(0);

                let percentage = if total_size > 0 {
                    (bytes_done as f64 / total_size as f64 * 100.0).min(100.0)
                } else {
                    0.0
                };
                let bar_width = 30;
                let filled = (bar_width as f64 * percentage / 100.0) as usize;
                let empty = bar_width - filled;

                let bar = format!(
                    "{}{}",
                    "+".repeat(filled).bright_green(),
                    "-".repeat(empty).bright_black()
                );

                let file_name = if current.is_empty() {
                    "Başlatılıyor...".to_string()
                } else {
                    Path::new(&current)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or(&current)
                        .to_string()
                };

                let display_name = if file_name.chars().count() > 35 {
                    let end = file_name
                        .char_indices()
                        .nth(32)
                        .map(|(i, _)| i)
                        .unwrap_or(file_name.len());
                    format!("{}...", &file_name[..end])
                } else {
                    file_name
                };

                print!(
                    "\r{} [{}/{}] {} {}/{} | {} | {} | Ort: {} | Kalan: {}{}",
                    bar,
                    files_done,
                    total_files,
                    format!("{:.1}%", percentage).bright_yellow(),
                    format_size(bytes_done),
                    format_size(total_size),
                    display_name.bright_blue(),
                    format_speed(instant_speed).bright_green(),
                    format_speed(avg_speed).bright_cyan(),
                    format_eta(eta_seconds).bright_magenta(),
                    " ".repeat(10)
                );
                let _ = io::stdout().flush();

                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        })
    }
}
