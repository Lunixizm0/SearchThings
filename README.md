# SearchThings

Paralel dosya arama aracı. Dizinlerdeki dosyaları çoklu iş parçacığı ile tarar, gerçek zamanlı ilerleme çubuğu gösterir.

## Özellikler

- Paralel arama (rayon)
- Gerçek zamanlı progress bar (hız, ETA, yüzde)
- Regex desteği
- Büyük/küçük harf duyarlılık seçeneği
- Çoklu dosya formatı desteği (txt, log, md, ...)
- Otomatik encoding algılama
- Komut satırı argümanları ve interaktif mod

## Kurulum

```bash
cargo install --path .
```

## Kullanım

### İnteraktif mod

```bash
search-things
# Klasör yolunu girin: /home/user/documents
# Aranacak metni girin: hello world
```

### Komut satırı argümanları

```bash
search-things --path /home/user/docs --search "error"
search-things -p /home/user/docs -s "error" --regex
search-things -p /home/user/docs -s "TODO" -p "rs,toml"
search-things -p /home/user/docs -s "fixme" --case-sensitive --max-results 10
search-things -p /home/user/docs -s "test" --quiet
```

## Seçenekler

| Seçenek | Açıklama |
|---|---|
| `-p, --path` | Aranacak klasör yolu |
| `-s, --search` | Aranacak metin |
| `--case-sensitive` | Büyük/küçük harf duyarlı |
| `--regex` | Regex deseni kullan |
| `-f, --pattern` | Dosya uzantıları (virgülle ayrılmış, varsayılan: txt) |
| `-m, --max-results` | Maksimum sonuç sayısı |
| `-q, --quiet` | Sessiz mod (progress bar yok) |

## Geliştirme

```bash
cargo build
cargo test
cargo clippy
cargo fmt
```

## Yapı

```
src/
├── main.rs        # CLI giriş noktası
├── lib.rs         # Ana arama mantığı
├── search.rs      # Dosya arama fonksiyonları
├── progress.rs    # İlerleme çubuğu
└── format.rs      # Boyut/hız/ETA formatlama
```
