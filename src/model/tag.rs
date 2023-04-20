use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub kind: TagKind,
    pub name: String,
}
