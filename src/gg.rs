use std::collections::HashMap;

use regex::Regex;
use reqwest::Method;

use crate::network::{
    self,
    http::{BASE_DOMAIN, request},
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to parse gg.js")]
    Parse,
}

pub struct GG {
    m: HashMap<u32, u32>,
    b: String,
    default: u32,
}

impl GG {
    pub async fn from_hitomi() -> crate::Result<Self> {
        let resp = request(Method::GET, &format!("https://ltn.{}/gg.js", BASE_DOMAIN)).await?;
        let status = resp.status();

        if status.is_success() {
            let text = resp.text().await.unwrap_or_default();

            Self::from_js(&text)
        } else {
            Err(network::http::Error::Status(status).into())
        }
    }

    pub fn from_js(s: &str) -> crate::Result<Self> {
        parse_gg(s).ok_or(Error::Parse.into())
    }

    pub(crate) fn m(&self, key: u32) -> u32 {
        self.m.get(&key).copied().unwrap_or(self.default)
    }

    pub(crate) fn b(&self) -> &str {
        &self.b
    }
}

fn parse_gg(s: &str) -> Option<GG> {
    let default_regex = Regex::new(r#"(?si)(var\s|default:)\s*o\s*=\s*(?<default>\d+)"#).unwrap();

    let default_value = default_regex
        .captures(s)?
        .name("default")
        .unwrap()
        .as_str()
        .parse::<u32>()
        .unwrap();

    //

    let case_regex = Regex::new(r#"(?si)case\s+(?<key>\d+):\s+o?\s*=?\s*(?<value>\d+)?"#).unwrap();

    let mut m = HashMap::new();
    let mut keys = Vec::new();

    for caps in case_regex.captures_iter(s) {
        let key = caps["key"].parse::<u32>().unwrap();

        keys.push(key);

        if let Some(value) = caps.name("value") {
            let value = value.as_str().parse::<u32>().unwrap();

            for key in keys.drain(..) {
                m.insert(key, value);
            }
        }
    }

    // for key in keys {
    //     m.insert(key, default_value);
    // }

    //

    let cond_regex = Regex::new(
        r#"(?si)if\s*[(]g\s*===\s*(?<key>\d+)[)]\s*[{]?\s*o\s*=\s*(?<value>\d+);?\s*[}]?"#,
    )
    .unwrap();

    for caps in cond_regex.captures_iter(s) {
        let key = caps["key"].parse::<u32>().unwrap();
        let value = caps["value"].parse::<u32>().unwrap();

        m.entry(key)
            .and_modify(|prev| *prev = value)
            .or_insert(value);
    }

    //

    let b_regex = Regex::new(r#"(?si)b:\s*["'](?<b>.+?)["']"#).unwrap();

    let b = &b_regex.captures(s)?["b"];

    Some(GG {
        m,
        b: b.strip_suffix('/').unwrap_or(b).to_owned(),
        default: default_value,
    })
}
