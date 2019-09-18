// #![deny(missing_docs)]

extern crate image;
// extern crate num_cpus;
// extern crate crossbeam;

pub mod energy;
pub use energy::{energy_to_horizontal_seam, energy_to_vertical_seam};

pub mod seamcarver;
pub use seamcarver::seamcarve;
