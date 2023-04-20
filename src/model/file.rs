use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct File {
    pub has_webp: bool,
    pub has_avif: bool,
    pub width: usize,
    pub height: usize,
    pub hash: String,
    pub name: String,
}
