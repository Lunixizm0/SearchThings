pub mod format;
pub mod progress;
pub mod search;

use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use colored::*;
use rayon::prelude::*;
use walkdir::WalkDir;

use progress::ProgressTracker;
use search::SearchResult;

pub struct SearchConfig {
    pub folder_path: String,
    pub search_text: String,
    pub case_sensitive: bool,
    pub use_regex: bool,
    pub file_pattern: Option<String>,
    pub max_results: Option<usize>,
    pub quiet: bool,
}

pub fn collect_files(folder_path: &str, file_pattern: &Option<String>) -> Vec<(PathBuf, u64)> {
    let extensions: Vec<String> = file_pattern
        .as_ref()
        .map(|p| {
            p.split(',')
                .map(|s| s.trim().trim_start_matches('*').trim_start_matches('.').to_string())
                .collect()
        })
        .unwrap_or_else(|| vec!["txt".to_string()]);

    WalkDir::new(folder_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .and_then(|s| s.to_str())
                .map(|ext| extensions.iter().any(|e| e == ext))
                .unwrap_or(false)
        })
        .filter_map(|e| {
            let path = e.path().to_path_buf();
            std::fs::metadata(&path)
                .ok()
                .map(|m| (path, m.len()))
        })
        .collect()
}

pub fn run_search(config: SearchConfig) -> Result<Vec<SearchResult>, String> {
    let path = Path::new(&config.folder_path);

    if !path.is_dir() {
        return Err(format!("'{}' geçerli bir klasör değil", config.folder_path));
    }

    if config.search_text.is_empty() {
        return Err("Arama metni boş olamaz".to_string());
    }

    let txt_files = collect_files(&config.folder_path, &config.file_pattern);
    let total_files = txt_files.len();
    let total_size: u64 = txt_files.iter().map(|(_, size)| size).sum();

    if total_files == 0 {
        return Err("Uygun dosya bulunamadı".to_string());
    }

    if !config.quiet {
        println!("{}", format!("Toplam {} dosya bulundu", total_files).cyan());
        println!(
            "{}",
            format!("Toplam boyut: {}\n", format::format_size(total_size)).cyan()
        );
        println!(
            "{}",
            format!(
                "'{}' içinde '{}' aranıyor...\n",
                config.folder_path, config.search_text
            )
            .green()
        );
    }

    let tracker = Arc::new(ProgressTracker::new(total_files, total_size));

    let progress_handle = if !config.quiet {
        Some(tracker.spawn_progress_thread())
    } else {
        None
    };

    let search_text = config.search_text.clone();
    let case_sensitive = config.case_sensitive;
    let use_regex = config.use_regex;
    let max_results = config.max_results;
    let tracker_clone = Arc::clone(&tracker);

    let all_results: Vec<SearchResult> = txt_files
        .par_iter()
        .flat_map(|(path, size)| {
            tracker_clone.update_current_file(path);

            let file_results =
                search::search_in_file(path, &search_text, case_sensitive, use_regex);

            tracker_clone.increment(*size);

            file_results
        })
        .collect();

    let results: Vec<SearchResult> = match max_results {
        Some(limit) => all_results.into_iter().take(limit).collect(),
        None => all_results,
    };

    if let Some(handle) = progress_handle {
        let _ = handle.join();
    }

    if !config.quiet {
        print!("\r{}\r", " ".repeat(150));
        let _ = std::io::stdout().flush();
    }

    Ok(results)
}
