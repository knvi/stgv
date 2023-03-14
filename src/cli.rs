use anyhow::{bail, Result};
use std::path::PathBuf;
use structopt::StructOpt;

use crate::bit::BitDistribution;
use crate::steg::StegMethod;

#[derive(StructOpt)]
#[structopt(
    name = "stgv",
    version = "0.1.0",
    about = "A steganography tool for images.",
    author = "by knvi"
)]
pub struct CLI {
    /// Decode a message from an image
    #[structopt(short, long)]
    pub decode: bool,

    /// Compress/decompress data
    #[structopt(short, long)]
    pub compress: bool,

    /// Check max message size that can be encoded with options given.
    #[structopt(short = "C", long)]
    pub check_max_length: bool,

    /// Method to use for encoding (lsb,rsb)
    #[structopt(short, long, default_value = "lsb")]
    pub method: StegMethod,

    /// Method for bit distribution (sequential, linear (linear-N when decoding))
    #[structopt(long, default_value = "sequential")]
    pub distribution: BitDistribution,

    /// Seed for random significant bit encoding
    #[structopt(short, long, required_if("method", "rsb"))]
    pub seed: Option<String>,

    /// Maximum bit to possible modify (1-4)
    #[structopt(short = "N", long, required_if("method", "rsb"))]
    pub max_bit: Option<u8>,

    /// Output file, stdout if not present
    #[structopt(short, long, parse(from_os_str))]
    pub output: Option<PathBuf>,

    /// Input file to encode, stdin if not present
    #[structopt(short, long, parse(from_os_str), conflicts_with = "decode")]
    pub input: Option<PathBuf>,

    /// Input image
    #[structopt(parse(from_os_str))]
    pub image: PathBuf,
}

impl CLI {
    pub fn validate(&self) -> Result<()> {
        if let Some(n) = self.max_bit {
            if !(1..=4).contains(&n) {
                bail!(format!("max-bit must be between 1-4. Got {}", n))
            }
        }
        Ok(())
    }
}
