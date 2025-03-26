use std::fmt::Display;

use bytes::Bytes;
use itertools::Itertools;
use reqwest::Method;

use crate::{
    gg::GG,
    model::File,
    network::{
        self,
        http::{request, BASE_DOMAIN},
    },
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("hasn't image ext: {0:?}")]
    HasNotImage(ImageExt),

    #[error("can't parsed u32 from hash: hash = {0}; hex = {1}")]
    ParseU32FromHash(String, String),

    #[error("can't parsed prefix subdomain: x = {0}")]
    ParsePrefixOfSubdomain(u32),
}

#[derive(Debug, Clone, Copy)]
pub enum ImageKind {
    Thumbnail,
    Original,
}

#[derive(Debug, Clone, Copy)]
pub enum ImageExt {
    Avif,
    Webp,
}

impl ImageExt {
    pub fn as_str(&self) -> &str {
        self.as_ref()
    }
}

impl AsRef<str> for ImageExt {
    fn as_ref(&self) -> &str {
        match self {
            ImageExt::Avif => "avif",
            ImageExt::Webp => "webp",
        }
    }
}

impl Display for ImageExt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_ref().fmt(f)
    }
}

impl Default for ImageExt {
    fn default() -> Self {
        Self::Avif
    }
}

pub struct Image {
    pub kind: ImageKind,
    pub ext: ImageExt,
    pub url: String,
    pub buf: Bytes,
}

pub async fn download(
    file: &File,
    kind: ImageKind,
    ext: ImageExt,
    gg: &GG,
) -> crate::Result<Image> {
    let image_url = parse_url(file, kind, ext, gg)?;

    let resp = request(Method::GET, &image_url).await?;

    let status = resp.status();

    if status.is_success() {
        let buf = resp.bytes().await?;

        Ok(Image {
            kind,
            ext,
            url: image_url,
            buf,
        })
    } else {
        Err(network::http::Error::Status(status).into())
    }
}

fn parse_url(file: &File, kind: ImageKind, ext: ImageExt, gg: &GG) -> Result<String, Error> {
    // validate image ext exists on hitomi
    match ext {
        ImageExt::Avif if file.has_avif => {}
        ImageExt::Webp if file.has_webp => {}
        _ => return Err(Error::HasNotImage(ext)),
    }

    let base_subdomain = match ext {
        ImageExt::Webp => 'w',
        ImageExt::Avif => 'a',
    };

    tracing::debug!(?base_subdomain);

    // var r = /\/[0-9a-f]{61}([0-9a-f]{2})([0-9a-f])/;
    let postfix = file.hash[file.hash.len() - 3..]
        .chars()
        .collect_array::<3>()
        .unwrap();

    tracing::debug!(?file.hash);
    tracing::debug!(?postfix);

    let parsed_hex_from_hash = format!("{}{}{}", postfix[2], postfix[0], postfix[1]);

    tracing::debug!(?parsed_hex_from_hash);

    let g = u32::from_str_radix(&parsed_hex_from_hash, 16)
        .map_err(|_| Error::ParseU32FromHash(file.hash.clone(), parsed_hex_from_hash.clone()))?;

    let m = gg.m(g);

    tracing::debug!(?g);

    let image_url = match kind {
        ImageKind::Thumbnail => {
            let prefix_of_subdomain =
                char::from_u32(97 + m).ok_or(Error::ParsePrefixOfSubdomain(m))?;

            let subdomain = format!("{}tn", prefix_of_subdomain);

            tracing::debug!(?subdomain);

            format!(
                "https://{}.{BASE_DOMAIN}/{ext}bigtn/{}/{}{}/{}.{ext}",
                subdomain, postfix[2], postfix[0], postfix[1], file.hash
            )
        }
        ImageKind::Original => {
            let subdomain = format!("{}{}", base_subdomain, 1 + m);

            tracing::debug!(?subdomain);

            format!(
                "https://{}.{BASE_DOMAIN}/{}/{}/{}.{ext}",
                subdomain,
                gg.b(),
                g,
                file.hash,
            )
        }
    };

    tracing::debug!(?image_url);

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
    async fn download_image() {
        tracing();

        let ids = nozomi::parse(Language::Korean, 1, 25).await.unwrap();

        let id = ids[2];

        let gallery_dir = format!("./sample/images/{id}");
        std::fs::create_dir_all(&gallery_dir).unwrap();

        let gallery = gallery::parse(id).await.unwrap().unwrap();

        let (_, file) = &gallery.files[0];

        let gg = GG::from_hitomi().await.unwrap();

        // avif thumbnail
        let avif_thumbnail = {
            let gg = &gg;
            let gallery_dir = &*gallery_dir;
            async move {
                let image = download(file, ImageKind::Thumbnail, ImageExt::Avif, gg)
                    .await
                    .unwrap();

                let name = format!("{}/thumbnail.{}", gallery_dir, image.ext.as_str());
                let mut f = std::fs::File::create(name).unwrap();

                f.write_all(&image.buf).unwrap();
            }
        };

        // avif original
        let avif_original = {
            let gg = &gg;
            let gallery_dir = &*gallery_dir;
            async move {
                let image = download(file, ImageKind::Original, ImageExt::Avif, gg)
                    .await
                    .unwrap();

                let name = format!("{}/original.{}", gallery_dir, image.ext.as_str());
                let mut f = std::fs::File::create(name).unwrap();

                f.write_all(&image.buf).unwrap();
            }
        };

        tokio::join!(avif_thumbnail, avif_original);
    }

    #[tokio::test]
    #[ignore = "use many network resource"]
    async fn download_images() {
        tracing();

        // let ids = nozomi::parse(Language::Korean, 1, 25).await.unwrap();

        // let id = ids[2];
        let id = 3282933;

        let gg = GG::from_hitomi().await.unwrap();

        let gallery_dir = format!("./sample/images/{id}");
        std::fs::create_dir_all(&gallery_dir).unwrap();

        let gallery = gallery::parse(id).await.unwrap().unwrap();

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
                    let image = download(file, ImageKind::Original, ImageExt::Avif, gg)
                        .await
                        .unwrap();

                    let name = format!("{}/{p}.{}", gallery_dir, image.ext.as_str());
                    let mut f = std::fs::File::create(name).unwrap();

                    f.write_all(&image.buf).unwrap();
                }
            })
            .await;
    }
}
