use std::{marker::PhantomData, time::Duration};

use reqwest::{
    header::{self, HeaderMap, HeaderName, HeaderValue},
    Method, Response, StatusCode,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Status: {0}")]
    Status(StatusCode),
}

struct EmptyIter<T>(PhantomData<T>);

impl<T> EmptyIter<T> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<T> Iterator for EmptyIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}

pub async fn request(method: Method, url: &str) -> reqwest::Result<Response> {
    request_with_headers(method, EmptyIter::new(), url).await
}

pub async fn request_with_headers(
    method: Method,
    headers: impl Iterator<Item = (HeaderName, HeaderValue)>,
    url: &str,
) -> reqwest::Result<Response> {
    let mut request = reqwest::Client::new()
        .request(method, url)
        .header(header::REFERER, "https://hitomi.la")
        .headers(HeaderMap::from_iter(headers));

    let is_ltn = url.starts_with("https://ltn.hitomi.la");

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
