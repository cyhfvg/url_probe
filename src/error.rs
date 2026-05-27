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
}

impl Display for InputError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InputError::OpenTargetFileError { path, source } => {
                write!(f, "打开目标文件失败: {}, 错误: {}", path, source)
            }
            InputError::ReadTargetLine { source } => {
                write!(f, "读取目标行失败: {}", source)
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
                write!(f, "写入输出失败: {}", source)
            }
            OutputError::CreateFile { path, source } => {
                write!(f, "创建输出文件失败: {}, 错误: {}", path, source)
            }
            OutputError::CsvWrite { source } => {
                write!(f, "CSV写入错误: {}", source)
            }
            OutputError::JsonlWrite { source } => {
                write!(f, "JSONL写入错误: {}", source)
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
                write!(f, "代理必须为有效的 scheme URL: {}", source)
            }
            RequestError::UnsupportedProxyScheme { scheme } => {
                write!(f, "不支持的代理 URL scheme: {}", scheme)
            }
            RequestError::ConfigureProxy { source } => {
                write!(f, "配置代理失败: {}", source)
            }
            RequestError::BuildClientFailed { source } => {
                write!(f, "构建HTTP客户端失败: {}", source)
            }
        }
    }
}
