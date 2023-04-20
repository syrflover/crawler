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
    #[error("hasn't avif or webp")]
    HasNotAvifOrWebp,

    #[error("can't parsed u32 from hash: hash = {0}; hex = {1}")]
    ParseU32FromHash(String, String),

    #[error("can't parsed prefix subdomain: x = {0}")]
    ParsePrefixOfSubdomain(u32),

    #[error("deserialize gg_json: {0}")]
    DeserializeGgJson(serde_json::Error),

    #[error("ltn: status = {0}; stdout = {1:?}; stderr = {2:?}")]
    Ltn(ExitStatus, Either<String, Vec<u8>>, Either<String, Vec<u8>>),

    #[error("io: {0}")]
    Io(#[from] io::Error),
}

pub struct Image {
    url: String,
    kind: ImageKind,
}

impl Image {
    pub fn new(id: u32, file: &File, kind: ImageKind) -> crate::Result<Self> {
        let url = {
            let (original_url, thumbnail_url) = parse_url(id, file)?;

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

/// # Returns
/// (image_url, thumbnail_url)
fn parse_url(id: u32, file: &File) -> Result<(String, String), Error> {
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

    // log::debug!("id_char utf16 code {}", c);

    let base_subdomain = if file.has_webp || file.has_avif {
        'a'
    } else {
        'b'
    };

    log::debug!("base subdomain {}", base_subdomain);

    let postfix = file.hash[file.hash.len() - 3..].chars().collect::<Vec<_>>();

    log::debug!("hash {}", file.hash);
    log::debug!("postfix {:?}", postfix);

    let parsed_hex_from_hash = format!("{}{}{}", postfix[2], postfix[0], postfix[1]);
    let x = u32::from_str_radix(&parsed_hex_from_hash, 16)
        .map_err(|_| Error::ParseU32FromHash(file.hash.clone(), parsed_hex_from_hash.clone()))?;

    log::debug!("parsed u32 from hash {} -> {}", parsed_hex_from_hash, x);

    #[derive(Debug, Deserialize)]
    struct GgJson {
        m: u32,
        b: String,
    }

    let ltn = Command::new("deno")
        .args(["run", "--allow-net", LTN_URL])
        .arg(id.to_string())
        .arg(x.to_string())
        .output()?;

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

    log::debug!("{gg_json:?}");

    let prefix_of_subdomain =
        char::from_u32(97 + gg_json.m).ok_or(Error::ParsePrefixOfSubdomain(gg_json.m))?;

    let subdomain = format!("{}{}", prefix_of_subdomain, base_subdomain);

    log::debug!("subdomain {}", subdomain);

    let image_url = if file.has_avif {
        format!(
            "https://{}.hitomi.la/avif/{}/{}/{}.avif",
            subdomain, gg_json.b, x, file.hash,
        )
    } else if file.has_webp {
        format!(
            "https://{}.hitomi.la/webp/{}/{}/{}.webp",
            subdomain, gg_json.b, x, file.hash,
        )
    } else {
        return Err(Error::HasNotAvifOrWebp);
    };

    let thumbnail_url = if file.has_avif {
        format!(
            "https://{}tn.hitomi.la/avifbigtn/{}/{}{}/{}.avif",
            prefix_of_subdomain, postfix[2], postfix[0], postfix[1], file.hash
        )
    } else if file.has_webp {
        format!(
            "https://{}tn.hitomi.la/webpbigtn/{}/{}{}/{}.webp",
            prefix_of_subdomain, postfix[2], postfix[0], postfix[1], file.hash
        )
    } else {
        return Err(Error::HasNotAvifOrWebp);
    };

    // log::debug!("image_file = {:?}", file);
    log::debug!("image_url = {}", image_url);
    log::debug!("thumbnail_url = {}", thumbnail_url);

    Ok((image_url, thumbnail_url))
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use futures::stream::{self, StreamExt};

    use crate::{
        gallery,
        nozomi::{self, Language},
    };

    use super::*;

    #[tokio::test]
    async fn parse_image_url() {
        simple_logger::init_with_level(log::Level::Debug).ok();

        let ids = nozomi::parse(Language::Korean, 1, 25).await.unwrap();

        let id = ids[2];

        let gallery_dir = format!("./sample/images/{id}");
        std::fs::create_dir_all(&gallery_dir).unwrap();

        let gallery = gallery::parse(id).await.unwrap();

        let (_, file) = &gallery.files[0];

        parse_url(id, file).unwrap();
    }

    #[tokio::test]
    async fn download_thumbnail() {
        simple_logger::init_with_level(log::Level::Debug).ok();

        let ids = nozomi::parse(Language::Korean, 1, 25).await.unwrap();

        let id = ids[2];

        let gallery_dir = format!("./sample/images/{id}");
        std::fs::create_dir_all(&gallery_dir).unwrap();

        let gallery = gallery::parse(id).await.unwrap();

        let (_, file) = &gallery.files[0];

        let thumbnail = Image::new(id, file, ImageKind::Thumbnail).unwrap();

        let buf = thumbnail.download().await.unwrap();

        let name = format!("{}/thumbnail.{}", gallery_dir, thumbnail.ext().unwrap());
        let mut f = std::fs::File::create(name).unwrap();

        f.write_all(&buf).unwrap();
    }

    #[tokio::test]
    #[ignore = "use many network resource"]
    async fn download_images() {
        simple_logger::init_with_level(log::Level::Debug).ok();

        let ids = nozomi::parse(Language::Korean, 1, 25).await.unwrap();

        let id = ids[2];

        let gallery_dir = format!("./sample/images/{id}");
        std::fs::create_dir_all(&gallery_dir).unwrap();

        let gallery = gallery::parse(id).await.unwrap();

        let gallery_dir = &gallery_dir;
        stream::iter(gallery.files.iter())
            .map(|(_, file)| Image::new(id, file, ImageKind::Original).unwrap())
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
