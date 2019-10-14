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

// A proxy for a generic image that rotates processing by 90 so that
// intermediate models can be safely sliced up for multi-threaded
// processing.
mod flipper;

// Trait defining how an image becomes a seam.
mod seamfinder;

// Some simple macros
mod ternary;

// A generic two-dimensional map, used to hold intermediate data.
mod twodmap;

// Functions to calculate the energy distance between
// two pixel pairs, using a variety of methods.
pub mod pixelpairs;

// The original algorithm by Avidan and Shamir.
pub mod avisha1;
pub use avisha1::AviShaOne;

// The "forward energy" algorithm by Avidan and Shamir.
pub mod avisha2;
pub use avisha2::AviShaTwo;

// Takes an Image and an ImageSeam and produces a new image with a seam
// carved out.
pub mod seamcarver;
pub use seamcarver::seamcarve;
