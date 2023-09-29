mod txb;

use std::{fs::File, path::PathBuf};

use clap::Parser;

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

    let out_file = if let Some(path) = args.output.take() {
        path
    } else {
        args.input.with_extension("png")
    };

    let out_file = File::create(out_file).expect("Failed to create output file");
    let (width, height, image_data) = txb::parse_to_linear(&input);

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
