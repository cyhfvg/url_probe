use crate::cli::OutputFormat;
use crate::error::OutputError;
use crate::probe::ProbeResult;
use std::fs::File;
use std::io;
use std::io::BufWriter;

const CSV_HEADER: [&str; 5] = ["url", "http_code", "size_download", "webtitle", "error"];

#[derive(serde::Serialize)]
struct JsonlRecord {
    url: String,
    http_code: Option<u16>,
    size_download: Option<u64>,
    webtitle: Option<String>,
    error: Option<String>,
}

/// 将ProbeResult转换为JsonLRecord，方便后续序列化为JSONL格式
fn to_jsonl_record(res: &ProbeResult) -> JsonlRecord {
    JsonlRecord {
        url: res.url.to_string(),
        http_code: res.http_code,
        size_download: res.size_download,
        webtitle: res.webtitle.clone(),
        error: res.error.clone(),
    }
}

pub enum OutputWriter {
    Csv(Box<csv::Writer<Box<dyn io::Write>>>),
    Jsonl(Box<dyn io::Write>),
}

/// 构建输出写入器，根据用户指定的格式和路径创建相应的输出对象
pub fn build_output_writer(
    output: Option<&str>,
    format: OutputFormat,
) -> Result<OutputWriter, OutputError> {
    match format {
        OutputFormat::Csv => {
            let writer = create_csv_writer(output)?;
            Ok(OutputWriter::Csv(Box::new(writer)))
        }
        OutputFormat::Jsonl => {
            let writer: Box<dyn io::Write> = create_jsonl_writer(output)?;
            Ok(OutputWriter::Jsonl(writer))
        }
    }
}

/// 写入输出文件的header
pub fn write_header(writer: &mut OutputWriter) -> Result<(), OutputError> {
    match writer {
        OutputWriter::Csv(csv_writer) => write_csv_header(csv_writer.as_mut()),
        OutputWriter::Jsonl(_) => Ok(()),
    }
}

/// 将ProbeResult记录写入输出文件
pub fn write_result(writer: &mut OutputWriter, res: &ProbeResult) -> Result<(), OutputError> {
    match writer {
        OutputWriter::Csv(csv_writer) => write_csv_record(csv_writer.as_mut(), res),
        OutputWriter::Jsonl(jsonl_writer) => write_jsonl_line(jsonl_writer, res),
    }
}

/// 结束输出，确保所有数据都被写入文件
pub fn finish_output(writer: &mut OutputWriter) -> Result<(), OutputError> {
    match writer {
        OutputWriter::Csv(csv_writer) => csv_writer
            .flush()
            .map_err(|why| OutputError::Write { source: why }),
        OutputWriter::Jsonl(jsonl_writer) => jsonl_writer
            .flush()
            .map_err(|why| OutputError::Write { source: why }),
    }
}

/// 创建CSV写入器，根据输出路径决定是写入文件还是标准输出
pub fn create_csv_writer(
    output_path: Option<&str>,
) -> Result<csv::Writer<Box<dyn io::Write>>, OutputError> {
    let writer: Box<dyn io::Write> = match output_path {
        Some(path) => {
            let file = File::create(path).map_err(|why| OutputError::CreateFile {
                path: path.to_string(),
                source: why,
            })?;
            Box::new(BufWriter::new(file))
        }
        None => Box::new(BufWriter::new(io::stdout().lock())),
    };
    Ok(csv::Writer::from_writer(writer))
}

/// 写入CSV文件的header
pub fn write_csv_header<W: io::Write>(writer: &mut csv::Writer<W>) -> Result<(), OutputError> {
    writer
        .write_record(CSV_HEADER)
        .map_err(|why| OutputError::CsvWrite { source: why })
}

/// 将ProbeResult记录写入CSV
pub fn write_csv_record<W: io::Write>(
    writer: &mut csv::Writer<W>,
    res: &ProbeResult,
) -> Result<(), OutputError> {
    let url = res.url.to_string();
    let http_code = res.http_code.map(|v| v.to_string()).unwrap_or_default();
    let size_download = res.size_download.map(|v| v.to_string()).unwrap_or_default();
    let webtitle = res.webtitle.clone().unwrap_or_default();
    let error = res.error.clone().unwrap_or_default();

    writer
        .write_record([
            url.as_str(),
            http_code.as_str(),
            size_download.as_str(),
            webtitle.as_str(),
            error.as_str(),
        ])
        .map_err(|why| OutputError::CsvWrite { source: why })
}

/// 创建JSONL写入器，根据输出路径决定是写入文件还是标准输出
pub fn create_jsonl_writer(output_path: Option<&str>) -> Result<Box<dyn io::Write>, OutputError> {
    let writer: Box<dyn io::Write> = match output_path {
        Some(path) => {
            let file = File::create(path).map_err(|why| OutputError::CreateFile {
                path: path.to_string(),
                source: why,
            })?;
            Box::new(BufWriter::new(file))
        }
        None => Box::new(BufWriter::new(io::stdout().lock())),
    };
    Ok(writer)
}

/// 将ProbeResult写入JSONL文件，每行一个JSON对象
pub fn write_jsonl_line(writer: &mut dyn io::Write, res: &ProbeResult) -> Result<(), OutputError> {
    let jsonl_record = to_jsonl_record(res);
    serde_json::to_writer(&mut *writer, &jsonl_record)
        .map_err(|why| OutputError::JsonlWrite { source: why })?;

    writer
        .write_all(b"\n")
        .map_err(|why| OutputError::Write { source: why })
}
