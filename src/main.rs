use std::fs;
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, AtomicU64, Ordering};
use rayon::prelude::*;
use encoding_rs_io::DecodeReaderBytesBuilder;
use walkdir::WalkDir;
use colored::*;

fn main() {
    println!("{}", "=== Metin Arama Aracı ===".bright_cyan().bold());
    
    // Sabit klasör yolu
    let folder_path = r"C:\Users\Lunix\Documents\Breaches";
    
    // Aranacak metni al
    println!("\n{}", "Aranacak metni girin:".yellow());
    let mut search_text = String::new();
    io::stdin().read_line(&mut search_text).expect("Okuma hatası");
    let search_text = search_text.trim();
    
    if search_text.is_empty() {
        println!("{}", "Arama metni boş olamaz!".red());
        return;
    }
    
    println!("\n{}", format!("'{}' içinde '{}' aranıyor...\n", folder_path, search_text).green());
    
    // Tüm txt dosyalarını topla ve boyutlarını hesapla
    let txt_files: Vec<(PathBuf, u64)> = WalkDir::new(folder_path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("txt"))
        .filter_map(|e| {
            let path = e.path().to_path_buf();
            fs::metadata(&path).ok().map(|m| (path, m.len()))
        })
        .collect();
    
    let total_files = txt_files.len();
    let total_size: u64 = txt_files.iter().map(|(_, size)| size).sum();
    
    println!("{}", format!("📊 Toplam {} txt dosyası bulundu", total_files).cyan());
    println!("{}", format!("💾 Toplam boyut: {}\n", format_size(total_size)).cyan());
    
    if total_files == 0 {
        println!("{}", "Hiç txt dosyası bulunamadı!".red());
        return;
    }
    
    // İlerleme takibi için atomik sayaçlar
    let processed_files = Arc::new(AtomicUsize::new(0));
    let processed_bytes = Arc::new(AtomicU64::new(0));
    let current_file = Arc::new(Mutex::new(String::new()));
    let start_time = std::time::Instant::now();
    
    // İlerleme gösterici thread'i
    let processed_files_clone = Arc::clone(&processed_files);
    let processed_bytes_clone = Arc::clone(&processed_bytes);
    let current_file_clone = Arc::clone(&current_file);
    
    let progress_handle = std::thread::spawn(move || {
        let mut last_bytes = 0u64;
        let mut last_time = std::time::Instant::now();
        
        loop {
            let files_done = processed_files_clone.load(Ordering::Relaxed);
            let bytes_done = processed_bytes_clone.load(Ordering::Relaxed);
            let current = current_file_clone.lock().unwrap().clone();
            
            if files_done >= total_files {
                break;
            }
            
            // Real-time hız hesaplama (son 0.5 saniye)
            let now = std::time::Instant::now();
            let elapsed_instant = now.duration_since(last_time).as_secs_f64();
            let bytes_diff = bytes_done.saturating_sub(last_bytes);
            let instant_speed = if elapsed_instant > 0.5 {
                last_bytes = bytes_done;
                last_time = now;
                (bytes_diff as f64 / elapsed_instant) as u64
            } else {
                0
            };
            
            // Ortalama hız hesaplama (başlangıçtan beri)
            let total_elapsed = start_time.elapsed().as_secs_f64();
            let avg_speed = if total_elapsed > 0.0 {
                (bytes_done as f64 / total_elapsed) as u64
            } else {
                0
            };
            
            // Kalan süre tahmini (ortalama hıza göre)
            let remaining_bytes = total_size.saturating_sub(bytes_done);
            let eta_seconds = if avg_speed > 0 {
                remaining_bytes / avg_speed
            } else {
                0
            };
            
            let percentage = (bytes_done as f64 / total_size as f64 * 100.0).min(100.0);
            let bar_width = 30;
            let filled = (bar_width as f64 * percentage / 100.0) as usize;
            let empty = bar_width - filled;
            
            let bar = format!("{}{}",
                "█".repeat(filled).bright_green(),
                "░".repeat(empty).bright_black()
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
            
            // Dosya adını kısalt (max 35 karakter)
            let display_name = if file_name.len() > 35 {
                format!("{}...", &file_name[..32])
            } else {
                file_name
            };
            
            print!("\r{} [{}/{}] {} {}/{} | 📄 {} | ⚡ {}/s | 📊 Ort: {}/s | ⏱️ Kalan: {}{}",
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
                " ".repeat(10) // Eski metni temizlemek için
            );
            io::stdout().flush().unwrap();
            
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    });
    
    // Paralel arama yap
    let results: Vec<SearchResult> = txt_files
        .par_iter()
        .flat_map(|(path, size)| {
            // Mevcut dosyayı güncelle
            if let Ok(mut current) = current_file.lock() {
                *current = path.to_string_lossy().to_string();
            }
            
            let file_results = search_in_file(path, search_text);
            
            // İlerleme sayaçlarını güncelle
            processed_files.fetch_add(1, Ordering::Relaxed);
            processed_bytes.fetch_add(*size, Ordering::Relaxed);
            
            file_results
        })
        .collect();
    
    // Progress thread'inin bitmesini bekle
    progress_handle.join().unwrap();
    
    // Son ilerleme çubuğunu temizle
    print!("\r{}\r", " ".repeat(150));
    io::stdout().flush().unwrap();
    
    // Sonuçları göster
    println!("\n{}", "=== SONUÇLAR ===".bright_green().bold());
    println!("{}", format!("🎯 Toplam {} eşleşme bulundu\n", results.len()).bright_yellow());
    
    if results.is_empty() {
        println!("{}", "❌ Hiç eşleşme bulunamadı.".red());
    } else {
        for (index, result) in results.iter().enumerate() {
            println!("{}", format!("━━━ Sonuç #{} ━━━", index + 1).bright_black());
            println!("{}", format!("📁 Dosya: {}", result.file_path).bright_blue().bold());
            println!("{}", format!("📍 Satır: {}", result.line_number).yellow());
            println!("{}", format!("📝 İçerik: {}", result.line_content).white());
            println!();
        }
        
        println!("{}", format!("✅ Arama tamamlandı! {} eşleşme bulundu.", results.len()).bright_green().bold());
    }
}

#[derive(Debug)]
struct SearchResult {
    file_path: String,
    line_number: usize,
    line_content: String,
}

fn search_in_file(path: &Path, search_text: &str) -> Vec<SearchResult> {
    let mut results = Vec::new();
    
    // Dosyayı aç
    let file = match fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return results,
    };
    
    // Encoding'i algıla ve okuyucu oluştur
    let reader = match create_reader(file) {
        Ok(r) => r,
        Err(_) => return results,
    };
    
    // Satır satır oku ve ara
    for (line_num, line) in reader.lines().enumerate() {
        if let Ok(line_content) = line {
            // Büyük/küçük harf duyarsız arama
            if line_content.to_lowercase().contains(&search_text.to_lowercase()) {
                results.push(SearchResult {
                    file_path: path.to_string_lossy().to_string(),
                    line_number: line_num + 1,
                    line_content: line_content.trim().to_string(),
                });
            }
        }
    }
    
    results
}

fn create_reader(file: fs::File) -> io::Result<BufReader<impl std::io::Read>> {
    // Encoding algılayıcı oluştur - UTF-8, UTF-16, Windows-1252 vb. destekler
    let decoder = DecodeReaderBytesBuilder::new()
        .build(file);
    
    Ok(BufReader::new(decoder))
}

fn format_size(bytes: u64) -> String {
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

fn format_speed(bytes_per_sec: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    
    if bytes_per_sec >= GB {
        format!("{:.2} GB", bytes_per_sec as f64 / GB as f64)
    } else if bytes_per_sec >= MB {
        format!("{:.1} MB", bytes_per_sec as f64 / MB as f64)
    } else if bytes_per_sec >= KB {
        format!("{:.0} KB", bytes_per_sec as f64 / KB as f64)
    } else {
        format!("{} B", bytes_per_sec)
    }
}

fn format_eta(seconds: u64) -> String {
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