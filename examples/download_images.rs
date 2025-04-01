use std::path::PathBuf;

use futures::{stream, StreamExt, TryStreamExt};
use hitomi_la::{
    gallery,
    gg::GG,
    image::{self, Image, ImageExt, ImageKind},
    nozomi::{self, Language},
};
use tokio::fs;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let base_dir = PathBuf::from("./galleries");

    let ids = nozomi::parse(Language::Korean, 1, 24).await?;

    println!("nozomi: {:?}", ids);

    let gg = GG::from_hitomi().await?;

    let galleries = stream::iter(ids)
        .map(gallery::parse)
        .buffered(8)
        .try_collect::<Vec<_>>()
        .await?;

    let gallery = galleries
        .into_iter()
        .flatten()
        .min_by_key(|g| g.files.len())
        .unwrap();

    println!("gallery: {:?}", gallery);

    let gallery_dir = base_dir.join(gallery.id.to_string());

    println!("create dir: {}", gallery_dir.as_os_str().to_string_lossy());

    fs::create_dir_all(&gallery_dir).await?;

    stream::iter(gallery.files)
        // page: starts from 1
        .map(|(page, file)| {
            let gg = &gg;
            let gallery_dir = &gallery_dir;

            async move {
                println!("download: start {}", page);

                let image: Image =
                    image::download(&file, ImageKind::Original, ImageExt::Avif, gg).await?;

                println!("download: complete {}", page);

                let image_dir = gallery_dir.join(format!("{}.{}", page, image.ext));

                fs::write(&image_dir, image.buf).await?;

                println!("write: {}", image_dir.as_os_str().to_string_lossy());

                Ok::<(), anyhow::Error>(())
            }
        })
        .buffer_unordered(4)
        .try_collect::<()>()
        .await?;

    Ok(())
}
