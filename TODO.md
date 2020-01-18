[ ] Make the Seam-Lattice library a submodule.

[ ] Remove std::default::Default dependency. (Remove New)

[ ] Write a function that takes an image and translates it into a
    Lattice, and then back into an image.

[ ] Write a function that takes that Lattice and populates a new field
	representing the energy map.

[ ] Write a function that takes that Lattice and populates yet another
    field, calculating the total seams energy.

[ ] Write a function that takes that Lattice and determines the seam
    with the lowest energy.

[ ] Write a function that takes that lattice and that result, and remove
    that seam.

[ ] Write a function that repairs the damaged lattice.

[ ] Write a function that repeats this effort `n` times, until the image
    has been processed completely.

[ ] Write a function that repeats this effort, switching horizontal and
	vertical scanning, until an image is reduced to a desired size in two
	dimensions.
