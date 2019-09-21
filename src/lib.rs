#![deny(missing_docs)]
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Seam Carving
//!
//! The seam carving algorithms analyze an image and identify (user
//! choice) a vertical or horizontal seam, a meandering path of
//! adjacent pixels, that if removed from the image would do the least
//! amount of damage to the information contained in the image.  It's
//! sort-of like automatically cropping the image, but rather than
//! just trim the edges, it trims out a line of pixels.  See the
//! examples.

extern crate image;

mod ternary;

pub mod energy;
pub use energy::{energy_to_horizontal_seam, energy_to_vertical_seam};

pub mod seamcarver;
pub use seamcarver::seamcarve;
