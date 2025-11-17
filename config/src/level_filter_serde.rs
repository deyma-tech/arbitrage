use log::LevelFilter;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy)]
pub struct SerdeLevelFilter(pub LevelFilter);

impl Serialize for SerdeLevelFilter {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}

impl<'de> Deserialize<'de> for SerdeLevelFilter {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let level = s.parse::<LevelFilter>().map_err(serde::de::Error::custom)?;
        Ok(SerdeLevelFilter(level))
    }
}
