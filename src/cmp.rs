use compression::prelude::*;

use crate::StegError;

/// Compresses a slice of bytes into a new vec of bytes.
pub fn compress(data: &[u8]) -> Result<Vec<u8>, StegError> {
    data.iter()
        .cloned()
        .encode(&mut BZip2Encoder::new(9), Action::Finish)
        .collect::<Result<Vec<_>, _>>()
        .map_err(StegError::Compression)
}

/// Decompresses a slice of bytes into a new vec of bytes.
pub fn decompress(data: &[u8]) -> Result<Vec<u8>, StegError> {
    data.iter()
        .cloned()
        .decode(&mut BZip2Decoder::new())
        .collect::<Result<Vec<_>, _>>()
        .map_err(StegError::Decompression)
}