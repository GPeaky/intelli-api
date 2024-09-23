use crate::{
    error::{AppResult, F1ServiceError},
    structs::PacketHeader,
};
pub(crate) use ids_generator::IdsGenerator;
pub(crate) use password_hash::*;
pub(crate) use ports::MachinePorts;

use postgres_types::ToSql;
use serde::{Deserialize, Deserializer};
use std::mem;

mod bitset;
mod ids_generator;
mod password_hash;
mod ports;

pub fn deserialize_i64_from_string<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    s.parse().map_err(serde::de::Error::custom)
}

pub fn header_cast(bytes: &[u8]) -> AppResult<&PacketHeader> {
    if mem::size_of::<PacketHeader>() > bytes.len() {
        Err(F1ServiceError::CastingError)?;
    }

    // SAFETY:
    // - We've verified there are enough bytes for T.
    // - The structure is packed, so there are no alignment requirements.
    // - We assume the data is little-endian (valid for F1 2023/2024 on player PCs).
    // - We're only performing reads, no writes
    Ok(unsafe { &*(bytes.as_ptr() as *const PacketHeader) })
}

#[inline(always)]
pub fn cast<T>(bytes: &[u8]) -> AppResult<&T> {
    if !mem::size_of::<T>() == bytes.len() {
        Err(F1ServiceError::CastingError)?;
    }

    // SAFETY:
    // - We've verified there are enough bytes for T.
    // - The structure is packed, so there are no alignment requirements.
    // - We assume the data is little-endian (valid for F1 2023/2024 on player PCs).
    // - We're only performing reads, no writes
    Ok(unsafe { &*(bytes.as_ptr() as *const T) })
}

#[inline(always)]
pub fn slice_iter<'a>(
    s: &'a [&'a (dyn ToSql + Sync)],
) -> impl ExactSizeIterator<Item = &'a dyn ToSql> + 'a {
    s.iter().map(|s| *s as _)
}
