use std::fmt::Display;

#[derive(Debug)]
pub enum CliError {
    Input(InputError),
    Output(OutputError),
    Request(RequestError),
    Args(String),
}

impl Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CliError::Input(err) => write!(f, "Input error: {}", err),
            CliError::Output(err) => write!(f, "Output error: {}", err),
            CliError::Request(err) => write!(f, "Request error: {}", err),
            CliError::Args(err) => write!(f, "Arguments error: {}", err),
        }
    }
}

impl From<InputError> for CliError {
    fn from(err: InputError) -> Self {
        CliError::Input(err)
    }
}

impl From<OutputError> for CliError {
    fn from(err: OutputError) -> Self {
        CliError::Output(err)
    }
}

impl From<RequestError> for CliError {
    fn from(err: RequestError) -> Self {
        CliError::Request(err)
    }
}

#[derive(Debug)]
pub enum InputError {
    OpenTargetFileError {
        path: String,
        source: std::io::Error,
    },
    ReadTargetLine {
        source: std::io::Error,
    },
    InvalidTargetUrl {
        value: String,
        line: Option<usize>,
        source: String,
    },
    NoTargets {
        source: String,
    },
}

impl Display for InputError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InputError::OpenTargetFileError { path, source } => {
                write!(f, "failed to open target file: {}, error: {}", path, source)
            }
            InputError::ReadTargetLine { source } => {
                write!(f, "failed to read target line: {}", source)
            }
            InputError::InvalidTargetUrl {
                value,
                line,
                source,
            } => {
                if let Some(line) = line {
                    write!(
                        f,
                        "invalid target URL (line {}): {}, reason: {}",
                        line, value, source
                    )
                } else {
                    write!(f, "invalid target URL: {}, reason: {}", value, source)
                }
            }
            InputError::NoTargets { source } => {
                write!(f, "no probe targets found: {}", source)
            }
        }
    }
}

#[derive(Debug)]
pub enum OutputError {
    Write {
        source: std::io::Error,
    },
    CreateFile {
        path: String,
        source: std::io::Error,
    },
    CsvWrite {
        source: csv::Error,
    },
    JsonlWrite {
        source: serde_json::Error,
    },
}

impl Display for OutputError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputError::Write { source } => {
                write!(f, "failed to write output: {}", source)
            }
            OutputError::CreateFile { path, source } => {
                write!(
                    f,
                    "failed to create output file: {}, error: {}",
                    path, source
                )
            }
            OutputError::CsvWrite { source } => {
                write!(f, "CSV write error: {}", source)
            }
            OutputError::JsonlWrite { source } => {
                write!(f, "JSONL write error: {}", source)
            }
        }
    }
}

#[derive(Debug)]
pub enum RequestError {
    InvalidProxyUrl { source: url::ParseError },
    UnsupportedProxyScheme { scheme: String },
    ConfigureProxy { source: reqwest::Error },
    BuildClientFailed { source: reqwest::Error },
}

impl Display for RequestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RequestError::InvalidProxyUrl { source } => {
                write!(f, "proxy must be a valid scheme URL: {}", source)
            }
            RequestError::UnsupportedProxyScheme { scheme } => {
                write!(f, "unsupported proxy URL scheme: {}", scheme)
            }
            RequestError::ConfigureProxy { source } => {
                write!(f, "failed to configure proxy: {}", source)
            }
            RequestError::BuildClientFailed { source } => {
                write!(f, "failed to build HTTP client: {}", source)
            }
        }
    }
}
