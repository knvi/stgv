use std::{convert::From, str::FromStr};
use structopt::StructOpt;
use rand::Rng;
use rand_pcg::Pcg64;
use rand_seeder::Seeder;

/// Supported bit encoding bit distribution methods
#[derive(StructOpt, Debug)]
pub enum BitDistribution {
    /// Encode bits sequentially into the image starting from top-left
    Sequential,
    /// Evenly space out the bits in the image so not all packed into top-left
    Linear { length: usize },
}

impl FromStr for BitDistribution {
    type Err = String;
    fn from_str(method: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = method.split('-').collect();
        let method = if parts.len() <= 1 { method } else { parts[0] };
        match method {
            "sequential" => Ok(Self::Sequential),
            "linear" => {
                let length = *(parts.get(1).unwrap_or(&"0"));
                let length = length.parse::<usize>().unwrap_or_else(|err| {
                    eprintln!(
                        "error parsing message length in linear bit distribution: {}",
                        err
                    );
                    std::process::exit(1);
                });
                Ok(Self::Linear { length })
            }
            other => Err(format!("unknown bit distribution {}", other)),
        }
    }
}

impl Default for BitDistribution {
    fn default() -> Self {
        BitDistribution::Sequential
    }
}

/// Bit masks for setting or clearing bits in bytes.
#[derive(Clone)]
enum BitMask {
    One = 0b0000_0001,
    Two = 0b0000_0010,
    Four = 0b0000_0100,
    Eight = 0b0000_1000,
}

impl From<u8> for BitMask {
    fn from(n: u8) -> Self {
        match n {
            1 => BitMask::One,
            2 => BitMask::Two,
            4 => BitMask::Four,
            8 => BitMask::Eight,
            other => panic!("cannot create bitmask from val {other}"),
        }
    }
}

/// Trait for encoding a single bit of information into a byte.
pub trait BitEncode {
    /// Encode a bit of information into a byte.
    fn encode(&mut self, byte: &u8, color_val: &mut u8);
    /// Decode a bit of information from a byte.
    fn decode(&mut self, color_val: &u8) -> u8;
}

/// Implementation of BitEncode
pub struct BitEncoder {
    pub encoder: Box<dyn BitEncode>,
    /// Bit distribution method
    pub bit_dist: BitDistribution,
    /// Add a token sequence to the end of encoding
    pub end_seq: bool,
}

impl BitEncoder {
    pub fn new(encoder: Box<dyn BitEncode>, bd: Option<BitDistribution>) -> Self {
        BitEncoder {
            encoder,
            bit_dist: bd.unwrap_or_default(),
            end_seq: true,
        }
    }
}

/// LSB
/// With a binary message, each bit of the message is encoded
/// into the least significant bit of each RGB byte of each pixel.

#[derive(Default)]
pub struct Lsb;

impl BitEncode for Lsb {
    fn encode(&mut self, bit: &u8, color_val: &mut u8) {
        if *bit == 0 {
            *color_val &= !(BitMask::One as u8);
        } else if *bit == 1 {
            *color_val |= BitMask::One as u8;
        }
    }

    fn decode(&mut self, color_val: &u8) -> u8 {
        color_val & BitMask::One as u8
    }
}

/// RSB
/// With a binary message, each bit of the message is encoded
/// randomly into one of the `n` least significant bits of each RGB byte of each pixel.

pub struct Rsb {
    /// The maximum significant bit to possibly set/clear when encoding (1-4)
    max: u8,
    /// A seeded random number generator do determine which significant bit to encode to/decode from
    rng: Pcg64,
}

impl Rsb {
    pub fn new(max: u8, seed: &str) -> Self {
        Rsb {
            max,
            rng: Seeder::from(seed).make_rng(),
        }
    }

    /// Returns a random number between 1 and `max` inclusive.
    /// Used to determine which bit to encode to/decode from.
    fn rand(&mut self) -> BitMask{
        let n: u8 = self.rng.gen_range(1..=self.max);
        BitMask::from(n)
    }
}

impl BitEncode for Rsb {
    fn encode(&mut self, bit: &u8, color_val: &mut u8) {
        let mask = self.rand();
        if *bit == 0 {
            *color_val &= !(mask as u8);
        } else if *bit == 1 {
            *color_val |= mask as u8;
        }
    }

    fn decode(&mut self, color_val: &u8) -> u8 {
        let mask = self.rand();
        let c = color_val & mask as u8;
        if c > 0 {
            1
        } else {
            0
        }
    }
}