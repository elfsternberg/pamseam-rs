// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Seamcarve - The main function
//!
//! The main seamcarver routine, with helpers for the horizontal and
//! vertical operations.

// TODO: The two ops are so damn close to each other in implementation
// that I have trouble believing I can't create an abstraction for it.
// But maybe it's faster and clearer this way.  Besides, we know that
// the horizontal seams will give us nightmares when we start trying
// to multithread this beast.

use crate::energy::{calculate_horizontal_seam, calculate_vertical_seam};
use image::{GenericImageView, ImageBuffer, Pixel, Primitive};

// The one tiny inefficiency here is that the seam is copied, into the
// new image, and then the path of pixels immediately to the right of
// the seam are copied over it.
fn remove_vertical_seam<I, P, S>(image: &I, seam: &Vec<u32>) -> ImageBuffer<P, Vec<S>>
where
    I: GenericImageView<Pixel = P>,
    P: Pixel<Subpixel = S> + 'static,
    S: Primitive + 'static,
{
    let (width, height) = image.dimensions();
    let mut imgbuf = image::ImageBuffer::new(width - 1, height);
    for y in 0..height {
        for x in 0..width {
            let pixel = image.get_pixel(x, y);
            imgbuf.put_pixel(
                if x < seam[y as usize] || x == 0 {
                    x
                } else {
                    x - 1
                },
                y,
                pixel,
            );
        }
    }
    imgbuf
}

// The one tiny inefficiency here is that the seam is copied, into the
// new image, and then the path of pixels immediately below the seam
// are copied over it.
fn remove_horizontal_seam<I, P, S>(image: &I, seam: &Vec<u32>) -> ImageBuffer<P, Vec<S>>
where
    I: GenericImageView<Pixel = P>,
    P: Pixel<Subpixel = S> + 'static,
    S: Primitive + 'static,
{
    let (width, height) = image.dimensions();
    let mut imgbuf = image::ImageBuffer::new(width, height - 1);
    for y in 0..height {
        for x in 0..width {
            let pixel = image.get_pixel(x, y);
            imgbuf.put_pixel(
                x,
                if y < seam[x as usize] || y == 0 {
                    y
                } else {
                    y - 1
                },
                pixel,
            );
        }
    }
    imgbuf
}

#[derive(PartialEq, Copy, Clone)]
enum Carve {
    Width,
    Height,
}

fn carveonce<I, P, S>(image: &I, direction: Carve) -> ImageBuffer<P, Vec<S>>
where
    I: GenericImageView<Pixel = P>,
    P: Pixel<Subpixel = S> + 'static,
    S: Primitive + 'static,
{
    if direction == Carve::Height {
        let seam = calculate_horizontal_seam(image);
        remove_horizontal_seam(image, &seam)
    } else {
        let seam = calculate_vertical_seam(image);
        remove_vertical_seam(image, &seam)
    }
}

/// Given an image and a desired new width and height, repeatedly carve
/// seams out of the image.  This is absurdly inefficient, as the
/// entire energy map and energy seam digraph is recalculated every
/// time.  It should be possible to find the span of columns or rows
/// affected by the carve and recalculate only the new ones.
pub fn seamcarve<I, P, S>(
    image: &I,
    newwidth: u32,
    newheight: u32,
) -> Result<ImageBuffer<P, Vec<S>>, String>
where
    I: GenericImageView<Pixel = P>,
    P: Pixel<Subpixel = S> + 'static,
    S: Primitive + 'static,
{
    let (mut width, mut height) = image.dimensions();
    if width < newwidth || height < newheight {
        return Err("seamcarve cannot upscale an image".to_string());
    }

    let mut direction = Carve::Width;
    let mut scratch = ImageBuffer::<P, Vec<S>>::new(width, height);
    for p in image.pixels() {
        scratch[(p.0, p.1)] = p.2.clone()
    }

    while width > newwidth && height > newheight {
        scratch = carveonce(&scratch, direction);
        direction = if direction == Carve::Height {
            Carve::Width
        } else {
            Carve::Height
        };
        width = scratch.width();
        height = scratch.height();
        println!("B: {}, {}", width, height);
    }

    while width > newwidth {
        scratch = carveonce(&scratch, Carve::Width);
        width = scratch.width();
        println!("W: {}, {}", width, height);
    }

    while height > newheight {
        scratch = carveonce(&scratch, Carve::Height);
        height = scratch.height();
        println!("H: {}, {}", width, height);
    }

    Ok(scratch)
}
