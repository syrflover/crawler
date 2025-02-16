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
        let home_dir = std::env::var("HOME").unwrap();
        let deno_install = format!("{home_dir}/.deno");
        let path = std::env::var("PATH").unwrap();
        std::env::set_var("PATH", format!("{deno_install}/bin:{path}"));

        if std::env::args().any(|arg| arg == "--nocapture") {
            let subscriber = tracing_subscriber::fmt()
                .with_max_level(tracing::Level::DEBUG)
                .with_line_number(true)
                .finish();

            tracing::subscriber::set_global_default(subscriber).ok();
        }
    }
}
