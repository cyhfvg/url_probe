use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

struct TestServer {
    addr: SocketAddr,
    shutdown: Arc<AtomicBool>,
    counts: Arc<Mutex<HashMap<String, usize>>>,
    methods: Arc<Mutex<Vec<(String, String)>>>,
    handle: Option<JoinHandle<()>>,
}

impl TestServer {
    fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind local test server");
        listener
            .set_nonblocking(true)
            .expect("set nonblocking listener");
        let addr = listener.local_addr().expect("local server address");
        let shutdown = Arc::new(AtomicBool::new(false));
        let counts = Arc::new(Mutex::new(HashMap::<String, usize>::new()));
        let methods = Arc::new(Mutex::new(Vec::<(String, String)>::new()));

        let thread_shutdown = Arc::clone(&shutdown);
        let thread_counts = Arc::clone(&counts);
        let thread_methods = Arc::clone(&methods);
        let thread_addr = addr;
        let handle = thread::spawn(move || {
            while !thread_shutdown.load(Ordering::Relaxed) {
                match listener.accept() {
                    Ok((stream, _)) => {
                        let counts = Arc::clone(&thread_counts);
                        let methods = Arc::clone(&thread_methods);
                        thread::spawn(move || {
                            handle_connection(stream, thread_addr, &counts, &methods)
                        });
                    }
                    Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(5));
                    }
                    Err(_) => break,
                }
            }
        });

        Self {
            addr,
            shutdown,
            counts,
            methods,
            handle: Some(handle),
        }
    }

    fn url(&self, path: &str) -> String {
        format!("http://{}{}", self.addr, path)
    }

    fn request_count(&self, path: &str) -> usize {
        self.counts
            .lock()
            .expect("counts lock")
            .get(path)
            .copied()
            .unwrap_or(0)
    }

    fn saw_method(&self, method: &str, path: &str) -> bool {
        self.methods
            .lock()
            .expect("methods lock")
            .iter()
            .any(|(seen_method, seen_path)| seen_method == method && seen_path == path)
    }
}

impl Drop for TestServer {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::Relaxed);
        let _ = TcpStream::connect(self.addr);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

struct TempFile {
    path: PathBuf,
}

impl TempFile {
    fn new(name: &str, contents: &str) -> Self {
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time after epoch")
            .as_millis();
        let path = std::env::temp_dir().join(format!(
            "url_probe_test_{}_{}_{}",
            std::process::id(),
            millis,
            name
        ));
        fs::write(&path, contents).expect("write temp file");
        Self { path }
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

fn handle_connection(
    mut stream: TcpStream,
    addr: SocketAddr,
    counts: &Arc<Mutex<HashMap<String, usize>>>,
    methods: &Arc<Mutex<Vec<(String, String)>>>,
) {
    let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
    let Some((method, path)) = read_request_line(&mut stream) else {
        return;
    };

    *counts
        .lock()
        .expect("counts lock")
        .entry(path.clone())
        .or_insert(0) += 1;
    methods
        .lock()
        .expect("methods lock")
        .push((method.clone(), path.clone()));

    match path.as_str() {
        "/get-title" => respond(
            &mut stream,
            200,
            &[("Content-Type", "text/html; charset=utf-8")],
            b"<!doctype html><title>Local Test Title</title><h1>Hello</h1>",
        ),
        "/head" => respond(&mut stream, 204, &[], b""),
        "/json" => respond(
            &mut stream,
            200,
            &[("Content-Type", "text/html")],
            b"<title>Json Title</title>",
        ),
        "/redirect" => respond(
            &mut stream,
            302,
            &[("Location", &format!("http://{addr}/get-title"))],
            b"",
        ),
        "/status/404" => respond(&mut stream, 404, &[], b"missing"),
        "/bytes/5" => respond(&mut stream, 200, &[], b"12345"),
        "/bytes/13" => respond(&mut stream, 200, &[], b"1234567890123"),
        "/slow" => {
            thread::sleep(Duration::from_millis(1_500));
            respond(&mut stream, 200, &[], b"slow");
        }
        "/close" => {}
        _ => respond(&mut stream, 404, &[], b"unknown"),
    }
}

fn read_request_line(stream: &mut TcpStream) -> Option<(String, String)> {
    let mut buffer = [0_u8; 1024];
    let mut request = Vec::new();
    loop {
        let read = stream.read(&mut buffer).ok()?;
        if read == 0 {
            return None;
        }
        request.extend_from_slice(&buffer[..read]);
        if request.windows(4).any(|window| window == b"\r\n\r\n") {
            break;
        }
    }

    let request = String::from_utf8_lossy(&request);
    let first_line = request.lines().next()?;
    let mut parts = first_line.split_whitespace();
    let method = parts.next()?.to_string();
    let path = parts.next()?.to_string();
    Some((method, path))
}

fn respond(stream: &mut TcpStream, status: u16, headers: &[(&str, &str)], body: &[u8]) {
    let reason = match status {
        200 => "OK",
        204 => "No Content",
        302 => "Found",
        404 => "Not Found",
        _ => "OK",
    };
    let mut response = format!(
        "HTTP/1.1 {} {}\r\nContent-Length: {}\r\n",
        status,
        reason,
        body.len()
    );
    for (name, value) in headers {
        response.push_str(name);
        response.push_str(": ");
        response.push_str(value);
        response.push_str("\r\n");
    }
    response.push_str("\r\n");

    stream
        .write_all(response.as_bytes())
        .expect("write response headers");
    if !body.is_empty() {
        stream.write_all(body).expect("write response body");
    }
}

fn run_url_probe(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_url_probe"))
        .args(args)
        .output()
        .expect("run url_probe")
}

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "expected success\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn stdout_string(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout is UTF-8")
}

fn csv_rows(stdout: &str) -> Vec<csv::StringRecord> {
    csv::Reader::from_reader(stdout.as_bytes())
        .records()
        .collect::<Result<Vec<_>, _>>()
        .expect("parse CSV output")
}

fn path_str(path: &Path) -> &str {
    path.to_str().expect("temp path is UTF-8")
}

#[test]
fn get_request_extracts_html_title_and_csv_output() {
    let server = TestServer::start();

    let output = run_url_probe(&["--target", &server.url("/get-title"), "--concurrency", "1"]);
    assert_success(&output);

    let stdout = stdout_string(&output);
    assert!(stdout.starts_with("url,http_code,size_download,webtitle,error\n"));
    let rows = csv_rows(&stdout);
    assert_eq!(rows.len(), 1);
    assert_eq!(&rows[0][1], "200");
    assert_eq!(&rows[0][2], "60");
    assert_eq!(&rows[0][3], "Local Test Title");
    assert_eq!(&rows[0][4], "");
    assert!(server.saw_method("GET", "/get-title"));
}

#[test]
fn head_request_does_not_download_body() {
    let server = TestServer::start();

    let output = run_url_probe(&[
        "--target",
        &server.url("/head"),
        "--method",
        "head",
        "--concurrency",
        "1",
    ]);
    assert_success(&output);

    let stdout = stdout_string(&output);
    let rows = csv_rows(&stdout);
    assert_eq!(rows.len(), 1);
    assert_eq!(&rows[0][1], "204");
    assert_eq!(&rows[0][2], "");
    assert_eq!(&rows[0][3], "");
    assert!(server.saw_method("HEAD", "/head"));
}

#[test]
fn jsonl_output_serializes_probe_result() {
    let server = TestServer::start();

    let output = run_url_probe(&[
        "--target",
        &server.url("/json"),
        "--format",
        "jsonl",
        "--concurrency",
        "1",
    ]);
    assert_success(&output);

    let stdout = stdout_string(&output);
    let record: Value = serde_json::from_str(stdout.trim()).expect("parse JSONL record");
    assert_eq!(record["http_code"], 200);
    assert_eq!(record["size_download"], 25);
    assert_eq!(record["webtitle"], "Json Title");
    assert!(record["error"].is_null());
}

#[test]
fn filters_by_status_and_blacklisted_size() {
    let server = TestServer::start();
    let targets = TempFile::new(
        "targets.txt",
        &format!(
            "{}\n{}\n{}\n",
            server.url("/bytes/5"),
            server.url("/bytes/13"),
            server.url("/status/404")
        ),
    );

    let output = run_url_probe(&[
        "--target",
        path_str(&targets.path),
        "--filter-http-code",
        "200",
        "--black-size",
        "13",
        "--concurrency",
        "1",
    ]);
    assert_success(&output);

    let stdout = stdout_string(&output);
    let rows = csv_rows(&stdout);
    assert_eq!(rows.len(), 1);
    assert!(rows[0][0].ends_with("/bytes/5"));
    assert_eq!(&rows[0][1], "200");
    assert_eq!(&rows[0][2], "5");
}

#[test]
fn follows_redirects_when_enabled_and_reports_redirect_when_disabled() {
    let server = TestServer::start();

    let followed = run_url_probe(&[
        "--target",
        &server.url("/redirect"),
        "--follow-redirect",
        "true",
        "--concurrency",
        "1",
    ]);
    assert_success(&followed);
    let followed_rows = csv_rows(&stdout_string(&followed));
    assert_eq!(&followed_rows[0][1], "200");
    assert_eq!(&followed_rows[0][3], "Local Test Title");

    let not_followed = run_url_probe(&[
        "--target",
        &server.url("/redirect"),
        "--follow-redirect",
        "false",
        "--concurrency",
        "1",
    ]);
    assert_success(&not_followed);
    let not_followed_rows = csv_rows(&stdout_string(&not_followed));
    assert_eq!(&not_followed_rows[0][1], "302");
    assert_eq!(&not_followed_rows[0][3], "");
}

#[test]
fn timeout_failures_are_retried() {
    let server = TestServer::start();

    let output = run_url_probe(&[
        "--target",
        &server.url("/slow"),
        "--timeout",
        "1",
        "--retry",
        "1",
        "--concurrency",
        "1",
    ]);
    assert_success(&output);

    let stdout = stdout_string(&output);
    let rows = csv_rows(&stdout);
    assert_eq!(rows.len(), 1);
    assert_eq!(&rows[0][1], "");
    assert!(rows[0][4].contains("Request error:"));
    assert_eq!(server.request_count("/slow"), 2);
}

#[test]
fn output_with_error_false_suppresses_failed_results() {
    let server = TestServer::start();

    let output = run_url_probe(&[
        "--target",
        &server.url("/close"),
        "--output-with-error",
        "false",
        "--concurrency",
        "1",
    ]);
    assert_success(&output);

    let stdout = stdout_string(&output);
    assert_eq!(stdout, "url,http_code,size_download,webtitle,error\n");
}

#[test]
fn cli_help_is_english_and_lists_key_defaults() {
    let output = run_url_probe(&["--help"]);
    assert_success(&output);

    let stdout = stdout_string(&output);
    assert!(stdout.contains("Probe HTTP status codes and downloaded sizes for URLs"));
    assert!(stdout.contains("Target URL, file containing URLs, or `-` to read from stdin"));
    assert!(stdout.contains("Maximum concurrent requests (minimum: 1) [default: 50]"));
    assert!(stdout.contains("Request timeout in seconds [default: 10]"));
    assert!(stdout.contains("Number of retries after a failed request [default: 0]"));
    assert!(
        stdout
            .contains("Maximum random delay in milliseconds before each HTTP request [default: 0]")
    );
    assert!(stdout.contains("HTTP request method [default: get] [possible values: get, head]"));
    assert!(stdout.contains("Output format [default: csv] [possible values: csv, jsonl]"));
}
