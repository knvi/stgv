use std::str::FromStr;

use image::{Pixel, RgbImage};
use itertools_num::linspace;
use structopt::StructOpt;

use crate::bit::{BitEncoder, BitDistribution};
use crate::StegError;

/// Supported steganography encoding algorithms
#[derive(StructOpt, Debug)]
pub enum StegMethod {
    /// Least significant bit encoding
    /// With a binary message, each bit of the message is encoded
    /// into the least significant bit of each RGB byte of each pixel.
    LeastSignificantBit,
    /// Random significant bit encoding
    /// With a binary message, each bit of the message is encoded
    /// randomly into one of the `n` least significant bits of each RGB byte of each pixel.
    RandomSignificantBit,
}

impl FromStr for StegMethod {
    type Err = String;
    fn from_str(method: &str) -> Result<Self, Self::Err> {
        match method {
            "lsb" => Ok(Self::LeastSignificantBit),
            "rsb" => Ok(Self::RandomSignificantBit),
            other => Err(format!("unknown encoding method: {}", other)),
        }
    }
}

const END: &[u8] = b"$TGV";

/// Trait to encode a message into an image and decode a message from an image.
pub trait Steganography {
    /// Encodes a message into an image.
    fn encode(&mut self, img: &RgbImage, msg: &[u8]) -> Result<RgbImage, StegError>;
    /// Decodes a message from an image.
    fn decode(&mut self, img: &RgbImage) -> Result<Vec<u8>, StegError>;
    /// Returns the maximum number of bytes that can be encoded into an image with the method implemented.
    fn max_bytes(&self, img: &RgbImage) -> usize;
}

impl Steganography for BitEncoder {
    fn max_bytes(&self, img: &RgbImage) -> usize {
        ((img.width() * img.height() * 3) as usize - (END.len() * 8)) / 8
    }

    fn encode(&mut self, img: &RgbImage, msg: &[u8]) -> Result<RgbImage, StegError> {
        let msg = if self.end_seq {
            [msg, END].concat()
        } else {
            msg.to_owned()
        };

        let mut binary_msg = String::with_capacity(msg.len() * 8);
        for byte in msg {
            binary_msg += &format!("{:08b}", byte);
        }
        let binary_msg: Vec<u8> = binary_msg
            .chars()
            .map(|c| c.to_digit(10).unwrap() as u8)
            .collect();

        let mut img = img.clone();

        // generate a linear distribution from 0th to last pixel, with (number of bits to encode / 3) inbetween
        // because in each pixel we encode 3 bits (rgb)
        let linspace_length = (binary_msg.len() as f64 / 3.).ceil() as usize;
        let linear_pixel_dist = get_linspace(
            0.,
            f64::from((img.width() * img.height()) - 1),
            linspace_length,
        );
        let mut linear_pixel_dist = linear_pixel_dist.iter();

        for (ctr, chunk) in binary_msg.chunks(3).enumerate() {
            let (x, y) = match self.bit_dist {
                BitDistribution::Sequential => {
                    let x = ctr as u32 % img.width();
                    let y = ctr as u32 / img.width();
                    (x, y)
                }
                BitDistribution::Linear { length: _ } => {
                    // SAFETY: unwrap as we create a linspace distribution based on the length of the message so we know
                    // there are enough pixels
                    let pixel_num = linear_pixel_dist.next().unwrap();
                    let x = *pixel_num as u32 % img.width();
                    let y = *pixel_num as u32 / img.width();
                    (x, y)
                }
            };
            let pixel = img.get_pixel_mut(x, y);
            for (idx, bit) in chunk.iter().enumerate() {
                self.encoder.encode(bit, &mut pixel[idx]);
            }
        }

        if let BitDistribution::Linear { length: _ } = self.bit_dist {
            println!(
                "Note: use length '{}' when decoding with linear distribution",
                linspace_length
            );
        }
        Ok(img)
    }

    fn decode(&mut self, img: &RgbImage) -> Result<Vec<u8>, StegError> {
        let mut bitstream: Vec<u8> = Vec::new();

        let mut endstream = String::new();
        for byte in END {
            endstream += &format!("{:08b}", byte);
        }

        let end = endstream
            .chars()
            .map(|c| c.to_digit(10).unwrap() as u8)
            .collect::<Vec<u8>>();

        match self.bit_dist {
            BitDistribution::Sequential => {
                'outer_seq: for (_, _, pixel) in img.enumerate_pixels() {
                    for value in pixel.channels() {
                        if has_end(&bitstream, &end) {
                            break 'outer_seq;
                        }
                        bitstream.push(self.encoder.decode(value));
                    }
                }
            }
            BitDistribution::Linear { length } => {
                let linear_pixel_dist =
                    get_linspace(0., f64::from((img.width() * img.height()) - 1), length);
                'outer_lin: for pixel_num in linear_pixel_dist {
                    let x = pixel_num as u32 % img.width();
                    let y = pixel_num as u32 / img.width();
                    let pixel = img.get_pixel(x, y);
                    for value in pixel.channels() {
                        if has_end(&bitstream, &end) {
                            break 'outer_lin;
                        }
                        bitstream.push(self.encoder.decode(value));
                    }
                }
            }
        }

        if self.end_seq {
            if !has_end(&bitstream, &end) {
                return Err(StegError::EncodingNotFound);
            }

            // message found in the bitstream, remove the END indicator
            bitstream.truncate(bitstream.len() - end.len());
        }
        let mut msg = Vec::new();
        for chrs in bitstream.chunks(8) {
            let binval = u8::from_str_radix(
                &chrs
                    .iter()
                    .map(|c| format! {"{}",c})
                    .collect::<String>(),
                2,
            )
            .map_err(|e| StegError::Decoding(format!("reconstructing byte: {}", e)))?;
            msg.push(binval);
        }
        Ok(msg)
    }
}

/// helper
pub fn has_end(bits: &[u8], end: &[u8]) -> bool {
    if bits.len() < end.len() {
        return false;
    }
    let start = bits.len() - end.len();
    bits[start..] == end[..]
}

/// linspace helper
pub fn get_linspace(a: f64, b: f64, n: usize) -> Vec<usize> {
    linspace(a, b, n)
        .map(|p| p.floor() as usize)
        .collect::<Vec<usize>>()
}