use std::fs;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

use encoding_rs_io::DecodeReaderBytesBuilder;

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub file_path: String,
    pub line_number: usize,
    pub line_content: String,
}

pub fn search_in_file(
    path: &Path,
    search_text: &str,
    case_sensitive: bool,
    use_regex: bool,
) -> Vec<SearchResult> {
    let mut results = Vec::new();

    let file = match fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return results,
    };

    let reader = match create_reader(file) {
        Ok(r) => r,
        Err(_) => return results,
    };

    if use_regex {
        if let Ok(re) = regex::RegexBuilder::new(search_text)
            .case_insensitive(!case_sensitive)
            .build()
        {
            for (line_num, line) in reader.lines().enumerate() {
                if let Ok(line_content) = line
                    && re.is_match(&line_content)
                {
                    results.push(SearchResult {
                        file_path: path.to_string_lossy().to_string(),
                        line_number: line_num + 1,
                        line_content: line_content.trim().to_string(),
                    });
                }
            }
        }
    } else if case_sensitive {
        for (line_num, line) in reader.lines().enumerate() {
            if let Ok(line_content) = line
                && line_content.contains(search_text)
            {
                results.push(SearchResult {
                    file_path: path.to_string_lossy().to_string(),
                    line_number: line_num + 1,
                    line_content: line_content.trim().to_string(),
                });
            }
        }
    } else {
        let search_lower = search_text.to_lowercase();
        for (line_num, line) in reader.lines().enumerate() {
            if let Ok(line_content) = line
                && line_content.to_lowercase().contains(&search_lower)
            {
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

fn create_reader(file: fs::File) -> io::Result<BufReader<impl io::Read>> {
    let decoder = DecodeReaderBytesBuilder::new().build(file);
    Ok(BufReader::new(decoder))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_file(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(content.as_bytes()).unwrap();
        file.flush().unwrap();
        file
    }

    #[test]
    fn test_basic_search() {
        let file = create_test_file("hello world\nfoo bar\nhello rust\n");
        let results = search_in_file(file.path(), "hello", false, false);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].line_number, 1);
        assert_eq!(results[1].line_number, 3);
    }

    #[test]
    fn test_case_insensitive() {
        let file = create_test_file("Hello World\nhello again\n");
        let results = search_in_file(file.path(), "hello", false, false);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_case_sensitive() {
        let file = create_test_file("Hello World\nhello again\n");
        let results = search_in_file(file.path(), "hello", true, false);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].line_content, "hello again");
    }

    #[test]
    fn test_regex_search() {
        let file = create_test_file("foo123\nbar456\nfoo789\n");
        let results = search_in_file(file.path(), r"foo\d+", false, true);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_regex_case_insensitive() {
        let file = create_test_file("Hello123\nhello456\n");
        let results = search_in_file(file.path(), r"hello\d+", false, true);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_empty_file() {
        let file = create_test_file("");
        let results = search_in_file(file.path(), "test", false, false);
        assert!(results.is_empty());
    }

    #[test]
    fn test_no_match() {
        let file = create_test_file("hello world\n");
        let results = search_in_file(file.path(), "xyz", false, false);
        assert!(results.is_empty());
    }

    #[test]
    fn test_nonexistent_file() {
        let results = search_in_file(Path::new("/nonexistent/file.txt"), "test", false, false);
        assert!(results.is_empty());
    }
}
