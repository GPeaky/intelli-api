use std::time::{SystemTime, UNIX_EPOCH};

pub use ports::MachinePorts;
use postgres_types::ToSql;
use serde::{Deserialize, Deserializer};

mod ports;

pub fn deserialize_i64_from_string<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    s.parse().map_err(serde::de::Error::custom)
}

#[inline]
pub fn slice_iter<'a>(
    s: &'a [&'a (dyn ToSql + Sync)],
) -> impl ExactSizeIterator<Item = &'a dyn ToSql> + 'a {
    s.iter().map(|s| *s as _)
}

pub fn current_timestamp_s() -> u32 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time got back")
        .as_secs() as u32
}
