// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Calculate the energy of an image
//!
//! Given an image, calucate the energy map and either a horizontal or
//! vertical seam for that image.  Currently uses the most
//! straightforward of the energy map algorithms, the one with no
//! forward energy calculation, although that is coming.

use crate::cq;
use crate::pixelpairs::energy_of_pair_luma as energy_of_pixel_pair;
use crate::seamfinder::SeamFinder;
use crate::twodmap::{EnergyAndBackPointer, TwoDimensionalMap};
use image::{GenericImageView, Pixel, Primitive};
// use num_cpus;

// TODO : How do we carve this up into uniform segments? The cheapest
// is to route around the energymap; divvy it up into width segments,
// then assemble the whole thing later.

// Image -> Energy Map

/// Compute the energy of every pixel in an image.  This is generic on
/// the image type, and it currently uses only the greyscale
/// calculator, rather than differentiating between the greyscale and
/// RGB calculators.  Also, the energy formula is the base one, and
/// none of the alternative energy algorithms described in [Avidan &
/// Shamir (2007)] are implemented.
// TODO: Implement alternative energy calculations?
pub fn calculate_energy<I, P, S>(image: &I) -> TwoDimensionalMap<u32>
where
    I: GenericImageView<Pixel = P>,
    P: Pixel<Subpixel = S> + 'static,
    S: Primitive + 'static,
{
    let (width, height) = image.dimensions();
    let (mw, mh) = (width - 1, height - 1);

    let mut emap = TwoDimensionalMap::new(width, height);
    for y in 0..height {
        for x in 0..width {
            let current_pixel = image.get_pixel(x, y);
            let (leftpixel, rightpixel, uppixel, downpixel) = (
                cq!(x == 0, current_pixel, image.get_pixel(x - 1, y)),
                cq!(x >= mw, current_pixel, image.get_pixel(x + 1, y)),
                cq!(y == 0, current_pixel, image.get_pixel(x, y - 1)),
                cq!(y >= mh, current_pixel, image.get_pixel(x, y + 1)),
            );
            emap[(x, y)] = energy_of_pixel_pair(&leftpixel, &rightpixel)
                + energy_of_pixel_pair(&uppixel, &downpixel);
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
pub fn energy_to_vertical_seam(energy: &TwoDimensionalMap<u32>) -> Vec<u32> {
    let (width, height) = (energy.width, energy.height);
    let mut target: TwoDimensionalMap<EnergyAndBackPointer<u32>> =
        TwoDimensionalMap::new(width, height);

    // Populate the first row with their native energies.
    for i in 0..width {
        target[(i, 0)].energy = energy[(i, 0)];
    }

    let maxwidth = width - 1;
    // For every subsequent row, populate the target cell with the sum
    // of the *lowest adjacent upper energy* and the *x coordinate of
    // that energy*
    for y in 1..height {
        for x in 0..width {
            let erg = energy[(x, y)];
            let range = cq!(x == 0, 0, x - 1)..=cq!(x == maxwidth, maxwidth, x + 1);
            let parent_x = range.min_by_key(|x| target[(*x, (y - 1))].energy).unwrap();
            let parent = target[(parent_x, (y - 1))];
            target[(x, y)] = EnergyAndBackPointer {
                energy: erg + parent.energy,
                parent: parent_x,
            };
        }
    }

    // Find the x coordinate of the bottomost seam with the least energy.
    let mut seam_col = (0..width)
        .min_by_key(|x| target[(*x, height - 1)].energy)
        .unwrap();
    // Working backwards, generate a vec of x coordinates that that map to
    // the seam, reverse and return.
    (0..height)
        .rev()
        .fold(Vec::<u32>::with_capacity(height as usize), |mut acc, y| {
            acc.push(seam_col);
            seam_col = target[(seam_col, y)].parent;
            acc
        })
        .into_iter()
        .rev()
        .collect()
}

// This would be much harder.  The column is broken up into
// segments, but reassembling those becomes a bit nightmarish.
// It's a completely different algorithm!

/// Given an energy map, return the list of y-coordinates that, when
/// mapped with the range (0..width), give the XY coordinates for each
/// pixel in the seam to be removed.
pub fn energy_to_horizontal_seam(energy: &TwoDimensionalMap<u32>) -> Vec<u32> {
    let (width, height) = (energy.width, energy.height);
    let mut target: TwoDimensionalMap<EnergyAndBackPointer<u32>> =
        TwoDimensionalMap::new(width, height);

    // Populate the first row with their native energies.
    for i in 0..height {
        target[(0, i)].energy = energy[(0, i)];
    }

    let maxheight = height - 1;
    // For every subsequent column, populate the target cell with the sum
    // of the *lowest adjacent leftmost energy* and the *y coordinate of
    // that energy*
    for x in 1..width {
        for y in 0..height {
            let erg = energy[(x, y)];
            let range = cq!(y == 0, 0, y - 1)..=cq!(y == maxheight, maxheight, y + 1);
            let parent_y = range.min_by_key(|y| target[(x - 1, *y)].energy).unwrap();
            let parent = target[(x - 1, parent_y)];
            target[(x, y)] = EnergyAndBackPointer {
                energy: erg + parent.energy,
                parent: parent_y,
            };
        }
    }

    // Find the y coordinate of the rightmost seam with the least
    // energy.
    let mut seam_col = (0..height)
        .min_by_key(|x| target[(width - 1, *x)].energy)
        .unwrap();
    // Working backwards, generate a vec of y coordinates that map to
    // the seam, reverse and return.
    (0..width)
        .rev()
        .fold(Vec::<u32>::with_capacity(width as usize), |mut acc, x| {
            acc.push(seam_col);
            seam_col = target[(x, seam_col)].parent;
            acc
        })
        .into_iter()
        .rev()
        .collect()
}

/// The basic seam enigen: just a simple image reference holder.
pub struct AviShaOne<'a, I, P, S>
where
    I: GenericImageView<Pixel = P>,
    P: Pixel<Subpixel = S> + 'static,
    S: Primitive + 'static,
{
    image: &'a I,
}

impl<'a, I, P, S> AviShaOne<'a, I, P, S>
where
    I: GenericImageView<Pixel = P>,
    P: Pixel<Subpixel = S> + 'static,
    S: Primitive + 'static,
{
    /// Takes a reference to an image, and holds onto it.
    pub fn new(image: &'a I) -> Self {
        AviShaOne { image }
    }
}

impl<'a, I, P, S> SeamFinder for AviShaOne<'a, I, P, S>
where
    I: GenericImageView<Pixel = P>,
    P: Pixel<Subpixel = S> + 'static,
    S: Primitive + 'static,
{
    fn find_horizontal_seam(&self) -> Vec<u32> {
        energy_to_horizontal_seam(&calculate_energy(self.image))
    }

    fn find_vertical_seam(&self) -> Vec<u32> {
        energy_to_vertical_seam(&calculate_energy(self.image))
    }
}

#[cfg(test)]
mod tests {
    /// Given an image, calculate an energy grid.
    use super::*;
    use image::{ImageBuffer, Luma};

    const IMAGE_DATA: [u8; 20] = [9, 9, 0, 9, 9, 9, 1, 9, 8, 9, 9, 9, 9, 9, 0, 9, 9, 9, 0, 9];
    const IMAGE_ENERGY: [u32; 20] = [
        0, 145, 81, 82, 0, 64, 0, 130, 0, 82, 0, 64, 0, 145, 81, 0, 0, 81, 81, 162,
    ];
    const ENERGY_DATA: [u32; 20] = [9, 9, 0, 9, 9, 9, 1, 9, 8, 9, 9, 9, 9, 9, 0, 9, 9, 9, 0, 9];

    #[test]
    fn energy_generator_works() {
        let buf: ImageBuffer<Luma<u8>, _> = ImageBuffer::from_raw(5, 4, &IMAGE_DATA[..]).unwrap();
        let energy = calculate_energy(&buf);
        assert_eq!(energy.energy, IMAGE_ENERGY);
    }

    #[test]
    fn energy_grid_to_vertical_seam() {
        let energies = TwoDimensionalMap {
            width: 5,
            height: 4,
            energy: ENERGY_DATA.to_vec(),
        };
        let expected = [2, 3, 4, 3];
        assert_eq!(energy_to_vertical_seam(&energies), expected);
    }

    #[test]
    fn energy_grid_to_horizontal_seam() {
        let energies = TwoDimensionalMap {
            width: 5,
            height: 4,
            energy: ENERGY_DATA.to_vec(),
        };
        let expected = [0, 1, 0, 1, 2];
        assert_eq!(energy_to_horizontal_seam(&energies), expected);
    }
}
