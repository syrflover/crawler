use std::io;

use crate::network;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Http: {0}")]
    Http(#[from] network::http::Error),

    // TODO: move to network::http::Error
    #[error("Reqwest: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("Io: {0}")]
    Io(#[from] io::Error),

    #[error("Image: {0}")]
    Image(#[from] crate::image::Error),

    #[error("Gallery: {0}")]
    Gallery(#[from] crate::gallery::Error),

    #[error("GG: {0}")]
    GG(#[from] crate::gg::Error),
}
