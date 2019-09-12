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

    let tmap = DataMap::new(
        width,
        height,
        iproduct!(0..height, 0..width)
            .map(|(y, x)| image.get_pixel(x, y).to_rgb().channels().to_owned())
            .collect(),
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

// Width, height, as always.
#[derive(Debug)]
struct CoordMap {
    w: usize,
    h: usize
};

impl<P> CoordMap {
    pub fn pt(&self, x: usize, y: usize) {
        return y * self.w + x
    }
}

#[derive(Debug)]
struct DataMap<P>{
    c: CoordMap,
    d: Vec<P>
}

impl<P> DataMap<P> {
    pub fn new(w: usize, h: usize, d: Vec<p>) {
        DataMap {
            c: CoordMap(w, h),
            d
        }
    }
    
    pub fn get(&self, x: u32, y: u32) -> &P {
        &self.d[self.c(x, y)]
    }

    pub fn set(&mut self, x: u32, y: u32, p: P) {
        self.d[self.c(x, y)] = p;
    }
}

