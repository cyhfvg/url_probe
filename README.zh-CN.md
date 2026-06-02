# url_probe

[English](README.md) | [简体中文](README.zh-CN.md)

`url_probe` 是一个用于检查 HTTP/HTTPS 端点的轻量命令行工具，可记录响应状态码、
下载大小和 HTML 页面标题。它适合用于核查已知 Web 资产、验证部署结果，以及处理
经过授权的端点清单。

> **仅限授权使用：** 只可对您拥有或已获得明确测试授权的系统运行 `url_probe`。
> 请遵守适用的政策、法律规定和流量限制。

## 功能特点

- 接收单个 URL、URL 列表文件，或从标准输入读取 URL。
- 支持 `GET` 和 `HEAD` 请求，可设置并发数、请求抖动、超时、重试、重定向和 User-Agent。
- 使用单一代理 URL 支持带认证的 HTTP、HTTPS、SOCKS5 和 SOCKS5H 代理。
- 可按 HTTP 状态码或下载响应大小筛选结果。
- 支持将 CSV 或 JSON Lines 结果输出到标准输出或指定文件。

## 安装

可从本仓库的 GitHub Releases 页面下载预编译压缩包：

- Linux：`x86_64-unknown-linux-musl`
- Windows：`x86_64-pc-windows-gnu`

从源码构建时，请先安装 Rust 工具链，然后运行：

```bash
cargo build --release
```

生成的可执行文件位于 `target/release/`（Windows 下为 `url_probe.exe`）。

## 快速开始

探测一个已授权端点：

```bash
url_probe --target https://example.com --insecure false
```

探测 URL 清单，并保存为 JSON Lines：

```bash
url_probe --target urls.txt --format jsonl --output results.jsonl --insecure false
```

从标准输入读取目标：

```bash
printf '%s\n' 'https://example.com' | url_probe --target - --insecure false
```

通过带认证的 SOCKS5H 代理探测：

```bash
url_probe --target https://example.com --proxy 'socks5h://username:password@127.0.0.1:1080' --insecure false
```

目标文件每行填写一个 HTTP 或 HTTPS URL。空行和以 `#` 开头的行会被忽略。无效
URL 行会报告具体行号，空目标集会以明确诊断退出。

## 常用选项

| 选项 | 用途 |
| --- | --- |
| `-t, --target <TARGET>` | URL、URL 列表文件，或使用 `-` 读取标准输入 |
| `-o, --output <FILE>` | 将结果写入文件，而不是标准输出 |
| `--format <csv\|jsonl>` | 选择输出格式，默认为 `csv` |
| `--method <get\|head>` | 选择请求方法，默认为 `get` |
| `--concurrency <N>` | 设置并发请求数，默认为 `50` |
| `--request-jitter-ms <MS>` | 每次 HTTP 请求发送前随机等待 `0..=MS` 毫秒，默认为 `0` |
| `--timeout <SECONDS>` | 设置请求超时秒数，默认为 `10` |
| `--retry <N>` | 对失败请求进行重试 |
| `--proxy <URL>` | 通过 `http`、`https`、`socks5` 或 `socks5h` 代理 URL 发起请求 |
| `--filter-http-code <CODES>` | 仅保留逗号分隔的指定状态码 |
| `--black-http-code <CODES>` | 排除逗号分隔的指定状态码 |
| `--black-size <SIZES>` | 排除逗号分隔的指定字节大小 |

运行 `url_probe --help` 可查看完整英文参数说明。

## 代理

请使用 `--proxy` 提供一个具有 scheme 的完整 URL。代理需要认证时，将凭据直接包含
在 URL 中：

```bash
--proxy 'socks5h://username:password@127.0.0.1:1080'
```

工具接受 `http://`、`https://`、`socks5://` 和 `socks5h://` 代理 URL。
`socks5h://` 会由代理端解析目标域名。本工具不提供 `-x` 短参数，也不提供单独的
代理认证覆盖参数。凭据中包含 URL 保留字符时，请进行百分号编码。

## 安全提示

当前版本为了便于检查使用非公开证书的环境，默认接受无效的 HTTPS 证书。在证书
验证有意义的日常检查中，请传入 `--insecure false`。

请为已授权目标选择合适的并发数，避免产生不必要的负载。`--request-jitter-ms`
有助于降低短时突发压力，但不能替代授权、保守的并发设置和约定好的测试窗口。

代理 URL 可能包含敏感凭据，请避免在 shell 历史记录、日志或共享进程查看环境中
暴露这些内容。

## 输出结果

CSV 输出包含表头和 `url`、`http_code`、`size_download`、`webtitle`、`error_kind`、
`error` 六列。JSON Lines 输出为每个结果写入一个具有对应字段的对象。`error_kind`
是稳定的错误分类，例如 `timeout`、`connect`、`redirect`、`body` 或 `request`，
比面向人类阅读的 `error` 文本更适合批量统计。

## 性能测试

运行 Criterion benchmark 套件：

```bash
cargo bench
```

该套件覆盖请求抖动计算、标题提取、客户端构建、过滤、CSV/JSON Lines 输出和 URL
列表加载。

## 项目状态

已实现功能与已知限制请见 [docs/PROGRESS.md](docs/PROGRESS.md)，后续计划请见
[docs/TODO.md](docs/TODO.md)。

## 许可证

本项目依据 [BSD 3-Clause License](LICENSE) 分发。
