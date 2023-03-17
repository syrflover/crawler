use bytes::Bytes;
use reqwest::Method;
use serde::Deserialize;

use crate::{
    model::File,
    network::{self, http::request},
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("can't parse url")]
    ParseUrl,
}

pub struct Image {
    url: String,
    kind: ImageKind,
}

impl Image {
    pub async fn new(id: u32, file: &File, kind: ImageKind) -> crate::Result<Self> {
        let url = {
            let (original_url, thumbnail_url) = match parse_url(id, file).await {
                Some((original_url, thumbnail_url)) => (original_url, thumbnail_url),
                None => return Err(Error::ParseUrl.into()),
            };

            match kind {
                ImageKind::Original => original_url,
                ImageKind::Thumbnail => thumbnail_url,
            }
        };

        Ok(Self { url, kind })
    }

    pub fn ext(&self) -> Option<&str> {
        self.url.split('.').last()
        // .expect("not include ext")
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
async fn parse_url(id: u32, file: &File) -> Option<(String, String)> {
    let id_string = id.to_string();
    let mut id_chars = id_string.chars();

    let c: u32 = id_chars
        .nth(id_string.len() - 1)
        .unwrap()
        .to_string()
        .encode_utf16()
        .next()
        .unwrap()
        .into();

    log::debug!("id_char utf16 code {}", c);

    let base_subdomain = if file.has_webp || file.has_avif {
        "a".to_string()
        /*
        let number_of_frontends = 2;

        char::from_u32(97 + c % number_of_frontends)
        .unwrap()
        .to_string() */
    } else {
        "b".to_string()
    };

    log::debug!("1st subdomain {}", base_subdomain);

    // log::debug!("file {:?}", file);

    let postfix = &file.hash[file.hash.len() - 3..].chars().collect::<Vec<_>>();

    log::debug!("hash {}", file.hash);
    log::debug!("postfix {:?}", postfix);

    // log::debug!("hash {}", file.hash);
    // log::debug!("postfix {:?}", postfix);

    let x = format!("{}{}{}", postfix[2], postfix[0], postfix[1]);

    log::debug!("x {}", x);

    // if let Ok(x) = u32::from_str_radix(&x, 16) {
    let x = u32::from_str_radix(&x, 16).unwrap();
    log::debug!("x {}", x);
    // log::debug!("x {}", x);

    let url = {
        let internal_url =
            std::env::var("LTN_URL").unwrap_or_else(|_| "http://localhost:13333".to_string());

        format!(
            "{}/gg.json?content-id={}&code-number={}",
            internal_url, id, x
        )
    };

    #[derive(Deserialize)]
    struct GgJson {
        m: u32,
        b: String,
    }

    let gg_json: GgJson =
        serde_json::from_str(&reqwest::get(&url).await.unwrap().text().await.unwrap()).unwrap();

    let n = gg_json.m;

    /* if x < 0x7c {
        n = 1;
    } */
    /* if x < 0x7a {
        n = 1;
    } */
    /* if x < 0x44 {
        n = 2;
    } */
    log::debug!("n {}", n);

    let calculated_subdomain = char::from_u32(97 + n).unwrap().to_string();

    let subdomain = format!("{}{}", calculated_subdomain, base_subdomain);

    log::debug!("2nd subdomain {}", subdomain);
    // log::debug!("2nd subdomain {}", subdomain);

    /*    let b = 16;

    let r = regex::Regex::new("/[0-9a-f]/([0-9a-f]{2})//").expect("regex create error");
    let m = r.find_iter(file); */

    /* let image_url = if file.has_webp() == false {
        format!(
            "https://{}b.hitomi.la/images/{}/{}{}/{}.{}",
            subdomain,
            postfix[2],
            postfix[0],
            postfix[1],
            file.hash,
            file.name.split(".").last().unwrap()
        )
    } else if file.hash.as_str() == "" {
        format!("https://{}a.hitomi.la/webp/{}.webp", subdomain, file.name)
    } else if file.hash.len() < 3 {
        format!("https://{}a.hitomi.la/webp/{}.webp", subdomain, file.hash)
    } else {
        format!(
            "https://{}a.hitomi.la/webp/{}/{}{}/{}.webp",
            subdomain, postfix[2], postfix[0], postfix[1], file.hash
        )
    }; */

    /* let image_url = format!(
        "https://{}b.hitomi.la/images/{}/{}{}/{}.{}",
        subdomain,
        postfix[2],
        postfix[0],
        postfix[1],
        file.hash,
        file.name.split('.').last().unwrap()
    ); */

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
        // panic!("hasn't webp / avif image of {}", id);
        return None;
    };

    // // log::debug!("image url = {}", image_url);

    /* let thumbnail_url = format!(
        "https://tn.hitomi.la/bigtn/{}/{}{}/{}.jpg",
        postfix[2], postfix[0], postfix[1], file.hash
    ); */

    /* let thumbnail_url = format!(
        "https://{}tn.hitomi.la/bigtn/{}/{}{}/{}.jpg",
        calculated_subdomain, postfix[2], postfix[0], postfix[1], file.hash
    ); */

    let thumbnail_url = if file.has_avif {
        format!(
            "https://{}tn.hitomi.la/avifbigtn/{}/{}{}/{}.avif",
            calculated_subdomain, postfix[2], postfix[0], postfix[1], file.hash
        )
    } else if file.has_webp {
        format!(
            "https://{}tn.hitomi.la/webpbigtn/{}/{}{}/{}.webp",
            calculated_subdomain, postfix[2], postfix[0], postfix[1], file.hash
        )
    } else {
        // panic!("hasn't webp / avif thumbnail of {}", id);
        return None;
    };

    // log::debug!("iamge = {:?}", file);
    // log::debug!("image url = {}", image_url);
    // log::debug!("thumbnail_url = {}", thumbnail_url);

    log::debug!("image_url = {}", image_url);
    log::debug!("thumbnail_url = {}", thumbnail_url);

    // log::debug!("image_url = {}", image_url);
    // log::debug!("thumbnail_url = {}", thumbnail_url);

    Some((image_url, thumbnail_url))
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use futures::stream::{self, StreamExt};

    use crate::{gallery, nozomi};

    use super::{Image, ImageKind};

    #[tokio::test]
    #[ignore = "need external service"]
    async fn parse_image_url() {
        simple_logger::init_with_level(log::Level::Debug).unwrap();

        let ids = nozomi::parse(1, 25).await.unwrap();

        let id = ids[2];

        let gallery_dir = format!("./sample/images/{id}");
        std::fs::create_dir_all(&gallery_dir).unwrap();

        let gallery = gallery::parse(id).await.unwrap();

        let (_, file) = &gallery.files[0];

        super::parse_url(id, file).await;
    }

    #[tokio::test]
    #[ignore = "using many network resource"]
    async fn download_thumbnail() {
        simple_logger::init_with_level(log::Level::Debug).unwrap();

        let ids = nozomi::parse(1, 25).await.unwrap();

        let id = ids[2];

        let gallery_dir = format!("./sample/images/{id}");
        std::fs::create_dir_all(&gallery_dir).unwrap();

        let gallery = gallery::parse(id).await.unwrap();

        let (_, file) = &gallery.files[0];

        let thumbnail = Image::new(id, file, ImageKind::Thumbnail).await.unwrap();

        let buf = thumbnail.download().await.unwrap();

        let name = format!("{}/thumbnail.{}", gallery_dir, thumbnail.ext().unwrap());
        let mut f = std::fs::File::create(name).unwrap();

        f.write_all(&buf).unwrap();
    }

    #[tokio::test]
    #[ignore = "using many network resource"]
    async fn download_images() {
        simple_logger::init_with_level(log::Level::Debug).unwrap();

        let ids = nozomi::parse(1, 25).await.unwrap();

        let id = ids[2];

        let gallery_dir = format!("./sample/images/{id}");
        std::fs::create_dir_all(&gallery_dir).unwrap();

        let gallery = gallery::parse(id).await.unwrap();

        let gallery_dir = &gallery_dir;
        stream::iter(gallery.files.iter())
            .map(|(_, file)| Image::new(id, file, ImageKind::Original))
            .enumerate()
            .for_each(|(p, fut)| async move {
                let image = fut.await.unwrap();

                let buf = image.download().await.unwrap();

                let name = format!("{}/{p}.{}", gallery_dir, image.ext().unwrap());
                let mut f = std::fs::File::create(name).unwrap();

                f.write_all(&buf).unwrap();
            })
            .await;
    }
}
