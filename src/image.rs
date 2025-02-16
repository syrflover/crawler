use std::{io, process::ExitStatus};

use bytes::Bytes;
use either::Either;
use reqwest::Method;

use crate::{
    gg::GG,
    model::File,
    network::{self, http::request},
};

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

pub struct Image {
    pub kind: ImageKind,
    pub url: String,
    pub buf: Bytes,
}

impl Image {
    pub fn ext(&self) -> Option<&str> {
        self.url.split('.').last()
    }
}

pub async fn download(
    file: &File,
    kind: ImageKind,
    ext: impl Into<Option<ImageExt>>,
    gg: &GG,
) -> crate::Result<Image> {
    let image_url = parse_url(file, kind, ext.into(), gg)?;

    let resp = request(Method::GET, &image_url).await?;

    let status = resp.status();

    if status.is_success() {
        let buf = resp.bytes().await?;

        Ok(Image {
            kind,
            url: image_url,
            buf,
        })
    } else {
        Err(network::http::Error::Status(status).into())
    }
}

/// # Returns
/// (image_url, thumbnail_url)
fn parse_url(
    file: &File,
    kind: ImageKind,
    ext: Option<ImageExt>,
    gg: &GG,
) -> Result<String, Error> {
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
    let g = u32::from_str_radix(&parsed_hex_from_hash, 16)
        .map_err(|_| Error::ParseU32FromHash(file.hash.clone(), parsed_hex_from_hash.clone()))?;

    let m = gg.m(g);

    tracing::debug!("parsed u32 from hash {} -> {}", parsed_hex_from_hash, g);

    let prefix_of_subdomain = char::from_u32(97 + m).ok_or(Error::ParsePrefixOfSubdomain(m))?;

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
        _ => return Err(Error::HasNotImage(ext)),
    };

    let image_url = match kind {
        ImageKind::Thumbnail => {
            format!(
                "https://{}tn.hitomi.la/{ext}bigtn/{}/{}{}/{}.{ext}",
                prefix_of_subdomain, postfix[2], postfix[0], postfix[1], file.hash
            )
        }
        ImageKind::Original => format!(
            "https://{}.hitomi.la/{ext}/{}/{}/{}.{ext}",
            subdomain,
            gg.b(),
            g,
            file.hash,
        ),
    };

    // tracing::debug!("image_file = {:?}", file);
    tracing::debug!("image_url = {}", image_url);
    // tracing::debug!("thumbnail_url = {}", thumbnail_url);

    Ok(image_url)
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

        let gg = GG::from_hitomi().await.unwrap();

        parse_url(file, ImageKind::Original, None, &gg).unwrap();
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

        let gg = GG::from_hitomi().await.unwrap();

        let image = download(file, ImageKind::Thumbnail, None, &gg)
            .await
            .unwrap();

        let name = format!("{}/thumbnail.{}", gallery_dir, image.ext().unwrap());
        let mut f = std::fs::File::create(name).unwrap();

        f.write_all(&image.buf).unwrap();
    }

    #[tokio::test]
    #[ignore = "use many network resource"]
    async fn download_images() {
        tracing();

        // let ids = nozomi::parse(Language::Korean, 1, 25).await.unwrap();

        // let id = ids[2];
        let id = 2804886;

        let gg = GG::from_hitomi().await.unwrap();

        let gallery_dir = format!("./sample/images/{id}");
        std::fs::create_dir_all(&gallery_dir).unwrap();

        let gallery = gallery::parse(id).await.unwrap();

        std::fs::write(
            format!("{}/files.json", gallery_dir),
            serde_json::to_vec_pretty(&gallery).unwrap(),
        )
        .unwrap();

        let gallery_dir = &gallery_dir;
        stream::iter(gallery.files.iter().take(100))
            .enumerate()
            .for_each(|(p, (_, file))| {
                let gg = &gg;
                async move {
                    let image = download(file, ImageKind::Original, None, gg).await.unwrap();

                    let name = format!("{}/{p}.{}", gallery_dir, image.ext().unwrap());
                    let mut f = std::fs::File::create(name).unwrap();

                    f.write_all(&image.buf).unwrap();
                }
            })
            .await;
    }
}
