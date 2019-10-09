// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Image dimensional flipper

//! A utility proxy for the ImageRS "Image" trait that maps the width
//! to the original height, and vice versa, as well as every x to y
//! and vice versa.
//!
//! This has the effect of making it possible to treat each column in a
//! horizontal scan as a continguous block, that is, the way a row is
//! usually represented in memory.
//!
//! These algorithms treat the image as a read-only source, and the
//! intermediate products can have any representation sufficient to
//! perform the necessary computation.  By virtually "flipping" the
//! image 90°, we can break the receiving "row" by chunks_mut and
//! update each chunk in a separate thread without having to do
//! anything unsafe.

use image::{GenericImageView, Pixel, Primitive};

pub struct Flipper<'a, I, P, S>
where
    I: GenericImageView<Pixel = P>,
    P: Pixel<Subpixel = S> + 'static,
    S: Primitive + 'static,
{
    pub image: &'a I,
}

impl<'a, I, P, S> GenericImageView for Flipper<'a, I, P, S>
where
    I: GenericImageView<Pixel = P>,
    P: Pixel<Subpixel = S> + 'static,
    S: Primitive + 'static,
{
    type Pixel = P;
    type InnerImageView = I;

    fn dimensions(&self) -> (u32, u32) {
        let (x, y) = self.image.dimensions();
        (y, x)
    }

    fn width(&self) -> u32 {
        self.image.height()
    }

    fn height(&self) -> u32 {
        self.image.width()
    }

    fn get_pixel(&self, x: u32, y: u32) -> P {
        self.image.get_pixel(y, x)
    }

    fn inner(&self) -> &Self::InnerImageView {
        self.image
    }

    fn bounds(&self) -> (u32, u32, u32, u32) {
        let (x1, y1, x2, y2) = self.image.bounds();
        (y1, x1, y2, x2)
    }
}
