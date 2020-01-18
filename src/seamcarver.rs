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

use image::{GenericImageView, ImageBuffer, Pixel, Pixels, Primitive};
use seam_lattice::{SeamLattice, SeamLatticeScanner, Walker};

#[derive(Copy, Clone)]
pub(crate) struct Ixel<P: Pixel>
where
	<P as Pixel>::Subpixel: 'static,
{
	pixel: P,
	energy: u64,
	total: u64,
	backpointer: u32,
}

/// Turn Pixels into Ixels
pub(crate) struct PixelsToIxels<'a, I, P, S>
where
	I: GenericImageView<Pixel = P>,
	P: Pixel<Subpixel = S> + Default + 'static,
	S: Primitive + Default + 'static,
{
	pixels: Pixels<'a, I>,
}

impl<'a, I, P, S> Iterator for PixelsToIxels<'a, I, P, S>
where
	I: GenericImageView<Pixel = P>,
	P: Pixel<Subpixel = S> + Default + 'static,
	S: Primitive + Default + 'static,
{
	type Item = Ixel<P>;

	#[inline(always)]
	fn next(&mut self) -> Option<Ixel<P>> {
		match self.pixels.next() {
			None => None,
			Some(p) => Some(Ixel {
				pixel: p.2,
				energy: 0,
				total: 0,
				backpointer: 0,
			}),
		}
	}
}

// Consumes a concrete iterator over the pixels, and

/// Right now, takes an image, returns an image. Woo.
pub fn seamcarve<I, P, S>(
	image: &I,
	newwidth: u32,
	newheight: u32,
) -> Result<ImageBuffer<P, Vec<S>>, String>
where
	I: GenericImageView<Pixel = P>,
	P: Pixel<Subpixel = S> + 'static,
	S: Primitive + Default + 'static,
{
	let (width, height) = image.dimensions();
	let mut pixels = image.pixels();
	let lattice = SeamLattice::new(width, height, &mut pixels);

	let mut scratch = ImageBuffer::<P, Vec<S>>::new(width, height);
	let mut walker = Walker::new(Box::new(SeamLatticeScanner::new(&lattice)));
	let mut p: u32 = 0;
	while let Some(v) = walker.next(&lattice) {
		scratch.put_pixel(p % width, p / width, (*v).2);
		p += 1;
	}
	Ok(scratch)
}
