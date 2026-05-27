use crate::error::InputError;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::PathBuf;
use url::Url;

/// 判断字符串是否是HTTP/HTTPS URL
fn is_http_url(s: &str) -> bool {
    Url::parse(s)
        .map(|url| url.scheme() == "http" || url.scheme() == "https")
        .unwrap_or(false)
}

/// 解析HTTP/HTTPS URL
fn parse_http_url(input: &str) -> Option<Url> {
    let url = Url::parse(input).ok()?;
    if matches!(url.scheme(), "http" | "https") {
        Some(url)
    } else {
        None
    }
}

/// 加载目标URL列表, 支持单个URL, URL列表文件, 或从标准输入读取
pub fn load_targets(file_path: &str) -> Result<Vec<Url>, InputError> {
    let targets = if is_http_url(file_path) {
        vec![parse_http_url(file_path).unwrap()]
    } else {
        let reader: Box<dyn BufRead> = if file_path == "-" {
            Box::new(BufReader::new(io::stdin().lock()))
        } else {
            let path = PathBuf::from(file_path);
            let file = File::open(path).map_err(|why| InputError::OpenTargetFileError {
                path: file_path.to_string(),
                source: why,
            })?;
            Box::new(BufReader::new(file))
        };

        let mut targets = Vec::new();
        for line_result in reader.lines() {
            let mut line = line_result.map_err(|why| InputError::ReadTargetLine { source: why })?;
            line = line.trim().to_string();
            // 跳过空行和注释行
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some(url) = parse_http_url(&line) {
                targets.push(url);
            }
        }
        targets
    };
    Ok(targets)
}
