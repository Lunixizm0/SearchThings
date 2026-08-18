use search_things::{SearchConfig, collect_files, run_search};
use std::fs;
use tempfile::TempDir;

fn setup_test_dir() -> TempDir {
    let dir = TempDir::new().unwrap();
    let base = dir.path();

    // Various text files
    fs::write(base.join("hello.txt"), "Hello World\nfoo bar\nHello Rust\n").unwrap();
    fs::write(
        base.join("data.txt"),
        "test data\nanother line\nfoo bar baz\n",
    )
    .unwrap();
    fs::write(base.join("empty.txt"), "").unwrap();
    fs::write(
        base.join("log.txt"),
        "ERROR: something failed\nINFO: all good\nERROR: again\n",
    )
    .unwrap();

    // Non-text file (should be ignored by default)
    fs::write(base.join("image.bin"), "not a text file").unwrap();

    // Subdirectory with more files
    let sub = base.join("subdir");
    fs::create_dir(&sub).unwrap();
    fs::write(sub.join("nested.txt"), "nested hello\nfoo bar nested\n").unwrap();

    dir
}

#[test]
fn test_collect_files_default_txt() {
    let dir = setup_test_dir();
    let files = collect_files(&dir.path().to_string_lossy(), &None);
    assert_eq!(files.len(), 5); // hello.txt, data.txt, empty.txt, log.txt, subdir/nested.txt
}

#[test]
fn test_collect_files_custom_pattern() {
    let dir = setup_test_dir();
    let files = collect_files(&dir.path().to_string_lossy(), &Some("txt,log".to_string()));
    assert_eq!(files.len(), 5); // same as default since log extension is already included via .txt check... actually log.txt has .txt extension
}

#[test]
fn test_collect_files_nonexistent_dir() {
    let files = collect_files("/nonexistent/path", &None);
    assert!(files.is_empty());
}

#[test]
fn test_run_search_basic() {
    let dir = setup_test_dir();
    let config = SearchConfig {
        folder_path: dir.path().to_string_lossy().to_string(),
        search_text: "hello".to_string(),
        case_sensitive: false,
        use_regex: false,
        file_pattern: None,
        max_results: None,
        quiet: true,
    };

    let results = run_search(config).unwrap();
    assert!(results.len() >= 2); // hello.txt + subdir/nested.txt
    for r in &results {
        assert!(r.line_content.to_lowercase().contains("hello"));
    }
}

#[test]
fn test_run_search_case_sensitive() {
    let dir = setup_test_dir();
    let config = SearchConfig {
        folder_path: dir.path().to_string_lossy().to_string(),
        search_text: "Hello".to_string(),
        case_sensitive: true,
        use_regex: false,
        file_pattern: None,
        max_results: None,
        quiet: true,
    };

    let results = run_search(config).unwrap();
    // Only "Hello World" and "Hello Rust" in hello.txt (case sensitive)
    assert!(results.len() >= 2);
    for r in &results {
        assert!(r.line_content.contains("Hello"));
    }
}

#[test]
fn test_run_search_regex() {
    let dir = setup_test_dir();
    let config = SearchConfig {
        folder_path: dir.path().to_string_lossy().to_string(),
        search_text: r"ERROR.*failed".to_string(),
        case_sensitive: false,
        use_regex: true,
        file_pattern: None,
        max_results: None,
        quiet: true,
    };

    let results = run_search(config).unwrap();
    assert_eq!(results.len(), 1);
    assert!(results[0].line_content.contains("ERROR"));
}

#[test]
fn test_run_search_max_results() {
    let dir = setup_test_dir();
    let config = SearchConfig {
        folder_path: dir.path().to_string_lossy().to_string(),
        search_text: "foo".to_string(),
        case_sensitive: false,
        use_regex: false,
        file_pattern: None,
        max_results: Some(2),
        quiet: true,
    };

    let results = run_search(config).unwrap();
    assert!(results.len() <= 2);
}

#[test]
fn test_run_search_no_match() {
    let dir = setup_test_dir();
    let config = SearchConfig {
        folder_path: dir.path().to_string_lossy().to_string(),
        search_text: "zzzznotfound".to_string(),
        case_sensitive: false,
        use_regex: false,
        file_pattern: None,
        max_results: None,
        quiet: true,
    };

    let results = run_search(config).unwrap();
    assert!(results.is_empty());
}

#[test]
fn test_run_search_invalid_folder() {
    let config = SearchConfig {
        folder_path: "/nonexistent/path".to_string(),
        search_text: "test".to_string(),
        case_sensitive: false,
        use_regex: false,
        file_pattern: None,
        max_results: None,
        quiet: true,
    };

    let result = run_search(config);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("geçerli bir klasör değil"));
}

#[test]
fn test_run_search_empty_query() {
    let dir = setup_test_dir();
    let config = SearchConfig {
        folder_path: dir.path().to_string_lossy().to_string(),
        search_text: "".to_string(),
        case_sensitive: false,
        use_regex: false,
        file_pattern: None,
        max_results: None,
        quiet: true,
    };

    let result = run_search(config);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("boş olamaz"));
}
