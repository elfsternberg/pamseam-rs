/// Defines the basic energy map: An addressable two-dimensional field
/// containing an object that represents one of several possible
/// objects during processing: a basic u32 for the energy map, or an
/// energy map + parent address, for the seam digraph, or the costs
/// map for the forward energy calculation.
#[derive(Debug)]
pub struct TwoDimensionalMap<P: Default + Copy> {
    width: u32,
    height: u32,
    energy: Vec<P>,
}

impl<P: Default + Copy> TwoDimensionalMap<P> {

    /// Define a new (abstract) energy map.  The content type must
    /// implement the Default trait.
    pub fn new(width: u32, height: u32) -> Self {
        TwoDimensionalMap {
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

impl<P: Default + Copy> Index<(u32, u32)> for TwoDimensionalMap<P> {
    type Output = P;

    /// A convenience addressing mode for getting values.
    fn index(&self, (x, y): (u32, u32)) -> &P {
        let index = self.get_index(x, y);
        &self.energy[index]
    }
}

impl<P: Default + Copy> IndexMut<(u32, u32)> for TwoDimensionalMap<P> {
    /// A convenience addressing mode for setting values.
    fn index_mut(&mut self, (x, y): (u32, u32)) -> &mut P {
        let index = self.get_index(x, y);
        &mut self.energy[index]
    }
}
