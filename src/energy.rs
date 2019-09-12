use image::{GenericImageView, GrayImage, ImageBuffer, Luma, Pixel, Primitive};
use itertools::{iproduct, zip};
use num_traits::pow::pow;
use num_traits::{clamp, NumCast};
use std::convert::TryInto;

// Takes the channels (R,G,B) from two pixels and maps the difference
// between each channel, squares it, and then sums them all up.  This
// is the rusty expression of:
//
//        |Δx|² = (Δrx)²+(Δgx)²+(Δbx)²
//        |Δy|² = (Δry)²+(Δgy)²+(Δby)²
//       e(x,y) = |Δx|²+|Δy|²
//
// Only using map for the channels, and fold for the
// final summation.
//
fn energy_of_pair<S>(p1: &[S], p2: &[S]) -> u32
where
    S: Primitive + 'static,
{
    zip(p1, p2)
        .map(|(c1, c2)| {
            let c1s: i32 = NumCast::from(*c1).unwrap();
            let c2s: i32 = NumCast::from(*c2).unwrap();
            pow(c1s - c2s, 2)
        })
        .fold(0, |a, c| a + <u32 as NumCast>::from(c).unwrap())
}

#[derive(Debug)]
struct TargetMap<P>(Vec<P>, u32, u32);

impl<P> TargetMap<P> {
    pub fn get(&self, x: u32, y: u32) -> &P {
        &(self.0[(y * self.2 + x) as usize])
    }

    pub fn set(&mut self, x: u32, y: u32, p: P) {
        self.0[(y * self.2 + x) as usize] = p;
    }
}

// I'm fond of the ternary operator.  I know that Rust's ifs are
// already expressions, but `cargo fmt` breaks them up line-by-line
// and the matrix of border-handling rules is much easier to read
// once this macro is understood.
macro_rules! t {
    ($condition: expr, $_true: expr, $_false: expr) => {
        if $condition {
            $_true
        } else {
            $_false
        }
    };
}

/// Compute the energy of every pixel in an image, returning

pub fn compute_energy<I, P, S>(image: &I) -> Vec<u32>
where
    I: GenericImageView<Pixel = P>,
    P: Pixel<Subpixel = S> + 'static,
    S: Primitive + 'static,
{
    let (width, height) = image.dimensions();
    let (mw, mh) = (width - 1, height - 1);

    let tmap = TargetMap(
        iproduct!(0..height, 0..width)
            .map(|(y, x)| image.get_pixel(x, y).to_rgb().channels().to_owned())
            .collect(),
        height,
        width,
    );

    iproduct!(0..height, 0..width)
        .map(|(y, x)| {
            let current_pixel = tmap.get(x, y);
            let (leftpixel, rightpixel, uppixel, downpixel) = (
                t!(x == 0, current_pixel, tmap.get(x - 1, y)),
                t!(x >= mw, current_pixel, tmap.get(x + 1, y)),
                t!(y == 0, current_pixel, tmap.get(x, y - 1)),
                t!(y >= mh, current_pixel, tmap.get(x, y + 1)),
            );
            energy_of_pair(&leftpixel, &rightpixel) + energy_of_pair(&uppixel, &downpixel)
        })
        .collect()
}

pub fn energy_to_image(energy: &[u32], width: u32, height: u32) -> GrayImage {
    let mut out: ImageBuffer<Luma<u8>, Vec<u8>> = ImageBuffer::new(width, height);
    let factor = energy.iter().max().unwrap();
    energy.iter().enumerate().for_each(|(i, c)| {
        let (x, y): (u32, u32) = (
            (i % (width as usize)).try_into().unwrap(),
            (i / (width as usize)).try_into().unwrap(),
        );
        let c: u32 = NumCast::from(*c).unwrap();
        let cs = [NumCast::from(clamp(c * 256 / factor, 0, 255)).unwrap()];
        let c = Pixel::from_slice(&cs);
        out.put_pixel(x, y, *c);
    });
    out
}

pub fn energy_to_seams(energy: &[u32], width: u32, height: u32) -> Vec<u32> {
    let origin: TargetMap<u32> = TargetMap(energy.to_owned(), height, width);

    let mut result: TargetMap<(u32, u32)> = TargetMap(
        vec![(0, 0); (height * width).try_into().unwrap()],
        height,
        width,
    );

    let m = width - 1;
    (0..width).for_each(|x| {
        result.set(x, 0, (*origin.get(x, 0), 0));
    });

    // When this is over, we have a map of (total energy of seam so
    // far, parent that created this seam so far)
    iproduct!(1..height, 0..width).for_each(|(y, x)| {
        let e = origin.get(x, y).to_owned();
        let p = [
            t!(
                x == 0,
                (e + result.get(x, y - 1).0, x),
                (e + result.get(x - 1, y - 1).0, x - 1)
            ),
            (e + result.get(x, y - 1).0, x),
            t!(
                x == m,
                (e + result.get(x, y - 1).0, x),
                (e + result.get(x + 1, y - 1).0, x + 1)
            ),
        ]
        .iter()
        .max_by_key(|a| a.0).unwrap().to_owned();
        result.set(x, y, p);
    });

    println!("{:?}", result);
    // Now we must find the x coordinate of the seam with the lowest energy.
    let endpoint = (0..width).map(|x| (x, result.get(x, height - 1).0)).min_by_key(|a| a.1).unwrap().0;
    let mut parent = result.get(endpoint, height - 1).1;
    let mut seam = vec![endpoint,parent,];
    (0..(height - 1)).rev().for_each(|y| {
        parent = result.get(parent, y).1;
        seam.push(parent);
    });
    seam
}
        
    
        
#[cfg(test)]
mod tests {
    /// Given an image, calculate an energy grid.
    use super::*;

    #[test]
    fn energy_grid_to_seam() {
        let width = 5;
        let height = 4;
        let energies = [
            9, 9, 0, 9, 9,
            9, 1, 9, 8, 9,
            9, 9, 9, 9, 0,
            9, 9, 9, 0, 9
        ];
        let expected = [2, 3, 4, 3];
        assert_eq!(energy_to_seams(&energies, height, width), expected);
    }
}
