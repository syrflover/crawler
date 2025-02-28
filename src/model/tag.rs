use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

impl TagKind {
    pub fn as_str(&self) -> &str {
        self.as_ref()
    }
}

impl AsRef<str> for TagKind {
    fn as_ref(&self) -> &str {
        use TagKind::*;

        match self {
            Artist => "artist",
            Group => "group",
            Series => "series",
            Character => "character",
            Female => "female",
            Male => "male",
            Misc => "misc",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tag {
    pub kind: TagKind,
    pub name: String,
}
