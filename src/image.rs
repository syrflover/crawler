use std::{
    io,
    process::{Command, ExitStatus},
};

use bytes::Bytes;
use either::Either;
use reqwest::Method;
use serde::Deserialize;

use crate::{
    model::File,
    network::{self, http::request},
};

const LTN_URL: &str = "https://raw.githubusercontent.com/syrflover/ltn/master/src/main.ts";

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("hasn't image ext: {0:?}")]
    HasNotImage(Option<ImageExt>),

    #[error("can't parsed u32 from hash: hash = {0}; hex = {1}")]
    ParseU32FromHash(String, String),

    #[error("can't parsed prefix subdomain: x = {0}")]
    ParsePrefixOfSubdomain(u32),

    #[error("deserialize gg_json: {0}")]
    DeserializeGgJson(serde_json::Error),

    #[error("ltn: status = {0}; stdout = {1:?}; stderr = {2:?}")]
    Ltn(ExitStatus, Either<String, Vec<u8>>, Either<String, Vec<u8>>),

    #[error("command: {0}")]
    Command(io::Error),
}

pub struct Image {
    url: String,
    kind: ImageKind,
}

impl Image {
    /// kind: 원하는 이미지 분류를 지정합니다. 썸네일 또는 원본 이미지를 지정할 수 있습니다.
    /// ext: 원하는 이미지 확장자를 지정합니다. 지정하지 않았을 경우에는 avif, webp, jxl 순서대로 이미지가 있는지 확인하고 다운로드 받습니다.
    pub fn new(
        id: u32,
        file: &File,
        kind: ImageKind,
        ext: impl Into<Option<ImageExt>>,
    ) -> crate::Result<Self> {
        let url = {
            let (original_url, thumbnail_url) = parse_url(id, file, ext.into())?;

            match kind {
                ImageKind::Original => original_url,
                ImageKind::Thumbnail => thumbnail_url,
            }
        };

        Ok(Self { url, kind })
    }

    pub fn ext(&self) -> Option<&str> {
        self.url.split('.').last()
    }

    pub fn kind(&self) -> ImageKind {
        self.kind
    }

    pub async fn download(&self) -> crate::Result<Bytes> {
        let resp = request(Method::GET, &self.url).await?;

        let status = resp.status();

        if status.is_success() {
            let buf = resp.bytes().await?;
            Ok(buf)
        } else {
            Err(network::http::Error::Status(status).into())
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ImageKind {
    Thumbnail,
    Original,
}

#[derive(Debug, Clone, Copy)]
pub enum ImageExt {
    Jxl,
    Avif,
    Webp,
}

/// # Returns
/// (image_url, thumbnail_url)
fn parse_url(id: u32, file: &File, ext: Option<ImageExt>) -> Result<(String, String), Error> {
    // let id_string = id.to_string();
    // let mut id_chars = id_string.chars();

    // let c: u32 = id_chars
    //     .nth(id_string.len() - 1)
    //     .unwrap()
    //     .to_string()
    //     .encode_utf16()
    //     .next()
    //     .unwrap()
    //     .into();

    // tracing::debug!("id_char utf16 code {}", c);

    // # removed at 20240121
    // let base_subdomain = if file.has_webp || file.has_avif {
    //     'a'
    // } else {
    //     'b'
    // };

    let base_subdomain = 'a';

    tracing::debug!("base subdomain {}", base_subdomain);

    let postfix = file.hash[file.hash.len() - 3..].chars().collect::<Vec<_>>();

    tracing::debug!("hash {}", file.hash);
    tracing::debug!("postfix {:?}", postfix);

    let parsed_hex_from_hash = format!("{}{}{}", postfix[2], postfix[0], postfix[1]);
    let x = u32::from_str_radix(&parsed_hex_from_hash, 16)
        .map_err(|_| Error::ParseU32FromHash(file.hash.clone(), parsed_hex_from_hash.clone()))?;

    tracing::debug!("parsed u32 from hash {} -> {}", parsed_hex_from_hash, x);

    #[derive(Debug, Deserialize)]
    struct GgJson {
        m: u32,
        b: String,
    }

    let ltn = Command::new("deno")
        .args(["run", "--allow-net", LTN_URL])
        .arg(id.to_string())
        .arg(x.to_string())
        .output()
        .map_err(Error::Command)?;

    let gg_json: GgJson = if ltn.status.success() {
        serde_json::from_slice(&ltn.stdout).map_err(Error::DeserializeGgJson)?
    } else {
        let stdout = String::from_utf8(ltn.stdout.clone())
            .map(Either::Left)
            .unwrap_or_else(|_| Either::Right(ltn.stdout));
        let stderr = String::from_utf8(ltn.stderr.clone())
            .map(Either::Left)
            .unwrap_or_else(|_| Either::Right(ltn.stderr));

        return Err(Error::Ltn(ltn.status, stdout, stderr));
    };

    tracing::debug!("{gg_json:?}");

    let prefix_of_subdomain =
        char::from_u32(97 + gg_json.m).ok_or(Error::ParsePrefixOfSubdomain(gg_json.m))?;

    let subdomain = format!("{}{}", prefix_of_subdomain, base_subdomain);

    tracing::debug!("subdomain {}", subdomain);

    // # removed at 20240121
    // let image_url = if file.has_avif {
    //     format!(
    //         "https://{}.hitomi.la/avif/{}/{}/{}.avif",
    //         subdomain, gg_json.b, x, file.hash,
    //     )
    // } else if file.has_webp {
    //     format!(
    //         "https://{}.hitomi.la/webp/{}/{}/{}.webp",
    //         subdomain, gg_json.b, x, file.hash,
    //     )
    // } else {
    //     return Err(Error::HasNotImage);
    // };

    // # removed at 20240121
    // let thumbnail_url = if file.has_avif {
    //     format!(
    //         "https://{}tn.hitomi.la/avifbigtn/{}/{}{}/{}.avif",
    //         prefix_of_subdomain, postfix[2], postfix[0], postfix[1], file.hash
    //     )
    // } else if file.has_webp {
    //     format!(
    //         "https://{}tn.hitomi.la/webpbigtn/{}/{}{}/{}.webp",
    //         prefix_of_subdomain, postfix[2], postfix[0], postfix[1], file.hash
    //     )
    // } else {
    //     return Err(Error::HasNotImage);
    // };

    let ext = match ext {
        None | Some(ImageExt::Avif) if file.has_avif => "avif",
        None | Some(ImageExt::Webp) if file.has_webp => "webp",
        None | Some(ImageExt::Jxl) if file.has_jxl => "jxl",
        _ => return Err(Error::HasNotImage(None)),
    };

    let image_url = format!(
        "https://{}.hitomi.la/{ext}/{}/{}/{}.{ext}",
        subdomain, gg_json.b, x, file.hash,
    );

    let thumbnail_url = format!(
        "https://{}tn.hitomi.la/{ext}bigtn/{}/{}{}/{}.{ext}",
        prefix_of_subdomain, postfix[2], postfix[0], postfix[1], file.hash
    );

    // tracing::debug!("image_file = {:?}", file);
    tracing::debug!("image_url = {}", image_url);
    tracing::debug!("thumbnail_url = {}", thumbnail_url);

    Ok((image_url, thumbnail_url))
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use futures::stream::{self, StreamExt};

    use crate::{
        gallery,
        nozomi::{self, Language},
        tests::tracing,
    };

    use super::*;

    #[tokio::test]
    async fn parse_image_url() {
        tracing();

        let ids = nozomi::parse(Language::Korean, 1, 25).await.unwrap();

        let id = ids[2];

        let gallery_dir = format!("./sample/images/{id}");
        std::fs::create_dir_all(&gallery_dir).unwrap();

        let gallery = gallery::parse(id).await.unwrap();

        let (_, file) = &gallery.files[0];

        parse_url(id, file, None).unwrap();
    }

    #[tokio::test]
    async fn download_thumbnail() {
        tracing();

        // let ids = nozomi::parse(Language::Korean, 1, 25).await.unwrap();

        // let id = ids[2];
        let id = 2709834;

        let gallery_dir = format!("./sample/images/{id}");
        std::fs::create_dir_all(&gallery_dir).unwrap();

        let gallery = gallery::parse(id).await.unwrap();

        let (_, file) = &gallery.files[0];

        let thumbnail = Image::new(id, file, ImageKind::Thumbnail, None).unwrap();

        let buf = thumbnail.download().await.unwrap();

        let name = format!("{}/thumbnail.{}", gallery_dir, thumbnail.ext().unwrap());
        let mut f = std::fs::File::create(name).unwrap();

        f.write_all(&buf).unwrap();
    }

    #[tokio::test]
    #[ignore = "use many network resource"]
    async fn download_images() {
        tracing();

        // let ids = nozomi::parse(Language::Korean, 1, 25).await.unwrap();

        // let id = ids[2];
        let id = 2804886;

        let gallery_dir = format!("./sample/images/{id}");
        std::fs::create_dir_all(&gallery_dir).unwrap();

        let gallery = gallery::parse(id).await.unwrap();

        let gallery_dir = &gallery_dir;
        stream::iter(gallery.files.iter().take(100))
            .map(|(_, file)| Image::new(id, file, ImageKind::Original, ImageExt::Webp).unwrap())
            .enumerate()
            .for_each(|(p, image)| async move {
                let buf = image.download().await.unwrap();

                let name = format!("{}/{p}.{}", gallery_dir, image.ext().unwrap());
                let mut f = std::fs::File::create(name).unwrap();

                f.write_all(&buf).unwrap();
            })
            .await;
    }
}
