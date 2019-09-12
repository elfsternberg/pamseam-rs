use pnmseam::{compute_energy, energy_to_image};
use std::fs;
use std::io;
use std::io::BufReader;

extern crate clap;
extern crate image;

use clap::{App, Arg};
use image::pnm::{PNMEncoder, PNMSubtype, SampleEncoding};
use image::{load, ColorType, GenericImageView, ImageFormat};

fn main() {
    let matches = App::new("pnmseam")
        .version("0.1.0")
        .author("Elf M. Sternberg <elf.sternberg@gmail.com>")
        .about("Seam carving for portable anymap")
        .arg(
            Arg::with_name("pnmfile")
                .help("The image to convert")
                .required(true)
                .index(1),
        )
        .get_matches();

    let rdr = BufReader::new(fs::File::open(matches.value_of("pnmfile").unwrap()).unwrap());
    let image = load(rdr, ImageFormat::PNM).unwrap();
    let energy = compute_energy(&image);
    let (width, height) = image.dimensions();
    let newmap = energy_to_image(&energy, width, height);

    PNMEncoder::new(io::stdout())
        .with_subtype(PNMSubtype::Graymap(SampleEncoding::Binary))
        .encode(
            newmap.into_flat_samples().as_slice(),
            width,
            height,
            ColorType::Gray(8),
        )
        .unwrap();
}
