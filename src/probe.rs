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
    pub error_kind: Option<ProbeErrorKind>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProbeErrorKind {
    Timeout,
    Connect,
    Redirect,
    Body,
    Request,
}

impl ProbeErrorKind {
    pub fn as_str(self) -> &'static str {
        match self {
            ProbeErrorKind::Timeout => "timeout",
            ProbeErrorKind::Connect => "connect",
            ProbeErrorKind::Redirect => "redirect",
            ProbeErrorKind::Body => "body",
            ProbeErrorKind::Request => "request",
        }
    }
}

fn classify_request_error(err: &reqwest::Error) -> ProbeErrorKind {
    if err.is_timeout() {
        ProbeErrorKind::Timeout
    } else if err.is_connect() {
        ProbeErrorKind::Connect
    } else if err.is_redirect() {
        ProbeErrorKind::Redirect
    } else if err.is_body() {
        ProbeErrorKind::Body
    } else {
        ProbeErrorKind::Request
    }
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

/// Build a reqwest client from CLI options.
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

/// Extract the HTML <title> text as the webtitle.
pub fn extract_title(body: &[u8]) -> Option<String> {
    let html = std::str::from_utf8(body).ok()?;
    let document = Html::parse_document(html);
    let selector = Selector::parse("title").ok()?;
    let title = document.select(&selector).next()?;
    Some(title.text().collect::<String>().trim().to_string())
}

pub fn request_jitter_delay(max_jitter_ms: u64) -> Option<Duration> {
    if max_jitter_ms == 0 {
        None
    } else {
        Some(Duration::from_millis(fastrand::u64(..=max_jitter_ms)))
    }
}

async fn wait_for_request_jitter(max_jitter_ms: u64) {
    if let Some(delay) = request_jitter_delay(max_jitter_ms) {
        tokio::time::sleep(delay).await;
    }
}

/// Probe one URL and return a ProbeResult.
pub async fn probe_once(
    client: &Client,
    url: Url,
    method: HttpMethod,
    request_jitter_ms: u64,
) -> ProbeResult {
    let url_for_result = url.clone();
    wait_for_request_jitter(request_jitter_ms).await;

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
                error_kind: Some(classify_request_error(&e)),
                error: Some(format!("Request error: {}", e)),
            };
        }
    };

    let status = response.status().as_u16();

    if matches!(method, HttpMethod::Head) {
        // HEAD requests do not download a response body.
        return ProbeResult {
            url: url_for_result,
            http_code: Some(status),
            size_download: None,
            webtitle: None,
            error_kind: None,
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
                error_kind: Some(ProbeErrorKind::Body),
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
        error_kind: None,
        error: None,
    }
}

/// Probe one URL and retry failed requests before returning the final ProbeResult.
pub async fn probe_once_with_retry(
    client: &Client,
    url: Url,
    method: HttpMethod,
    retries: usize,
    request_jitter_ms: u64,
) -> ProbeResult {
    let mut last_result = probe_once(client, url.clone(), method, request_jitter_ms).await;
    if last_result.error.is_none() {
        return last_result;
    }

    for _ in 0..retries {
        let result = probe_once(client, url.clone(), method, request_jitter_ms).await;
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

    #[test]
    fn request_jitter_delay_is_disabled_for_zero() {
        assert_eq!(request_jitter_delay(0), None);
    }

    #[test]
    fn request_jitter_delay_is_bounded_by_maximum() {
        for _ in 0..256 {
            let delay = request_jitter_delay(25).expect("jitter delay");

            assert!(delay <= Duration::from_millis(25));
        }
    }
}
