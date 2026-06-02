# url_probe

[English](README.md) | [简体中文](README.zh-CN.md)

`url_probe` is a small command-line tool for checking HTTP/HTTPS endpoints and
recording response status, downloaded size, and HTML page title. It is useful
for reviewing known web assets, validating deployments, and processing an
authorized list of endpoints.

> **Authorized use only:** Run `url_probe` only against systems you own or have
> explicit permission to assess. Respect applicable policies, laws, and
> traffic limits.

## Features

- Accepts a single URL, a URL list file, or URLs piped through standard input.
- Supports `GET` and `HEAD` requests with configurable concurrency, timeout,
  retries, redirects, per-request jitter, and user agent.
- Supports authenticated HTTP, HTTPS, SOCKS5, and SOCKS5H proxies through a
  single proxy URL.
- Filters results by HTTP status code or downloaded response size.
- Writes machine-readable CSV or JSON Lines output to stdout or a file.

## Installation

Download a prebuilt archive from this repository's GitHub Releases page:

- Linux: `x86_64-unknown-linux-musl`
- Windows: `x86_64-pc-windows-gnu`

To build from source, install the Rust toolchain and run:

```bash
cargo build --release
```

The compiled executable is placed in `target/release/` (`url_probe.exe` on
Windows).

## Quick Start

Probe one authorized endpoint:

```bash
url_probe --target https://example.com --insecure false
```

Probe a URL list and save JSON Lines output:

```bash
url_probe --target urls.txt --format jsonl --output results.jsonl --insecure false
```

Read targets from stdin:

```bash
printf '%s\n' 'https://example.com' | url_probe --target - --insecure false
```

Probe through an authenticated SOCKS5H proxy:

```bash
url_probe --target https://example.com --proxy 'socks5h://username:password@127.0.0.1:1080' --insecure false
```

A target file contains one HTTP or HTTPS URL per line. Empty lines and lines
beginning with `#` are ignored.

## Common Options

| Option | Purpose |
| --- | --- |
| `-t, --target <TARGET>` | URL, URL list file, or `-` for stdin |
| `-o, --output <FILE>` | Write output to a file instead of stdout |
| `--format <csv\|jsonl>` | Choose output format; default is `csv` |
| `--method <get\|head>` | Choose request method; default is `get` |
| `--concurrency <N>` | Set concurrent requests; default is `50` |
| `--request-jitter-ms <MS>` | Wait a random `0..=MS` milliseconds before each HTTP request; default is `0` |
| `--timeout <SECONDS>` | Set request timeout; default is `10` |
| `--retry <N>` | Retry failed requests |
| `--proxy <URL>` | Route requests through an `http`, `https`, `socks5`, or `socks5h` proxy URL |
| `--filter-http-code <CODES>` | Only include comma-separated status codes |
| `--black-http-code <CODES>` | Exclude comma-separated status codes |
| `--black-size <SIZES>` | Exclude comma-separated byte sizes |

Run `url_probe --help` for the complete option list.

## Proxy

Provide the proxy as one scheme URL with `--proxy`. When authentication is
needed, include it in that URL:

```bash
--proxy 'socks5h://username:password@127.0.0.1:1080'
```

The tool accepts `http://`, `https://`, `socks5://`, and `socks5h://` proxy
URLs. `socks5h://` resolves destination hostnames through the proxy. There is
no `-x` short option or separate proxy credential override option. Percent-encode
reserved URL characters used within credentials.

## Security Notes

The current version accepts invalid HTTPS certificates by default for probing
environments with non-public certificates. For routine checks where certificate
validation matters, pass `--insecure false`.

Use a concurrency value appropriate for the authorized target and avoid causing
unnecessary load. `--request-jitter-ms` can reduce short burst pressure, but it
does not replace authorization, conservative concurrency, or an agreed testing
window.

Proxy URLs may contain secrets. Take care not to expose them through shell
history, logs, or shared process inspection.

## Output

CSV output includes a header and the columns `url`, `http_code`,
`size_download`, `webtitle`, and `error`. JSON Lines output writes one object
per result with equivalent fields.

## Benchmarks

Run the Criterion benchmark suite with:

```bash
cargo bench
```

The suite covers request jitter calculation, title extraction, client building,
filtering, CSV/JSON Lines output, and URL list loading.

## Project Status

See [docs/PROGRESS.md](docs/PROGRESS.md) for implemented behavior and known
limitations, and [docs/TODO.md](docs/TODO.md) for planned improvements.

## License

This project is distributed under the [BSD 3-Clause License](LICENSE).
