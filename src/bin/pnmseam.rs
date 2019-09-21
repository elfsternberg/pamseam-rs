use pnmseam::seamcarve;
use std::io;

extern crate clap;
extern crate image;

use clap::{App, Arg};
use image::ColorType;

fn main() {
    let matches = App::new("pnmseam")
        .version("0.1.0")
        .author("Elf M. Sternberg <elf.sternberg@gmail.com>")
        .about("Seam carving for portable anymap")
        .arg(
            Arg::with_name("imagefile")
                .help("The image to convert")
                .required(true)
                .index(1),
        )
        .get_matches();

    let image = image::open(matches.value_of("imagefile").unwrap()).unwrap();
    let newimage = seamcarve(&image, 896, 1079).unwrap();
    newimage.save("test-resize.png").unwrap();
}
