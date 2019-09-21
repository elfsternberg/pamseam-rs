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
use image::{GenericImageView, Pixel, Primitive};
use num_traits::NumCast;
// use num_cpus;

use std::ops::{Index, IndexMut};

/// Defines the basic energy map: An addressable two-dimensional field
/// containing an object that represents one of several possible
/// objects during processing: a basic u32 for the energy map, or an
/// energy map + parent address, for the seam digraph.
#[derive(Debug)]
pub struct EnergyMap<P: Default + Copy> {
    width: u32,
    height: u32,
    energy: Vec<P>,
}

impl<P: Default + Copy> EnergyMap<P> {

    /// Define a new (abstract) energy map.  The content type must
    /// implement the Default trait.
    pub fn new(width: u32, height: u32) -> Self {
        EnergyMap {
            width,
            height,
            energy: vec![P::default(); width as usize * height as usize],
        }
    }

    // Absolutely, the number one name of this game is keep the index
    // math in a singular location and never, ever mess with it.  This
    // particular variant is the same one used in image.rs.
    fn get_index(&self, x: u32, y: u32) -> usize {
        (y as usize) * (self.width as usize) + (x as usize)
    }

    /// Get the value at a single pixel's address
    pub fn get_pt(&self, x: u32, y: u32) -> P {
        self.energy[self.get_index(x, y)]
    }

    /// Get a mutable reference to the value at a single pixel's address
    pub fn get_pt_mut(&mut self, x: u32, y: u32) -> &mut P {
        let index = self.get_index(x, y);
        &mut self.energy[index]
    }

    /// Set a value at a single pixel's address
    pub fn put_pt(&mut self, x: u32, y: u32, e: P) {
        *self.get_pt_mut(x, y) = e
    }
}

impl<P: Default + Copy> Index<(u32, u32)> for EnergyMap<P> {
    type Output = P;

    /// A convenience addressing mode for getting values.
    fn index(&self, (x, y): (u32, u32)) -> &P {
        let index = self.get_index(x, y);
        &self.energy[index]
    }
}

impl<P: Default + Copy> IndexMut<(u32, u32)> for EnergyMap<P> {
    /// A convenience addressing mode for setting values.
    fn index_mut(&mut self, (x, y): (u32, u32)) -> &mut P {
        let index = self.get_index(x, y);
        &mut self.energy[index]
    }
}

#[derive(Default, Debug, Copy, Clone)]
struct EnergyAndBackPointer<P: Default + Copy> {
    energy: P,
    parent: u32,
}

impl<P: Default + Copy> EnergyAndBackPointer<P> {
    pub fn new(energy: P, parent: u32) -> Self {
        EnergyAndBackPointer { energy, parent }
    }
}

// (Pixel, Pixel) -> Energy
#[inline]
fn energy_of_pair<P, S>(p1: &P, p2: &P) -> u32
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
pub fn calculate_energy<I, P, S>(image: &I) -> EnergyMap<u32>
where
    I: GenericImageView<Pixel = P>,
    P: Pixel<Subpixel = S> + 'static,
    S: Primitive + 'static,
{
    let (width, height) = image.dimensions();
    let (mw, mh) = (width - 1, height - 1);

    let mut emap = EnergyMap::new(width, height);
    for y in 0..height {
        for x in 0..width {
            let current_pixel = image.get_pixel(x, y);
            let (leftpixel, rightpixel, uppixel, downpixel) = (
                cq!(x == 0, current_pixel, image.get_pixel(x - 1, y)),
                cq!(x >= mw, current_pixel, image.get_pixel(x + 1, y)),
                cq!(y == 0, current_pixel, image.get_pixel(x, y - 1)),
                cq!(y >= mh, current_pixel, image.get_pixel(x, y + 1)),
            );
            emap[(x, y)] =
                energy_of_pair(&leftpixel, &rightpixel) + energy_of_pair(&uppixel, &downpixel);
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
pub fn energy_to_vertical_seam(energy: &EnergyMap<u32>) -> Vec<u32> {
    let (width, height) = (energy.width, energy.height);
    let mut target: EnergyMap<EnergyAndBackPointer<u32>> = EnergyMap::new(width, height);

    // Populate the first row with their native energies.
    for i in 0..width {
        target[(i, 0)].energy = energy.get_pt(i, 0);
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
            target[(x, y)] = EnergyAndBackPointer::new(erg + parent.energy, parent_x);
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
pub fn energy_to_horizontal_seam(energy: &EnergyMap<u32>) -> Vec<u32> {
    let (width, height) = (energy.width, energy.height);
    let mut target: EnergyMap<EnergyAndBackPointer<u32>> = EnergyMap::new(width, height);

    // Populate the first row with their native energies.
    for i in 0..height {
        target[(0, i)].energy = energy.get_pt(0, i);
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
            target[(x, y)] = EnergyAndBackPointer::new(erg + parent.energy, parent_y);
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

/// A convenience wrapper: Given an image, get back a vector with the
/// next top-to-bottom seam for that image.
pub fn calculate_vertical_seam<I, P, S>(image: &I) -> Vec<u32>
where
    I: GenericImageView<Pixel = P>,
    P: Pixel<Subpixel = S> + 'static,
    S: Primitive + 'static,
{
    energy_to_vertical_seam(&calculate_energy(image))
}

/// A convenience wrapper: Given an image, get back a vector with the
/// next left-to-write seam for that image.
pub fn calculate_horizontal_seam<I, P, S>(image: &I) -> Vec<u32>
where
    I: GenericImageView<Pixel = P>,
    P: Pixel<Subpixel = S> + 'static,
    S: Primitive + 'static,
{
    energy_to_horizontal_seam(&calculate_energy(image))
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
        let energies = EnergyMap {
            width: 5,
            height: 4,
            energy: ENERGY_DATA.to_vec(),
        };
        let expected = [2, 3, 4, 3];
        assert_eq!(energy_to_vertical_seam(&energies), expected);
    }

    #[test]
    fn energy_grid_to_horizontal_seam() {
        let energies = EnergyMap {
            width: 5,
            height: 4,
            energy: ENERGY_DATA.to_vec(),
        };
        let expected = [0, 1, 0, 1, 2];
        assert_eq!(energy_to_horizontal_seam(&energies), expected);
    }

}
