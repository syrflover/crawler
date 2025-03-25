use std::num::NonZeroUsize;

use reqwest::{
    header::{self, HeaderName, HeaderValue},
    Method,
};

use crate::network::http::{request_with_headers, BASE_DOMAIN};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("required page must be non-zero")]
    InvalidPage,
}

#[derive(Debug, Clone, Copy)]
pub enum Language {
    All,
    Korean,
    Japanese,
    English,
}

impl Language {
    fn to_nozomi_url(self) -> String {
        match self {
            Language::All => format!("https://ltn.{}/index-all.nozomi", BASE_DOMAIN),
            Language::Korean => format!("https://ltn.{}/index-korean.nozomi", BASE_DOMAIN),
            Language::Japanese => format!("https://ltn.{}/index-japanese.nozomi", BASE_DOMAIN),
            Language::English => format!("https://ltn.{}/index-english.nozomi", BASE_DOMAIN),
        }
    }
}

#[inline]
fn range(page: usize, per_page: usize) -> (usize, usize) {
    let start_byte = (page - 1) * per_page * 4;
    let end_byte = start_byte + per_page * 4 - 1;

    (start_byte, end_byte)
}

/// Fetches the nozomi file from hitomi server and Returns ID list sorted in descending order.
///
/// ## Errors
/// - if page == zero
pub async fn parse(
    lang: Language,
    page: impl TryInto<NonZeroUsize>,
    per_page: usize,
) -> crate::Result<Vec<u32>> {
    let (start_byte, end_byte) = range(
        page.try_into().map_err(|_| Error::InvalidPage)?.into(),
        per_page,
    );

    tracing::trace!("start_byte={}", start_byte);
    tracing::trace!("end_byte={}", end_byte);

    let range: (HeaderName, HeaderValue) = (
        header::RANGE,
        format!("bytes={}-{}", start_byte, end_byte)
            .try_into()
            .unwrap(),
    );

    let resp =
        request_with_headers(Method::GET, std::iter::once(range), &lang.to_nozomi_url()).await?;

    let bytes = resp.bytes().await?;

    tracing::trace!("bytes={:?}", bytes);

    // check bytes length
    debug_assert_eq!(per_page, bytes.len() / 4);

    let mut res = Vec::with_capacity(per_page);

    for step in (0..bytes.len()).step_by(4) {
        tracing::trace!("step={}", step);

        let mut acc = 0;

        // similar to u32::from_be_bytes
        for j in 0..3 {
            if let Some(byte) = bytes.get(step + (3 - j)) {
                let byte: u32 = (*byte).into();
                tracing::trace!("byte={}", byte);

                acc += byte << (j << 3);
                tracing::trace!("acc={}", acc);
            } else {
                // TODO: throw error
                break;
            }
        }

        res.push(acc);
    }

    res.sort_unstable_by(|a, b| b.cmp(a));

    tracing::debug!("ids={res:?}");

    Ok(res)
}

#[cfg(test)]
mod tests {
    use crate::tests::tracing;

    use super::*;

    #[tokio::test]
    async fn parse_nozomi() {
        tracing();

        let ids = parse(Language::Korean, 1, 25).await.unwrap();

        // length
        assert_eq!(ids.len(), 25);

        // descending
        assert!(ids.iter().rev().is_sorted());
    }
}
