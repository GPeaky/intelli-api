use std::mem;

use error::{AppResult, F1ServiceError};

use crate::f1_structs::PacketHeader;

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

#[inline]
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
