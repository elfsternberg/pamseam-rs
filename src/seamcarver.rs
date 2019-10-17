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

use crate::avisha2::AviShaTwo;
use crate::seamfinder::SeamFinder;
use image::{GenericImageView, ImageBuffer, Pixel, Primitive};

// The one tiny inefficiency here is that the seam is copied, into the
// new image, and then the path of pixels immediately to the right of
// the seam are copied over it.
fn remove_vertical_seam<I, P, S>(image: &I, seam: &[u32]) -> ImageBuffer<P, Vec<S>>
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
fn remove_horizontal_seam<I, P, S>(image: &I, seam: &[u32]) -> ImageBuffer<P, Vec<S>>
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

// This is silly and basically a reimplementation of `bool` and `not`,
// but it makes it much clearer in the code what I'm doing.  And I
// like that.

#[derive(PartialEq, Copy, Clone)]
enum Carve {
    Width,
    Height,
}

impl Carve {
    fn turn(self) -> Self {
        if self == Carve::Width {
            Carve::Height
        } else {
            Carve::Width
        }
    }
}

fn carveonce<I, P, S>(image: &I, direction: Carve) -> ImageBuffer<P, Vec<S>>
where
    I: GenericImageView<Pixel = P>,
    P: Pixel<Subpixel = S> + 'static,
    S: Primitive + 'static,
{
    if direction == Carve::Height {
        let carver = AviShaTwo::new(image);
        let seam = carver.find_horizontal_seam();
        remove_horizontal_seam(image, &seam)
    } else {
        let carver = AviShaTwo::new(image);
        let seam = carver.find_vertical_seam();
        println!("{:?}", seam);
        remove_vertical_seam(image, &seam)
    }
}

// It isn't necessary at this point to be using a struct-based
// implementation, but it lays the groundwork for caching intermediate
// results.  I have no idea if those results will be better than
// rebuilding the whole energy map, but it's the start.

/// A struct for holding the image to be carved.
pub struct SeamCarver<'a, I, P, S>
where
    I: GenericImageView<Pixel = P>,
    P: Pixel<Subpixel = S> + 'static,
    S: Primitive + 'static,
{
    image: &'a I,
}

impl<'a, I, P, S> SeamCarver<'a, I, P, S>
where
    I: GenericImageView<Pixel = P>,
    P: Pixel<Subpixel = S> + 'static,
    S: Primitive + 'static,
{
    /// Creates a new SeamCarver with an image to be carved.
    pub fn new(image: &'a I) -> Self {
        Self { image }
    }

    // This is absurdly inefficient, as the entire energy map and
    // energy seam digraph is recalculated every time.  It should be
    // possible to find the span of columns or rows affected by the
    // carve and recalculate only the new ones.

    /// Given an image and a desired new width and height, repeatedly carve
    /// seams out of the image.
    pub fn carve(&self, newwidth: u32, newheight: u32) -> Result<ImageBuffer<P, Vec<S>>, String> {
        let (mut width, mut height) = self.image.dimensions();
        if width < newwidth || height < newheight {
            return Err("seamcarve cannot upscale an image".to_string());
        }
        let mut direction = Carve::Width;
        let mut scratch = ImageBuffer::<P, Vec<S>>::new(width, height);

        // Initialize the scratch space.
        self.image.pixels().for_each(|p| scratch[(p.0, p.1)] = p.2);

        while width > newwidth && height > newheight {
            scratch = carveonce(&scratch, direction);
            direction = direction.turn();
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
}
