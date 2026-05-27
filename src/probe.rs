use crate::cli::Args;
use crate::{cli::HttpMethod, error::RequestError};
use reqwest::{Client, header};
use scraper::{Html, Selector};
use std::time::Duration;
use url::Url;

#[derive(Debug, Clone)]
pub struct ProbeResult {
    pub url: Url,
    pub http_code: Option<u16>,
    pub size_download: Option<u64>,
    pub webtitle: Option<String>,
    pub error: Option<String>,
}

fn build_proxy(proxy_url: &str) -> Result<reqwest::Proxy, RequestError> {
    let parsed_url =
        Url::parse(proxy_url).map_err(|source| RequestError::InvalidProxyUrl { source })?;
    if !matches!(parsed_url.scheme(), "http" | "https" | "socks5" | "socks5h") {
        return Err(RequestError::UnsupportedProxyScheme {
            scheme: parsed_url.scheme().to_string(),
        });
    }

    reqwest::Proxy::all(parsed_url).map_err(|source| RequestError::ConfigureProxy { source })
}

/// 构建reqwest客户端，根据命令行参数设置用户代理、超时、重定向策略等
pub fn build_reqwest_client(args: &Args) -> Result<Client, RequestError> {
    let redirect_policy = if args.follow_redirect {
        reqwest::redirect::Policy::limited(10)
    } else {
        reqwest::redirect::Policy::none()
    };

    let mut client_builder = Client::builder()
        .user_agent(&args.user_agent)
        .timeout(Duration::from_secs(args.timeout))
        .danger_accept_invalid_certs(args.insecure)
        .redirect(redirect_policy);

    if let Some(proxy_url) = args.proxy.as_deref() {
        client_builder = client_builder.proxy(build_proxy(proxy_url)?);
    }

    client_builder
        .build()
        .map_err(|why| RequestError::BuildClientFailed { source: why })
}

/// 从HTML内容中提取<title>标签的内容，作为webtitle
fn extract_title(body: &[u8]) -> Option<String> {
    let html = std::str::from_utf8(body).ok()?;
    let document = Html::parse_document(html);
    let selector = Selector::parse("title").ok()?;
    let title = document.select(&selector).next()?;
    Some(title.text().collect::<String>().trim().to_string())
}

/// 对单个URL进行探测，返回ProbeResult
pub async fn probe_once(client: &Client, url: Url, method: HttpMethod) -> ProbeResult {
    let url_for_result = url.clone();
    let request = match method {
        HttpMethod::Get => client.get(url.clone()),
        HttpMethod::Head => client.head(url.clone()),
        // HttpMethod::Post => client.post(url.clone()),
    };

    let response = match request.send().await {
        Ok(resp) => resp,
        Err(e) => {
            return ProbeResult {
                url: url_for_result,
                http_code: None,
                size_download: None,
                webtitle: None,
                error: Some(format!("Request error: {}", e)),
            };
        }
    };

    let status = response.status().as_u16();

    if matches!(method, HttpMethod::Head) {
        // HEAD请求不下载内容，直接返回结果
        return ProbeResult {
            url: url_for_result,
            http_code: Some(status),
            size_download: None,
            webtitle: None,
            error: None,
        };
    }

    let content_type = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok());

    let is_html = content_type
        .map(|ct| ct.to_lowercase().contains("text/html"))
        .unwrap_or(false);

    let body = match response.bytes().await {
        Ok(bytes) => bytes,
        Err(e) => {
            return ProbeResult {
                url: url_for_result,
                http_code: Some(status),
                size_download: None,
                webtitle: None,
                error: Some(format!("Read body error: {}", e)),
            };
        }
    };

    let size_download = body.len() as u64;

    let webtitle = if is_html { extract_title(&body) } else { None };

    ProbeResult {
        url: url_for_result,
        http_code: Some(status),
        size_download: Some(size_download),
        webtitle,
        error: None,
    }
}

/// 对单个URL进行探测，失败时重试指定次数，返回最终的ProbeResult
pub async fn probe_once_with_retry(
    client: &Client,
    url: Url,
    method: HttpMethod,
    retries: usize,
) -> ProbeResult {
    let mut last_result = probe_once(client, url.clone(), method).await;
    if last_result.error.is_none() {
        return last_result;
    }

    for _ in 0..retries {
        let result = probe_once(client, url.clone(), method).await;
        if result.error.is_none() {
            return result;
        }
        last_result = result;
    }
    last_result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_proxy_url_with_socks5h_authentication() {
        let proxy =
            build_proxy("socks5h://username:password@127.0.0.1:1080").expect("valid proxy URL");

        assert!(Client::builder().proxy(proxy).build().is_ok());
    }

    #[test]
    fn accepts_authenticated_http_proxy_url() {
        let proxy =
            build_proxy("http://username:password@127.0.0.1:8080").expect("valid proxy URL");

        assert!(Client::builder().proxy(proxy).build().is_ok());
    }

    #[test]
    fn accepts_authenticated_https_proxy_url() {
        let proxy =
            build_proxy("https://username:password@127.0.0.1:8443").expect("valid proxy URL");

        assert!(Client::builder().proxy(proxy).build().is_ok());
    }

    #[test]
    fn accepts_authenticated_socks5_proxy_url() {
        let proxy =
            build_proxy("socks5://username:password@127.0.0.1:1080").expect("valid proxy URL");

        assert!(Client::builder().proxy(proxy).build().is_ok());
    }

    #[test]
    fn rejects_proxy_value_without_scheme() {
        assert!(matches!(
            build_proxy("127.0.0.1:1080"),
            Err(RequestError::InvalidProxyUrl { .. })
        ));
    }

    #[test]
    fn rejects_unsupported_proxy_scheme() {
        assert!(matches!(
            build_proxy("ftp://127.0.0.1:21"),
            Err(RequestError::UnsupportedProxyScheme { .. })
        ));
    }
}
