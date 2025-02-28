use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gallery {
    pub id: u32,
    pub title: String,
    pub kind: String,
    /// (page, File)
    pub files: Vec<(usize, File)>,
    pub language: Option<String>,
    pub tags: Vec<Tag>,
    pub date: DateTime<Utc>,
}
