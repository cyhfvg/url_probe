use crate::error::InputError;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::PathBuf;
use url::Url;

fn looks_like_url(input: &str) -> bool {
    input.contains("://")
}

/// Parse an HTTP/HTTPS URL.
fn parse_http_url(input: &str, line: Option<usize>) -> Result<Url, InputError> {
    let url = Url::parse(input).map_err(|source| InputError::InvalidTargetUrl {
        value: input.to_string(),
        line,
        source: source.to_string(),
    })?;
    if matches!(url.scheme(), "http" | "https") {
        Ok(url)
    } else {
        Err(InputError::InvalidTargetUrl {
            value: input.to_string(),
            line,
            source: format!("unsupported URL scheme '{}'", url.scheme()),
        })
    }
}

/// Load target URLs from a single URL, a URL list file, or stdin.
pub fn load_targets(file_path: &str) -> Result<Vec<Url>, InputError> {
    let targets = if file_path == "-" {
        let reader = BufReader::new(io::stdin().lock());
        load_targets_from_reader(reader, "stdin")?
    } else if looks_like_url(file_path) {
        vec![parse_http_url(file_path, None)?]
    } else {
        let path = PathBuf::from(file_path);
        let file = File::open(path).map_err(|why| InputError::OpenTargetFileError {
            path: file_path.to_string(),
            source: why,
        })?;
        let reader = BufReader::new(file);
        load_targets_from_reader(reader, file_path)?
    };
    Ok(targets)
}

fn load_targets_from_reader<R: BufRead>(
    reader: R,
    source_name: &str,
) -> Result<Vec<Url>, InputError> {
    let mut targets = Vec::new();
    for (index, line_result) in reader.lines().enumerate() {
        let line = line_result.map_err(|why| InputError::ReadTargetLine { source: why })?;
        let line = line.trim();
        // Skip empty lines and comment lines.
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        targets.push(parse_http_url(line, Some(index + 1))?);
    }

    if targets.is_empty() {
        return Err(InputError::NoTargets {
            source: source_name.to_string(),
        });
    }

    Ok(targets)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn load_targets_from_reader_reports_invalid_url_line() {
        let input = Cursor::new("https://example.com\nftp://example.com\n");
        let err = load_targets_from_reader(input, "fixture").expect_err("invalid URL error");

        assert!(matches!(
            err,
            InputError::InvalidTargetUrl { line: Some(2), .. }
        ));
    }

    #[test]
    fn load_targets_from_reader_reports_empty_target_set() {
        let input = Cursor::new("# comment\n\n");
        let err = load_targets_from_reader(input, "fixture").expect_err("empty target set");

        assert!(matches!(err, InputError::NoTargets { .. }));
    }

    #[test]
    fn load_targets_reports_unsupported_single_url_scheme() {
        let err = load_targets("ftp://example.com").expect_err("invalid URL error");

        assert!(matches!(
            err,
            InputError::InvalidTargetUrl { line: None, .. }
        ));
    }
}
