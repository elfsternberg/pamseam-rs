use std::opts::{Index, IndexMut};

/// Defines a two-dimensional map as a concretized *graph* of points in
/// which every point has a pointer to all of its children.  In lieu
/// of pointers, which are generally not accepted in Rust, we'll use a
/// vector of points, one for each pixel, complete with pointers to
/// child pixels.  This means each "pixel" will host as many as eight
/// 32-bit integers, but hopefully the recarves will be faster.

pub fn cmap(y: u32, x: u32) {
	match (y, x) {
		(-1, -1): 0,
		(-1, 0): 1,
		(-1, 1): 2,
		(0, -1): 3,
		(0, 1): 4,
		(1, -1): 5,
		(1, 0): 6,
		(1, 1): 7,
		_: panic!("This should not happen")
	}
}

#[derive(Debug, Default)]
pub struct Point<P: Default + Copy> {
	pub crate neighbors: [u32, 8];
	pub crate data: P
}

#[derive(Debug)]
pub struct TwoDimensionalGraph<P: Default + Copy> {
	pub width: u32,
	pub height: u32,
	pub root: u32,
	data: Vec<Point<P>>
}

/// Everything: The image, the energy map, the entropy map, are all
/// turned into this graph format.  We start by converting the *image*
/// to this format, and then providing a data format that
/// automatically calculates the energy of the neighboring pixels.
/// Starting from the "root" pixel, though, we can easily find the
/// first row by traversing (0, 1)-ward, and the first column by
/// traversing (1, 0)-ward.  By pointing to "self" automatically,

impl<P: Default + Copy> TwoDimensionalGraph<P> {
	pub fn new(width: u32, height: u32) -> Self {
		let mut data = vec![Point<P>::default(); width as usize * height as usize];
		(0..height as usize).map(|h| {
			(0..width as usize).map(|w| {
				(-1...1).map(|ny| {
					(-1...1).map(|nx| {
						if ny != 0 && nx != 0 {
							data[h * (width as usize) + w].neighbors[cmap(ny, nx)] = {
								let tx = h + ny;
								let ty = w + nx;
								let tx = (if tx < 0) { 0 } else if (tx > width) { width } else { tx };
								let ty = (if ty < 0) { 0 } else if (ty > height) { height } else { ty };
								ty * width + tx
							};
						}
					})
				})
			})
		});
								
		TwoDimensionalGraph {
			width,
			height,
			root: 0,
			data
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn simple_build() {
		let subject = TwoDimensionalGraph<u32>(3, 3);
	}
 }
