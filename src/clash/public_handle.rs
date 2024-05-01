use std::str::FromStr;

use anyhow::anyhow;
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct PublicHandle(String);

impl FromStr for PublicHandle {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.chars().all(|ch| ch.is_ascii_hexdigit()) {
            Ok(PublicHandle(String::from(s)))
        } else {
            Err(anyhow!("valid handles only contain characters 0-9 and a-f"))
        }
    }
}

impl std::fmt::Display for PublicHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<'de> Deserialize<'de> for PublicHandle {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        FromStr::from_str(&s).map_err(serde::de::Error::custom)
    }
}
