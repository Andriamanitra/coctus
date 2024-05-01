use std::str::FromStr;

use anyhow::anyhow;
use serde::{Deserialize, Deserializer, Serialize};

/// `PublicHandle` is a hexadecimal string that uniquely identifies a clash
/// or a puzzle. It is the last part of the URL when viewing a clash or a puzzle
/// on the CodinGame contribution page.
///
/// # Examples
///
/// ```
/// use clashlib::clash::PublicHandle;
/// use std::str::FromStr;
///
/// let handle = PublicHandle::from_str("682102420fbce0fce95e0ee56095ea2b9924");
/// assert!(handle.is_ok());
/// let invalid_handle = PublicHandle::from_str("xyz");
/// assert!(invalid_handle.is_err());
/// ```
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
