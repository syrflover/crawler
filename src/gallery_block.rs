//! group, character를 제외한 metadata를 파싱할 수 있음
//!
//! useless module, use instead crate::gallery

use reqwest::{Method, StatusCode};
use scraper::{ElementRef, Html, Selector};

use crate::network::http::request;

#[derive(Debug, Clone, Default)]
pub struct GalleryBlock {
    pub gallery_path: String,
    pub title: String,
    pub kind: String,
    pub language: String,
    pub created_at: String,
    pub tags: Vec<String>,
    pub artists: Vec<String>,
    pub series: Vec<String>,
}

#[derive(Debug, Clone, Copy)]
pub enum MetadataKind {
    GalleryPath,
    Title,
    Kind,
    Language,
    Tags,
    Series,
    Group,
    Artist,
    CreatedAt,
}

impl MetadataKind {
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        use MetadataKind::*;

        let r = match s {
            "GalleryPath" => GalleryPath,
            "Title" => Title,
            "Type" => Kind,
            "Language" => Language,
            "Tags" => Tags,
            "Series" => Series,
            "Group" => Group,
            "Artist" => Artist,
            "CreatedAt" => CreatedAt,
            _s => return None,
        };

        Some(r)
    }
}

/// ```html
/// <td>
///     <ul>
///         <li><a href="/series/original-all.html">original</a></li>
///     </ul>
/// </td>
/// ```
fn parse_list_metadata(td: ElementRef<'_>) -> Vec<String> {
    // TODO: N/A case도 처리해줘
    let sel = Selector::parse("li").unwrap();
    let li = td.select(&sel);

    let sel = Selector::parse("a").unwrap();
    li.filter_map(|x| x.select(&sel).next())
        .map(|x| x.text().collect())
        .collect()
}

fn parse_single_metadata(td: ElementRef<'_>) -> Option<String> {
    // TODO: N/A case도 처리해줘
    let sel = Selector::parse("a").unwrap();
    let mut a = td.select(&sel);

    Some(a.next()?.text().collect())
}

/// .dj-content > dj-desc > tr > td > a
pub fn parse_metadata(dom: &Html) -> Option<GalleryBlock> {
    let mut gallery_block = GalleryBlock::default();

    // title
    /* let sel = Selector::parse(".dj > h1 > a").unwrap();
    let title = dom
        .select(&sel)
        .next()?
        .text()
        .flat_map(|x| x.chars())
        .filter(|x| !x.is_whitespace())
        .collect::<String>();
    log::debug!("{title}");

    {
        metadata.title = title;
    } */

    let sel = Selector::parse(".dj-desc").unwrap();
    let desc_table = dom.select(&sel).next()?;

    let sel = Selector::parse("tr").unwrap();
    let rows = desc_table.select(&sel);

    for row in rows {
        let sel = Selector::parse("td").unwrap();
        let mut td = row.select(&sel);

        // <td>Series</td>
        let metadata_kind = td.next()?.first_child()?.value().as_text()?.to_string();

        log::debug!("{metadata_kind}");

        match MetadataKind::from_str(&metadata_kind) {
            Some(MetadataKind::Series) => {
                let list = parse_list_metadata(td.next()?);

                log::debug!("- {list:?}");

                gallery_block.series = list;
            }

            Some(MetadataKind::Tags) => {
                let list = parse_list_metadata(td.next()?);

                log::debug!("- {list:?}");

                gallery_block.tags = list;
            }

            Some(MetadataKind::Kind) => {
                let kind = parse_single_metadata(td.next()?)?;

                log::debug!("- {kind}");

                gallery_block.kind = kind;
            }

            Some(MetadataKind::Language) => {
                let language = parse_single_metadata(td.next()?);

                log::debug!("- {language:?}");

                if let Some(language) = language {
                    gallery_block.language = language;
                }
            }

            _ => {}
        }
    }

    // title
    {
        let sel = Selector::parse("h1 > a").unwrap();
        let title = dom.select(&sel).next()?;

        let gallery_path = title.value().attr("href").unwrap().to_owned();

        log::debug!("{:?}", MetadataKind::GalleryPath);
        log::debug!("- {gallery_path}");

        gallery_block.gallery_path = gallery_path;

        let title = title.text().collect::<String>();

        log::debug!("{:?}", MetadataKind::Title);
        log::debug!("- {title}");

        gallery_block.title = title;
    }

    // artists
    {
        let sel = Selector::parse(".artist-list").unwrap();
        let artist_list = dom.select(&sel).next()?;

        let artist_list = parse_list_metadata(artist_list);

        log::debug!("{:?}", MetadataKind::Artist);
        log::debug!("- {artist_list:?}");

        gallery_block.artists = artist_list;
    }

    // created at
    {
        let sel = Selector::parse(".date").unwrap();
        let created_at = dom.select(&sel).next()?;

        let created_at = created_at.text().collect::<String>();

        log::debug!("{:?}", MetadataKind::CreatedAt);
        log::debug!("- {created_at}");

        gallery_block.created_at = created_at;
    }

    log::debug!("{gallery_block:#?}");

    Some(gallery_block)
}

pub async fn parse(id: u32) -> crate::Result<GalleryBlock> {
    let url = format!("https://ltn.hitomi.la/galleryblock/{}.html", id);
    let resp = request(Method::GET, &url).await?;

    if resp.status() != StatusCode::OK {
        todo!("Error Handle");
    }

    let buf = resp.text().await?;

    let dom = Html::parse_fragment(&buf);

    let gallery_block = parse_metadata(&dom).unwrap();

    Ok(gallery_block)
}

#[cfg(test)]
mod tests {
    use crate::nozomi;

    #[tokio::test]
    async fn parse_meta() {
        simple_logger::init_with_level(log::Level::Debug).ok();

        let ids = nozomi::parse(1, 25).await.unwrap();

        // no tags, no artists, no series
        // let id = 2275484;

        // have all
        // 2277954

        let id = ids[2];

        log::debug!("{}", id);

        super::parse(id).await.unwrap();
    }
}
