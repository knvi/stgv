use std::fs::File;
use std::io::{stdin, stdout, Read, Write};
use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use image::io::Reader;
use pretty_bytes::converter::convert;
use tabled::Table;
use atty::Stream;

use crate::bit::{BitEncoder, Rsb, Lsb};
use crate::cmp::{decompress, compress};
use crate::{cli, steg};
use crate::steg::Steganography;

fn load_rgb8_img(path: &PathBuf) -> Result<image::RgbImage> {
    let img = Reader::open(path)
        .context(format!("opening {:?}", path))?
        .decode()?;
    Ok(img.into_rgb8())
}

/// Executes the steganography from given cli options.
pub fn run(opt: cli::CLI) -> Result<()> {
    let rgb8_img = load_rgb8_img(&opt.image)?;

    // create an encoder
    let mut encoder: Box<dyn Steganography> = match opt.method {
        steg::StegMethod::LeastSignificantBit => {
            let lsb = Box::new(Lsb::default());
            Box::new(BitEncoder::new(lsb, Some(opt.distribution)))
        }
        steg::StegMethod::RandomSignificantBit => {
            let rsb = Box::new(Rsb::new(opt.max_bit.unwrap(), &(opt.seed.unwrap())));
            Box::new(BitEncoder::new(rsb, Some(opt.distribution)))
        }
    };

    let max_msg_len = encoder.max_bytes(&rgb8_img);
    
    if opt.check_max_length {
        let tab = Table::new(vec![
            ("Image", opt.image.to_str().unwrap()),
            ("Encoding Method", &format!("{:?}", opt.method)),
            ("Max Message Length", &convert(max_msg_len as f64)),
        ])
        .with(tabled::Style::blank())
        .with(tabled::Disable::Row(..1))
        .with(tabled::Modify::new(tabled::object::Segment::all()).with(tabled::Alignment::left()))
        .to_string();
        println!("{}", tab);
        return Ok(());
    }

    if opt.decode {
        let mut result = encoder
            .decode(&rgb8_img)
            .context("failed to decode message from image")?;

        if opt.compress {
            result = decompress(&result)?;
        }

        if let Some(path) = opt.output {
            let mut f = File::create(&path)
                .context(format!("Failed to create file: {}", path.to_str().unwrap()))?;

            f.write_all(&result)
                .context(format!("Failed to write to file: {}", path.to_str().unwrap()))?;
        } else {
            let res = match String::from_utf8(result.clone()) {
                Ok(s) => s,
                Err(_) => unsafe { String::from_utf8_unchecked(result) },
            };
            println!("{}", res);
        }
    } else {
        // read msg to encode to image from stdin/file

        let mut msg = match &opt.input {
            Some(path) => {
                let mut f = File::open(path)
                    .context(format!("Failed to open file: {}", path.to_str().unwrap()))?;

                let mut buf = Vec::new();
                f.read_to_end(&mut buf)
                    .context(format!("Failed to read file: {}", path.to_str().unwrap()))?;

                buf
            }
            None => {
                let mut buf = Vec::new();
                if atty::is(Stream::Stdin) {
                    print!("Enter what you want to encode. ");

                    let _ = stdout().flush();

                    let mut str_buf = String::new();
                    stdin().read_line(&mut str_buf)?;
                    buf = str_buf.into_bytes();

                } else {
                    stdin().read_to_end(&mut buf)?;
                }
                buf
            }
        };

        if opt.compress {
            msg = compress(&msg)?;
        }

        // CHECK IF THE MESSAGE IS TOO LONG
        if msg.len() > max_msg_len {
            bail!(
                "Message length is too long. It exceeds capacity that can fit in the image supplied. {} > {}.
                Try using the compression flag, or using either larger images or less data",
                convert(msg.len() as f64),
                convert(max_msg_len as f64)
            );
        }

        // Encode
        let res = encoder
            .encode(&rgb8_img, &msg)
            .context("failed to encode message to image")?;

        match opt.output {
            Some(path) => {
                res.save(path.clone())
                    .context(format!("saving image to {:?}", path))?;
            }
            None => {
                let mut out = std::io::stdout();
                out.write_all(res.as_raw())?;
                out.flush()?;
            }
        }

    }

    Ok(())
}