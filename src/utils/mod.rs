use crate::error::{AppResult, F1ServiceError};
pub(crate) use ids_generator::IdsGenerator;
pub(crate) use password_hash::*;
pub(crate) use ports::MachinePorts;

use brotli::CompressorWriter;
use ntex::util::Bytes;
use postgres_types::ToSql;
use std::{io::Write, mem};

mod ids_generator;
mod password_hash;
mod ports;

#[inline(always)]
pub fn cast<T>(bytes: &[u8]) -> AppResult<&T> {
    if bytes.len() < mem::size_of::<T>() {
        Err(F1ServiceError::CastingError)?;
    }

    let ptr = bytes.as_ptr();
    let alignment = mem::align_of::<T>();

    if (ptr as usize) % alignment != 0 {
        panic!(
            "Error: Unable to cast because the alignment of type '{}' is {} bytes, but the pointer address is not properly aligned.",
            std::any::type_name::<T>(),
            alignment
        );
    }

    Ok(unsafe { &*(ptr.cast()) })
}

#[inline]
pub fn slice_iter<'a>(
    s: &'a [&'a (dyn ToSql + Sync)],
) -> impl ExactSizeIterator<Item = &'a dyn ToSql> + 'a {
    s.iter().map(|s| *s as _)
}

#[inline(always)]
pub async fn compress_async(data: Bytes) -> AppResult<Bytes> {
    ntex::rt::spawn_blocking(move || compress(&data)).await?
}

#[inline(always)]
fn compress(data: &[u8]) -> AppResult<Bytes> {
    let mut compressed_data = Vec::with_capacity(data.len());

    {
        let mut compressor = CompressorWriter::new(&mut compressed_data, 4096, 3, 22);
        compressor.write_all(data).unwrap();
    }

    Ok(Bytes::from(compressed_data))
}
