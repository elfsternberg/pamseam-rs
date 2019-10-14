// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Calculate the energy of a pixel pair
//!
//! Given two pixels, the energy between them is the
//! relative distance between the colors that make them
//! up.  Several algorithms have been provided here,
//! from the classic d(R^2) + d(G^2) + d(B^2) to a
//! simple convert-to-grayscale and d(L^2).

use image::{Pixel, Primitive};
use num_traits::NumCast;

/// The type signature of our energy pair function.
pub type PixelPair<P> = dyn Fn(&P, &P) -> u32;

/// (Pixel, Pixel) -> Energy
///
/// Given a pair of pixels, calculate the energy between them.  This
/// variant uses the lumacolor channel.
#[inline]
pub fn energy_of_pair_luma<P, S>(p1: &P, p2: &P) -> u32
where
	P: Pixel<Subpixel = S> + 'static,
	S: Primitive + 'static,
{
	#[inline]
	fn lumachannel<S, P>(p: &P) -> u32
	where
		P: Pixel<Subpixel = S> + 'static,
		S: Primitive + 'static,
	{
		let c = p.to_luma().channels().to_owned();
		NumCast::from(c[0]).unwrap()
	}

	let css = lumachannel(p1) - lumachannel(p2);
	css * css
}
