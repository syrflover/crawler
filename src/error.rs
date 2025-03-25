use std::io;

use crate::network;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Http: {0}")]
    Http(#[from] network::http::Error),

    #[error("Io: {0}")]
    Io(#[from] io::Error),

    #[error("Nozomi: {0}")]
    Nozomi(#[from] crate::nozomi::Error),

    #[error("Image: {0}")]
    Image(#[from] crate::image::Error),

    #[error("Gallery: {0}")]
    Gallery(#[from] crate::gallery::Error),

    #[error("GG: {0}")]
    GG(#[from] crate::gg::Error),
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::Http(network::http::Error::from(err))
    }
}
