//! # hitomi.rs
//!
//! A hitomi.la API wrapper for Rust programming language.

pub mod error;
pub mod gallery;
pub mod gg;
pub mod image;
pub mod model;
pub mod network;
pub mod nozomi;

pub use error::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    pub fn tracing() {
        if std::env::args().any(|arg| arg == "--nocapture") {
            let subscriber = tracing_subscriber::fmt()
                .with_max_level(tracing::Level::DEBUG)
                .with_line_number(true)
                .finish();

            tracing::subscriber::set_global_default(subscriber).ok();
        }
    }
}
