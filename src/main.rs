use std::io::{self, Write};
use std::process;

use clap::Parser;
use colored::*;

use search_things::{SearchConfig, run_search};

#[derive(Parser)]
#[command(name = "SearchThings", about = "Paralel dosya arama aracı", version)]
struct Cli {
    /// Aranacak klasör yolu
    #[arg(short, long)]
    path: Option<String>,

    /// Aranacak metin
    #[arg(short, long)]
    search: Option<String>,

    /// Büyük/küçük harf duyarlı arama
    #[arg(long)]
    case_sensitive: bool,

    /// Regex deseni kullan
    #[arg(long)]
    regex: bool,

    /// Dosya uzantıları
    #[arg(short = 'e', long, default_value = "txt")]
    pattern: String,

    /// Maksimum sonuç sayısı
    #[arg(short, long)]
    max_results: Option<usize>,

    /// Sessiz mod (progress bar gösterme)
    #[arg(short, long)]
    quiet: bool,
}

fn read_input(prompt: &str) -> String {
    print!("{}", prompt.yellow());
    let _ = io::stdout().flush();
    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_err() {
        eprintln!("{}", "Okuma hatası".red());
        process::exit(1);
    }
    input.trim().to_string()
}

fn main() {
    println!("{}", "SearchThings".bright_cyan().bold());

    let cli = Cli::parse();

    let folder_path = match &cli.path {
        Some(p) => p.clone(),
        None => read_input("\nKlasör yolunu girin: "),
    };

    if folder_path.is_empty() {
        eprintln!("{}", "Klasör yolu boş olamaz".red());
        process::exit(1);
    }

    let search_text = match &cli.search {
        Some(s) => s.clone(),
        None => read_input("Aranacak metni girin: "),
    };

    if search_text.is_empty() {
        eprintln!("{}", "Arama metni boş olamaz".red());
        process::exit(1);
    }

    let file_pattern = if cli.pattern == "txt" {
        None
    } else {
        Some(cli.pattern.clone())
    };

    let config = SearchConfig {
        folder_path,
        search_text,
        case_sensitive: cli.case_sensitive,
        use_regex: cli.regex,
        file_pattern,
        max_results: cli.max_results,
        quiet: cli.quiet,
    };

    let results = match run_search(config) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{}", e.red());
            process::exit(1);
        }
    };

    println!("\n{}", "Sonuçlar".bright_green().bold());
    println!(
        "{}",
        format!("Toplam {} eşleşme bulundu\n", results.len()).bright_yellow()
    );

    if results.is_empty() {
        println!("{}", "Hiç eşleşme bulunamadı.".red());
    } else {
        for (index, result) in results.iter().enumerate() {
            println!("{}", format!("Sonuç #{}", index + 1).bright_black());
            println!(
                "{}",
                format!("Dosya: {}", result.file_path).bright_blue().bold()
            );
            println!("{}", format!("Satır: {}", result.line_number).yellow());
            println!("{}", format!("İçerik: {}", result.line_content).white());
            println!();
        }

        println!(
            "{}",
            format!("Arama tamamlandı! {} eşleşme bulundu.", results.len())
                .bright_green()
                .bold()
        );
    }
}
