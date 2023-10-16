mod tx;
mod txb;

use bytes::Buf;
use clap::Parser;
use std::{ffi::OsStr, fs::File, path::PathBuf};

#[derive(Parser, Debug)]
struct Cli {
    input: PathBuf,
    output: Option<PathBuf>,
}

fn main() {
    let mut args: Cli = Cli::parse();
    let input = std::fs::read(&args.input).expect(&format!(
        "Failed to read input file: {}",
        args.input.display()
    ));

    let out_file = match args.output.take() {
        Some(path) => File::create(path),
        None => File::create(args.input.with_extension("png")),
    }
    .expect("Failed to create output file");

    let (width, height, image_data) = match args.input.extension().and_then(OsStr::to_str) {
        Some("5txb") => txb::convert(&input),
        Some("5tx") => tx::convert(&input),
        _ => panic!("Unsupported file format"),
    };

    let mut png_writer = create_png_writer(out_file, width as u32, height as u32);
    png_writer.write_image_data(&image_data).unwrap();
    png_writer.finish().unwrap();
}

fn create_png_writer(file: File, width: u32, height: u32) -> png::Writer<File> {
    let mut encoder = png::Encoder::new(file, width, height);

    encoder.set_color(png::ColorType::Rgb);
    encoder.set_depth(png::BitDepth::Eight);

    encoder.write_header().unwrap()
}

fn extract_palette(mut slice: &[u8], palette_size: usize) -> Vec<[u8; 3]> {
    let mut pal = Vec::with_capacity(palette_size);

    for _ in 0..palette_size {
        let col = slice.get_u16_le();
        let mut r = ((col & 0x1F) * 8) as u8;
        let mut g = (((col & 0x3E0) >> 5) * 8) as u8;
        let mut b = (((col & 0x7C00) >> 10) * 8) as u8;

        r = r + r / 32;
        g = g + g / 32;
        b = b + b / 32;

        pal.push([r, g, b]);
    }

    pal
}
