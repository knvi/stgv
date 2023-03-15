#![warn(clippy::all, clippy::pedantic, clippy::nursery)]

pub mod steg;
pub mod bit;
pub mod cmp;
pub mod cli;
pub mod exec;

use thiserror::Error;

#[derive(Error, Debug, Eq, PartialEq)]
pub enum StegError {
    #[error("Encoded message not found in data")]
    EncodingNotFound,
    #[error("Error decoding message: `{0}`")]
    Decoding(String),
    #[error("Compression error")]
    Compression(#[from] compression::prelude::CompressionError),
    #[error("Decompression error")]
    Decompression(#[from] compression::prelude::BZip2Error),
}