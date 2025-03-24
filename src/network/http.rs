use std::time::Duration;

use reqwest::{
    Method, Response, StatusCode,
    header::{self, HeaderMap, HeaderName, HeaderValue},
};

pub const BASE_DOMAIN: &str = "gold-usergeneratedcontent.net";

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Status: {0}")]
    Status(StatusCode),
}

pub async fn request(method: Method, url: &str) -> reqwest::Result<Response> {
    request_with_headers(method, std::iter::empty(), url).await
}

pub async fn request_with_headers(
    method: Method,
    headers: impl Iterator<Item = (HeaderName, HeaderValue)>,
    url: &str,
) -> reqwest::Result<Response> {
    let client = reqwest::Client::builder().zstd(true).build().unwrap();

    let mut request = client
        .request(method, url)
        .header(header::REFERER, "https://hitomi.la")
        .headers(HeaderMap::from_iter(headers));

    let is_ltn = url.starts_with("https://ltn.");

    if is_ltn {
        request = request.timeout(Duration::from_secs(3));
    }

    let mut retry = 0;

    let resp = loop {
        let resp = request.try_clone().unwrap().send().await;

        let resp = match resp {
            Ok(resp) => resp,
            Err(err) => {
                if is_ltn && err.is_timeout() && retry < 10 {
                    retry += 1;
                    continue;
                } else {
                    return Err(err);
                }
            }
        };

        break resp;
    };

    Ok(resp)
}
