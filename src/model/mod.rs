use std::fmt::Display;

use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct File {
    pub has_webp: bool,
    pub has_avif: bool,
    pub width: usize,
    pub height: usize,
    pub hash: String,
    pub name: String,
}

#[derive(Debug, Serialize)]
pub enum TagKind {
    #[serde(rename = "artist")]
    Artist,
    #[serde(rename = "group")]
    Group,
    #[serde(rename = "series")]
    Series,
    #[serde(rename = "character")]
    Character,
    #[serde(rename = "female")]
    Female,
    #[serde(rename = "male")]
    Male,
    #[serde(rename = "misc")]
    Misc,
}

impl Display for TagKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use TagKind::*;

        let x = match self {
            Artist => "artist",
            Group => "group",
            Series => "series",
            Character => "character",
            Female => "female",
            Male => "male",
            Misc => "misc",
        };

        write!(f, "{x}")
    }
}

#[derive(Debug, Serialize)]
pub struct Tag {
    pub kind: TagKind,
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct Gallery {
    pub id: u32,
    pub title: String,
    pub kind: String,
    pub files: Vec<File>,
    pub language: Option<String>,
    pub tags: Vec<Tag>,
    pub date: String,
}
