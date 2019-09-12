use image::{GenericImageView, GrayImage, ImageBuffer, Luma, Pixel, Primitive};
use itertools::{iproduct, zip};
use num_traits::pow::pow;
use num_traits::{clamp, NumCast};
use std::convert::TryInto;
use std::cmp;

macro_rules! t {
    ($condition: expr, $_true: expr, $_false: expr) => {
        if $condition {
            $_true
        } else {
            $_false
        }
    };
}

#[derive(Debug, Copy, Clone)]
struct EnergyAndBackPointer<P> {
    energy: P,
    parent: usize
}

impl<P> EnergyAndBackPointer<P> {
    pub fn new(energy: P, parent: usize) -> Self {
        EnergyAndBackPointer {
            energy,
            parent
        }
    }
}

fn energy_of_pair<P, S>(p1: &P, p2: &P) -> u32
where
    P: Pixel<Subpixel = S> + 'static,
    S: Primitive + 'static,
{
    zip(p1.to_rgb().channels().to_owned(), p2.to_rgb().channels().to_owned())
        .map(|(c1, c2)| {
            let c1s: i32 = NumCast::from(c1).unwrap();
            let c2s: i32 = NumCast::from(c2).unwrap();
            pow(c1s - c2s, 2)
        })
        .fold(0, |a, c| a + <u32 as NumCast>::from(c).unwrap())
}


/// Compute the energy of every pixel in an image, returning
pub fn compute_energy<I, P, S>(image: &I) -> Vec<Vec<u32>>
where
    I: GenericImageView<Pixel = P>,
    P: Pixel<Subpixel = S> + 'static,
    S: Primitive + 'static,
{
    let (width, height) = image.dimensions();
    let (mw, mh) = (width - 1, height - 1);

    let mut emap: Vec<Vec<u32>> = vec![];
    for y in 0..height {
        let mut rowmap = vec![];
        for x in 0..width {
            let current_pixel = image.get_pixel(x, y);
            let (leftpixel, rightpixel, uppixel, downpixel) = (
                t!(x == 0, current_pixel, image.get_pixel(x - 1, y)),
                t!(x >= mw, current_pixel, image.get_pixel(x + 1, y)),
                t!(y == 0, current_pixel, image.get_pixel(x, y - 1)),
                t!(y >= mh, current_pixel, image.get_pixel(x, y + 1)),
            );
            rowmap.push(energy_of_pair(&leftpixel, &rightpixel) +
                        energy_of_pair(&uppixel, &downpixel))
        }
        emap.push(rowmap);
    }
    emap
}    
            

pub fn energy_to_vertical_seam(energy: &[u32], width: usize, height: usize) -> Vec<usize> {
    let energy: Vec<&[u32]> = energy.chunks(width).collect();

    let mut target: Vec<Vec<EnergyAndBackPointer<u32>>> = (0..height)
        .map(|_| vec![EnergyAndBackPointer::new(0, 0); width]).collect();

    // Populate the first row with their native energies.
    for (i, e) in energy[0].iter().enumerate() {
        target[0][i].energy = *e
    }

    let maxwidth = width - 1;
    // For every subsequent row, populate the target cell with the sum
    // of the *lowest adjacent upper energy* and the *x coordinate of
    // that energy*
    for y in 1..height {
        let row = energy[y];
        for (x, erg) in row.iter().enumerate() {
            let range = t!(x == 0, 0, x - 1)..=t!(x == maxwidth, maxwidth, x + 1);
            let parent_x = range.min_by_key(|x| target[y - 1][*x].energy).unwrap();
            let parent = target[y - 1][parent_x];
            target[y][x] = EnergyAndBackPointer::new(erg + parent.energy, parent_x);
        }
    }

    // Find the x coordinate of the bottomost seam with the least energy.
    let mut seam_col = (0..width).min_by_key(|x| target[height - 1][*x].energy).unwrap();
    // Working backwards, generate a vec of x coordinates that that map to
    // the seam, reverse and return.
    let seams: Vec<usize> = vec![];
    (0..height).rev().fold(seams, |mut acc, y| {
        acc.push(seam_col);
        seam_col = target[y][seam_col].parent;
        acc
    }).into_iter().rev().collect()
}

pub fn energy_to_horizontal_seam(energy: &[u32], width: usize, height: usize) -> Vec<usize> {
    let energy: Vec<&[u32]> = energy.chunks(width).collect();

    let mut target: Vec<Vec<EnergyAndBackPointer<u32>>> = (0..height)
        .map(|_| vec![EnergyAndBackPointer::new(0, 0); width]).collect();

    // Iterate through the rows, copying the energy of the first column
    // to the first column of the target rows.
    for (i, e) in energy.iter().enumerate() {
        target[i][0].energy = e[0];
    }

    let maxheight = height - 1;
    let maxwidth = width - 1;
    // For every subsequent row, populate the target cell with the sum
    // of the *lowest adjacent upper energy* and the *x coordinate of
    // that energy*
    for x in 1..width {
        for y in 0..height {
            let erg = energy[y][x];
            let range = t!(y == 0, 0, y - 1)..=t!(y == maxheight, maxheight, y + 1);
            let parent_y = range.min_by_key(|y| target[*y][x - 1].energy).unwrap();
            let parent = target[parent_y][x - 1];
            target[y][x] = EnergyAndBackPointer::new(erg + parent.energy, parent_y);
        }
    }

    // Find the x coordinate of the bottomost seam with the least energy.
    let mut seam_row = (0..height).min_by_key(|y| target[*y][width - 1].energy).unwrap();
    // Working backwards, generate a vec of x coordinates that that map to
    // the seam, reverse and return.
    let seams: Vec<usize> = vec![];
    (0..width).rev().fold(seams, |mut acc, y| {
        acc.push(seam_row);
        seam_row = target[seam_row][y].parent;
        acc
    }).into_iter().rev().collect()
}

#[cfg(test)]
mod tests {
    /// Given an image, calculate an energy grid.
    use super::*;

    #[test]
    fn energy_grid_to_vertical_seam() {
        let width = 5;
        let height = 4;
        let energies = [
            9, 9, 0, 9, 9,
            9, 1, 9, 8, 9,
            9, 9, 9, 9, 0,
            9, 9, 9, 0, 9
        ];
        let expected = [2, 3, 4, 3];
        assert_eq!(energy_to_vertical_seam(&energies, width, height), expected);
    }

    #[test]
    fn energy_grid_to_horizontal_seam() {
        let width = 5;
        let height = 4;
        let energies = [
            9, 9, 0, 9, 9,
            9, 1, 9, 8, 9,
            9, 9, 9, 9, 0,
            9, 9, 9, 0, 9
        ];
        let expected = [0, 1, 0, 1, 2];
        assert_eq!(energy_to_horizontal_seam(&energies, width, height), expected);
    }

}
