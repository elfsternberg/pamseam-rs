// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Calculate the energy of an image using the Avidan & Shamir
//! "Forward Energy" algorithm.
//!
//! Given an image, calucate the energy map and either a horizontal or
//! vertical seam for that image.  Currently uses the most
//! straightforward of the energy map algorithms, the one with no
//! forward energy calculation, although that is coming.

use crate::flipper::Flipper;
use crate::pixelpairs::energy_of_pair_luma as energy_of_pixel_pair;
use crate::seamfinder::SeamFinder;
use crate::twodmap::{EnergyAndBackPointer, TwoDimensionalMap};
use crossbeam;
use crossbeam::thread::ScopedJoinHandle;
use num_cpus;

use image::{GenericImageView, Pixel, Primitive};

type EnergyMap = TwoDimensionalMap<EnergyAndBackPointer<u32>>;

// 1. Given a pixel coordinate *not* in the first row,
// 2. There exist three possible seams to which that pixel contributes,
// 3. Calculate the cost of reaching this pixel given those three seams
// 4. And return the pair of (lowest cost, which parent)
//
// Standard differences:
//
//  CL(x,y) = D[(x−1,y),(x+1,y)]+D[(x,y−1),(x−1,y)]
//  CU(x,y) = D[(x−1,y),(x+1,y)]
//  CR(x,y) = D[(x−1,y),(x+1,y)]+D[(x,y−1),(x+1,y)]
//
// Pixels on the top row:
//
// CL(x,0) = 0
// CU(x,0) = D[(x−1,0),(x+1,0)]
// CR(x,0) = 0
//
// Edges:
//
// Near edge:
// CL(0,y)=D[(0,y),(1,y)]+D[(0,y−1),(0,y)]
// CU(0,y)=D[(0,y),(1,y)]
// CR(0,y)=D[(0,y),(1,y)]+D[(0,y−1),(1,y)]
//
// The far edge is handled by analogy.
//
// The energy for a specific pixel is therefore:
//
//           ⎧ M(x−1,y−1)+CL(x,y)
// M(x,y)=min⎨ M(x,y−1)+CU(x,y)
//           ⎩ M(x+1,y−1)+CR(x,y)
//

fn cost_candidate_pixel<I, P, S>(
	image: &I,
	energy: &EnergyMap,
	(x, y): (u32, u32),
) -> EnergyAndBackPointer<u32>
where
	I: GenericImageView<Pixel = P> + Sync,
	P: Pixel<Subpixel = S> + Sync + 'static,
	S: Primitive + Sync + 'static,
{
	let epp = |(x1, y1), (x2, y2)| {
		energy_of_pixel_pair(&image.get_pixel(x1, y1), &image.get_pixel(x2, y2))
	};

	let y_above = y - 1;
	let max_width = image.width() - 1;

	let cost_up = if x == 0 {
		epp((x, y_above), (x + 1, y_above))
	} else if x == max_width {
		epp((x - 1, y_above), (x, y_above))
	} else {
		epp((x - 1, y_above), (x + 1, y_above))
	};

	let mut current_cost = EnergyAndBackPointer {
		energy: cost_up + energy[(x, y_above)].energy,
		parent: x,
	};

	let ccc = |x_above, current_cost: EnergyAndBackPointer<u32>| {
		let n = cost_up + energy[(x_above, y_above)].energy + epp((x, y_above), (x_above, y));
		if n < current_cost.energy {
			EnergyAndBackPointer {
				energy: n,
				parent: x_above,
			}
		} else {
			current_cost
		}
	};

	if x != 0 {
		current_cost = ccc(x - 1, current_cost)
	}

	if x != max_width {
		current_cost = ccc(x + 1, current_cost)
	};

	current_cost
}

fn start_and_end(
	index: usize,
	segment_size: usize,
	width: usize,
	last_thread: usize,
) -> (usize, usize) {
	(
		index * segment_size,
		if index == last_thread {
			width as usize
		} else {
			(index + 1) * segment_size
		},
	)
}

fn calculate_one_row<I, P, S>(
	y: u32,
	width: usize,
	segment_size: usize,
	thread_count: usize,
	image: &I,
	emap: &EnergyMap,
) -> Vec<EnergyAndBackPointer<u32>>
where
	I: GenericImageView<Pixel = P> + Sync,
	P: Pixel<Subpixel = S> + Sync + 'static,
	S: Primitive + Sync + 'static,
{
	let last_thread = thread_count - 1;
	let row = crossbeam::scope(|nursery| {
		let mut row: Vec<EnergyAndBackPointer<u32>> = vec![Default::default(); width];
		let handles: Vec<ScopedJoinHandle<Vec<EnergyAndBackPointer<u32>>>> = (0..thread_count)
			.map(|index| {
				nursery.spawn(move |_| {
					let (sx, ex) = start_and_end(index, segment_size, width, last_thread);
					let mut segment: Vec<EnergyAndBackPointer<u32>> =
						Vec::with_capacity((ex - sx) + 1);
					for x in sx..ex {
						segment.push(cost_candidate_pixel(image, &emap, (x as u32, y)));
					}
					segment
				})
			})
			.collect();

		handles.into_iter().enumerate().for_each(|(index, handle)| {
			let segment = handle.join().unwrap();
			let (sx, ex) = start_and_end(index, segment_size, width, last_thread);
			(row[sx..ex]).copy_from_slice(&segment);
		});
		row
	})
	.unwrap();
	row
}

fn calculate_cost<I, P, S>(image: &I) -> EnergyMap
where
	I: GenericImageView<Pixel = P> + Sync,
	P: Pixel<Subpixel = S> + Sync + 'static,
	S: Primitive + Sync + 'static,
{
	let (width, height) = image.dimensions();
	let mut emap = EnergyMap::new(width, height);
	let mw = width - 1;

	let nebp = |(xl, yl), (xr, yr), parent| EnergyAndBackPointer {
		energy: energy_of_pixel_pair(&image.get_pixel(xl, yl), &image.get_pixel(xr, yr)),
		parent: parent,
	};

	// The upper corners are super-special cases!
	emap[(0, 0)] = nebp((0, 0), (1, 0), 0);
	emap[(mw, 0)] = nebp((mw - 1, 0), (mw, 0), 0);

	// The top row is a special case.  Using the RangeInclusive
	// operator to make explicit that I'm avoiding the corners.
	for x in 1..=(mw - 1) {
		emap[(x, 0)] = nebp((x - 1, 0), (x + 1, 0), 0);
	}

	let thread_count = num_cpus::get();
	let segment_size = ((width + 1) as usize) / thread_count;
	{
		for y in 1..height {
			let row =
				calculate_one_row(y, width as usize, segment_size, thread_count, image, &emap);
			let erow = emap.get_mut_row(y);
			erow.copy_from_slice(&row)
		}
	}
	emap
}

// Again, the trick here is to divvy up the width into segments,
// breaking the target into mut_chunks and readdressing them
// afterward for each row.

/// Given an energy map, return the list of x-coordinates that, when
/// mapped with the range (0..height), give the XY coordinates for each
/// pixel in the seam to be removed.
fn energy_to_seam(energy: &EnergyMap) -> Vec<u32> {
	let (width, height) = (energy.width, energy.height);

	// Find the x coordinate of the bottomost seam with the least energy.
	let mut seam_col = (0..width)
		.min_by_key(|x| energy[(*x, height - 1)].energy)
		.unwrap();
	// Working backwards, generate a vec of x coordinates that that map to
	// the seam, reverse and return.
	(0..height)
		.rev()
		.fold(Vec::<u32>::with_capacity(height as usize), |mut acc, y| {
			acc.push(seam_col);
			seam_col = energy[(seam_col, y)].parent;
			acc
		})
		.into_iter()
		.rev()
		.collect()
}

/// The basic seam engine: just a simple image reference holder, and the pair of functions
/// needed to invoke the AviSha algorithm.
pub struct AviShaTwo<'a, I, P, S>
where
	I: GenericImageView<Pixel = P> + Sync,
	P: Pixel<Subpixel = S> + Sync + 'static,
	S: Primitive + Sync + 'static,
{
	/// A reference to the image we'll be manipulating.
	pub image: &'a I,
}

impl<'a, I, P, S> AviShaTwo<'a, I, P, S>
where
	I: GenericImageView<Pixel = P> + Sync,
	P: Pixel<Subpixel = S> + Sync + 'static,
	S: Primitive + Sync + 'static,
{
	/// Takes a reference to an image, and holds onto it.
	pub fn new(image: &'a I) -> Self {
		AviShaTwo { image }
	}
}

impl<'a, I, P, S> SeamFinder for AviShaTwo<'a, I, P, S>
where
	I: GenericImageView<Pixel = P> + Sync,
	P: Pixel<Subpixel = S> + Sync + 'static,
	S: Primitive + Sync + 'static,
{
	fn find_horizontal_seam(&self) -> Vec<u32> {
		energy_to_seam(&calculate_cost(&Flipper { image: self.image }))
	}

	fn find_vertical_seam(&self) -> Vec<u32> {
		energy_to_seam(&calculate_cost(self.image))
	}
}
