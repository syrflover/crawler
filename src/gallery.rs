use reqwest::Method;

use crate::{model, network::http::request};

mod sealed {
    use either::Either;
    use itertools::Itertools;
    use serde::{Deserialize, Deserializer};

    use crate::model;

    type Flag = Either<String, u8>;

    #[derive(Debug, Deserialize)]
    pub struct File {
        #[serde(with = "either::serde_untagged")]
        pub hasavif: Flag,
        #[serde(with = "either::serde_untagged")]
        pub haswebp: Flag,
        pub height: usize,
        pub width: usize,
        pub name: String,
        pub hash: String,
    }

    #[derive(Debug, Deserialize)]
    pub struct Artist {
        pub artist: String,
        pub url: String,
    }

    fn default_flag() -> Option<Flag> {
        Some(Either::Right(0))
    }

    #[derive(Debug, Deserialize)]
    pub struct Tag {
        #[serde(with = "either::serde_untagged_optional", default = "default_flag")]
        pub female: Option<Flag>,
        #[serde(with = "either::serde_untagged_optional", default = "default_flag")]
        pub male: Option<Flag>,
        pub tag: String,
        pub url: String,
    }

    #[derive(Debug, Deserialize)]
    pub struct Character {
        pub character: String,
        pub url: String,
    }

    #[derive(Debug, Deserialize)]
    pub struct Series {
        #[serde(rename = "parody")]
        pub series: String,
        pub url: String,
    }

    #[derive(Debug, Deserialize)]
    pub struct Group {
        pub group: String,
        pub url: String,
    }

    fn unwrap_or_default<'de, D, T>(d: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
        T: Default + Deserialize<'de>,
    {
        let opt = Option::deserialize(d)?;
        let val = opt.unwrap_or_default();
        Ok(val)
    }

    #[derive(Debug, Deserialize)]
    pub struct Gallery {
        #[serde(rename = "type")]
        pub kind: String,
        pub files: Vec<File>,
        #[serde(with = "either::serde_untagged")]
        pub id: Either<String, u32>,
        pub title: String,
        pub language: Option<String>,
        #[serde(default, deserialize_with = "unwrap_or_default")]
        pub artists: Vec<Artist>,
        #[serde(default, deserialize_with = "unwrap_or_default")]
        pub groups: Vec<Group>,
        #[serde(default, deserialize_with = "unwrap_or_default")]
        pub tags: Vec<Tag>,
        #[serde(default, deserialize_with = "unwrap_or_default")]
        pub characters: Vec<Character>,
        #[serde(rename = "parodys", default, deserialize_with = "unwrap_or_default")]
        pub series: Vec<Series>,
        pub date: String,
    }

    impl From<File> for model::File {
        fn from(file: File) -> Self {
            let has_webp = file.haswebp.right_or_else(|x| x.parse().unwrap_or(0)) == 1;
            let has_avif = file.hasavif.right_or_else(|x| x.parse().unwrap_or(0)) == 1;

            Self {
                has_webp,
                has_avif,
                width: file.width,
                height: file.height,
                hash: file.hash,
                name: file.name,
            }
        }
    }

    impl From<Artist> for model::Tag {
        fn from(x: Artist) -> Self {
            Self {
                kind: model::TagKind::Artist,
                name: x.artist,
            }
        }
    }

    impl From<Group> for model::Tag {
        fn from(x: Group) -> Self {
            Self {
                kind: model::TagKind::Group,
                name: x.group,
            }
        }
    }

    impl From<Series> for model::Tag {
        fn from(x: Series) -> Self {
            Self {
                kind: model::TagKind::Series,
                name: x.series,
            }
        }
    }

    impl From<Character> for model::Tag {
        fn from(x: Character) -> Self {
            Self {
                kind: model::TagKind::Character,
                name: x.character,
            }
        }
    }

    impl From<Tag> for model::Tag {
        fn from(x: Tag) -> Self {
            let is_female = x
                .female
                .unwrap_or(Either::Right(0))
                .right_or_else(|x| x.parse().unwrap_or(0))
                == 1;
            let is_male = x
                .male
                .unwrap_or(Either::Right(0))
                .right_or_else(|x| x.parse().unwrap_or(0))
                == 1;

            let kind = if is_female {
                model::TagKind::Female
            } else if is_male {
                model::TagKind::Male
            } else {
                model::TagKind::Misc
            };

            Self { kind, name: x.tag }
        }
    }

    impl From<Gallery> for model::Gallery {
        fn from(g: Gallery) -> Self {
            let id = g.id.right_or_else(|x| x.parse().unwrap());

            let artists = g.artists.into_iter().map_into();
            let groups = g.groups.into_iter().map_into();
            let series = g.series.into_iter().map_into();
            let characters = g.characters.into_iter().map_into();
            let tags = g.tags.into_iter().map_into();

            /* let date = {
                log::debug!("{:?}", g.date.chars().rev().nth(2));
                // "2022-07-25 06:30:00-05"
                if let Some('-' | '+') = g.date.chars().rev().nth(2) {
                    g.date + ":00"
                } else {
                    g.date
                }
            };

            log::debug!("{date}");
            */

            Self {
                id,
                title: g.title,
                kind: g.kind,
                files: g
                    .files
                    .into_iter()
                    .enumerate()
                    .map(|(i, file)| (i + 1, file.into()))
                    .collect(),
                language: g.language,
                tags: artists
                    .chain(groups)
                    .chain(series)
                    .chain(characters)
                    .chain(tags)
                    .collect(),
                date: g.date,
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(
        "deserialize gallery:\
         \n\n\
         {0}\
         \n\n\
         {1}"
    )]
    DeserializeGallery(String, serde_json::Error),
}

pub async fn parse(id: u32) -> crate::Result<model::Gallery> {
    let url = format!("https://ltn.hitomi.la/galleries/{id}.js");

    let resp = request(Method::GET, &url).await?;

    let txt = resp.text().await?;
    let (_, x) = txt.split_once('=').unwrap_or_default();

    let gallery: model::Gallery = serde_json::from_str::<sealed::Gallery>(x)
        .map_err(|err| Error::DeserializeGallery(txt, err))?
        .into();

    log::debug!("{gallery:#?}");
    log::debug!("page = {}", gallery.files.len());

    Ok(gallery)
}

#[cfg(test)]
mod tests {
    use crate::nozomi::{self, Language};

    use super::*;

    #[tokio::test]
    async fn parse_gallery() {
        simple_logger::init_with_level(log::Level::Debug).ok();

        let _ids = nozomi::parse(Language::Korean, 1, 25).await.unwrap();

        let mut galleries = Vec::new();

        // for id in ids {
        match parse(2288317).await {
            Ok(gallery) => {
                galleries.push(gallery);
            }
            Err(err) => {
                log::error!("{err}");
                panic!();
            }
        }
        // }

        let g = &galleries[0];

        log::debug!("{g:#?}");
    }
}
