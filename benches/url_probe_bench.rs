use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use std::fs;
use std::hint::black_box;
use std::path::PathBuf;
use url::Url;
use url_probe::cli::{Args, HttpMethod, OutputFormat};
use url_probe::filter::should_output;
use url_probe::input::load_targets;
use url_probe::output::{write_csv_header, write_csv_record, write_jsonl_line};
use url_probe::probe::{ProbeResult, build_reqwest_client, extract_title, request_jitter_delay};

struct UrlFixture {
    path: PathBuf,
}

impl Drop for UrlFixture {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

fn sample_args() -> Args {
    Args {
        target: "https://example.com".to_string(),
        filter_http_code: vec![200, 204],
        black_http_code: vec![404, 500],
        black_size: vec![0, 13],
        concurrency: 50,
        timeout: 10,
        retry: 1,
        request_jitter_ms: 0,
        output: None,
        method: HttpMethod::Get,
        user_agent: "url_probe-bench/0.1".to_string(),
        proxy: None,
        follow_redirect: true,
        insecure: true,
        output_with_error: true,
        format: OutputFormat::Csv,
    }
}

fn sample_probe_result() -> ProbeResult {
    ProbeResult {
        url: Url::parse("https://example.com/index.html").expect("valid URL"),
        http_code: Some(200),
        size_download: Some(1024),
        webtitle: Some("Example Domain".to_string()),
        error: None,
    }
}

fn write_url_fixture(count: usize) -> UrlFixture {
    let path = std::env::temp_dir().join(format!(
        "url_probe_bench_{}_{}_urls.txt",
        std::process::id(),
        count
    ));
    let mut body = String::with_capacity(count * 32);
    for index in 0..count {
        body.push_str("https://example.com/path/");
        body.push_str(&index.to_string());
        body.push('\n');
    }
    fs::write(&path, body).expect("write URL fixture");
    UrlFixture { path }
}

fn bench_request_jitter(c: &mut Criterion) {
    let mut group = c.benchmark_group("request_jitter");
    group.bench_function("disabled", |b| {
        b.iter(|| black_box(request_jitter_delay(black_box(0))))
    });
    group.bench_function("bounded_250ms", |b| {
        b.iter(|| black_box(request_jitter_delay(black_box(250))))
    });
    group.finish();
}

fn bench_probe_helpers(c: &mut Criterion) {
    let html = br#"<!doctype html>
<html>
<head><title>Example Domain</title></head>
<body><h1>Example Domain</h1><p>This domain is for examples.</p></body>
</html>"#;
    let args = sample_args();

    let mut group = c.benchmark_group("probe_helpers");
    group.bench_function("extract_html_title", |b| {
        b.iter(|| black_box(extract_title(black_box(html))))
    });
    group.bench_function("build_reqwest_client", |b| {
        b.iter(|| black_box(build_reqwest_client(black_box(&args)).expect("build client")))
    });
    group.finish();
}

fn bench_filtering(c: &mut Criterion) {
    let args = sample_args();
    let result = sample_probe_result();

    c.bench_function("filter_matching_result", |b| {
        b.iter(|| black_box(should_output(black_box(&result), black_box(&args))))
    });
}

fn bench_output(c: &mut Criterion) {
    let result = sample_probe_result();
    let mut group = c.benchmark_group("output");

    group.bench_function("csv_header_and_record", |b| {
        b.iter_batched(
            || csv::Writer::from_writer(Vec::with_capacity(256)),
            |mut writer| {
                write_csv_header(&mut writer).expect("write CSV header");
                write_csv_record(&mut writer, black_box(&result)).expect("write CSV record");
                black_box(writer.into_inner().expect("finish CSV writer"))
            },
            BatchSize::SmallInput,
        )
    });

    group.bench_function("jsonl_record", |b| {
        b.iter_batched(
            || Vec::with_capacity(256),
            |mut writer| {
                write_jsonl_line(&mut writer, black_box(&result)).expect("write JSONL record");
                black_box(writer)
            },
            BatchSize::SmallInput,
        )
    });

    group.finish();
}

fn bench_input(c: &mut Criterion) {
    let fixture = write_url_fixture(1_000);
    let path = fixture.path.to_string_lossy().into_owned();

    c.bench_function("load_targets_file_1000", |b| {
        b.iter(|| black_box(load_targets(black_box(&path)).expect("load targets")))
    });
}

criterion_group!(
    benches,
    bench_request_jitter,
    bench_probe_helpers,
    bench_filtering,
    bench_output,
    bench_input
);
criterion_main!(benches);
