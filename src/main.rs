mod tx;
mod txb;

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
