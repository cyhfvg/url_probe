use clap::{ArgAction, Parser, ValueEnum};

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum HttpMethod {
    Get,
    Head,
    // Post,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum OutputFormat {
    Csv,
    Jsonl,
}

#[derive(Debug, Parser)]
#[command(
    name = "url_probe",
    version,
    about = "Probe HTTP status codes and downloaded sizes for URLs"
)]
pub struct Args {
    /// Target URL, file containing URLs, or `-` to read from stdin
    #[arg(short = 't', long = "target")]
    pub target: String,

    /// Only output these HTTP status codes (comma-separated)
    #[arg(long = "filter-http-code", value_delimiter = ',')]
    pub filter_http_code: Vec<u16>,

    /// Exclude these HTTP status codes (comma-separated)
    #[arg(long = "black-http-code", value_delimiter = ',')]
    pub black_http_code: Vec<u16>,

    /// Exclude these downloaded sizes in bytes (comma-separated)
    #[arg(long = "black-size", value_delimiter = ',')]
    pub black_size: Vec<u64>,

    /// Maximum concurrent requests (minimum: 1)
    #[arg(long = "concurrency", default_value_t = 50)]
    pub concurrency: usize,

    /// Request timeout in seconds
    #[arg(long = "timeout", default_value_t = 10)]
    pub timeout: u64,

    /// Number of retries after a failed request
    #[arg(long = "retry", default_value_t = 0)]
    pub retry: usize,

    /// Output file path (defaults to stdout)
    #[arg(short = 'o', long = "output")]
    pub output: Option<String>,

    /// HTTP request method
    #[arg(long = "method", value_enum, default_value_t = HttpMethod::Get)]
    pub method: HttpMethod,

    /// User-Agent header value
    #[arg(
        long = "user-agent",
        default_value = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"
    )]
    pub user_agent: String,

    /// Proxy URL for all requests (http, https, socks5, or socks5h; may include authentication)
    #[arg(long = "proxy")]
    pub proxy: Option<String>,

    /// Follow HTTP redirects (up to 10 redirects)
    #[arg(long = "follow-redirect", default_value_t = true, action = ArgAction::Set)]
    pub follow_redirect: bool,

    /// Accept invalid HTTPS certificates
    #[arg(long = "insecure", default_value_t = true, action = ArgAction::Set)]
    pub insecure: bool,

    /// Include failed requests in output
    #[arg(long = "output-with-error", default_value_t = true, action = ArgAction::Set)]
    pub output_with_error: bool,

    /// Output format
    #[arg(long = "format", value_enum, default_value_t = OutputFormat::Csv)]
    pub format: OutputFormat,
}
