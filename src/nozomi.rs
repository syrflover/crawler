use reqwest::{
    header::{self, HeaderName, HeaderValue},
    Method,
};

use crate::network::http::request_with_headers;

#[derive(Debug, Clone, Copy)]
pub enum Language {
    All,
    Korean,
    Japanese,
    English,
}

impl Language {
    fn to_nozomi_url(&self) -> &str {
        match self {
            Language::All => "https://ltn.hitomi.la/index-all.nozomi",
            Language::Korean => "https://ltn.hitomi.la/index-korean.nozomi",
            Language::Japanese => "https://ltn.hitomi.la/index-japanese.nozomi",
            Language::English => "https://ltn.hitomi.la/index-english.nozomi",
        }
    }
}

fn range(page: usize, per_page: usize) -> (usize, usize) {
    let start_bytes = (page - 1) * per_page * 4;
    let end_bytes = start_bytes + per_page * 4 - 1;

    (start_bytes, end_bytes)
}

/// tested only korean
pub async fn parse(
    lang: impl Into<Option<Language>>,
    page: usize,
    per_page: usize,
) -> crate::Result<Vec<u32>> {
    let lang = (lang.into() as Option<_>).unwrap_or(Language::All);
    let url = lang.to_nozomi_url();

    let (start_bytes, end_bytes) = range(page, per_page);

    log::debug!("start_bytes = {}", start_bytes);
    log::debug!("end_bytes = {}", end_bytes);

    let range: (HeaderName, HeaderValue) = (
        header::RANGE,
        format!("bytes={}-{}", start_bytes, end_bytes)
            .try_into()
            .unwrap(),
    );

    let resp = request_with_headers(Method::GET, [range].into_iter(), url).await?;

    let bytes = resp.bytes().await?;

    let mut res = vec![];

    for i in (0..bytes.len()).step_by(4) {
        let mut temp = 0;

        for j in 0..3 {
            // https://github.com/Project-Madome/Madome-Synchronizer/issues/1
            if let Some(a) = bytes.get(i + (3 - j)) {
                let a: u32 = (*a).try_into().unwrap();
                temp += a << (j << 3);
            } else {
                break;
            }
        }

        // log::debug!("id = {}", temp);

        res.push(temp);
    }

    res.sort_by(|a, b| b.cmp(a));

    log::debug!("ids = {res:?}");

    Ok(res)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn parse_nozomi() {
        simple_logger::init_with_level(log::Level::Debug).ok();

        let _ids = parse(Language::Korean, 1, 25).await.unwrap();
    }
}
