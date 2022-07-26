use reqwest::Method;

use crate::{model, network::http::request};

mod sealed {
    use either::Either;
    use itertools::Itertools;
    use serde::{Deserialize, Deserializer};

    use crate::model;

    #[derive(Debug, Deserialize)]
    pub struct File {
        #[serde(with = "either::serde_untagged")]
        pub hasavif: Either<String, u8>,
        #[serde(with = "either::serde_untagged")]
        pub haswebp: Either<String, u8>,
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

    fn default_either_flag() -> Option<Either<String, u8>> {
        Some(Either::Right(0))
    }

    #[derive(Debug, Deserialize)]
    pub struct Tag {
        #[serde(
            with = "either::serde_untagged_optional",
            default = "default_either_flag"
        )]
        pub female: Option<Either<String, u8>>,
        #[serde(
            with = "either::serde_untagged_optional",
            default = "default_either_flag"
        )]
        pub male: Option<Either<String, u8>>,
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

    fn null_to_default<'de, D, T>(d: D) -> Result<T, D::Error>
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
        #[serde(default, deserialize_with = "null_to_default")]
        pub artists: Vec<Artist>,
        #[serde(default, deserialize_with = "null_to_default")]
        pub groups: Vec<Group>,
        #[serde(default, deserialize_with = "null_to_default")]
        pub tags: Vec<Tag>,
        #[serde(default, deserialize_with = "null_to_default")]
        pub characters: Vec<Character>,
        #[serde(rename = "parodys", default, deserialize_with = "null_to_default")]
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

            Self {
                id,
                title: g.title,
                kind: g.kind,
                files: g.files.into_iter().map_into().collect(),
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

pub async fn parse(id: u32) -> crate::Result<model::Gallery> {
    let url = format!("https://ltn.hitomi.la/galleries/{id}.js");

    let resp = request(Method::GET, &url).await?;

    let buf = resp.text().await?;
    let buf = buf.split('=').nth(1).unwrap();

    let gallery: model::Gallery = serde_json::from_str::<sealed::Gallery>(buf).unwrap().into();

    log::debug!("{gallery:#?}");
    log::debug!("page = {}", gallery.files.len());

    /* let mut file = std::fs::File::create("./gallery.json").unwrap();

    file.write_all(buf.as_bytes()).unwrap(); */

    Ok(gallery)
}

#[cfg(test)]
mod tests {
    use crate::nozomi;

    // https://hitomi.la/gamecg/survivor-sarah-2-cruel-world-2278520.html
    #[tokio::test]
    async fn parse_gallery() {
        simple_logger::init_with_level(log::Level::Debug).unwrap();

        let ids = nozomi::parse(1, 25).await.unwrap();

        let id = ids[2];

        log::debug!("{id}");

        super::parse(id).await.unwrap();
    }
}
