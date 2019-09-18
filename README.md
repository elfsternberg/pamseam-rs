# pnmseam - scale a portable anymap using seam carving

pnmseam is a library and accompanying command line program to scale any
pnm (portable anymap) file using the seamcarving algorithm.  Several
variants of the seamcarving algorithm are supplied. By default the
algorithms are single-threaded, but multithread capability is available
as a compile-time option.

## Synopsis

	pnmseam -s --scale [pnmfile]
	pnmseam -r --reduce [pnmfile]
	pnmseam -z --xysize rows cols [pnmfile]
	pnmseam -p --pixels s [pnmfile]
	pnmseam -o [output file]
	pnmseam --fsc
	pnmseam --fgsc

## NOTICE

This is a design document at this point.  *None* of what is described
below is currently possible with the current code base. The intent is to
start small and build out a complete Rust solution, and then port to C.

## Description

`pnmseam` attempts to resize an image using the algorithms described in
Avidan & Shamir's paper [Seam Carving for Content-Aware Image
Resizing](https://dl.acm.org/citation.cfm?id=1276390) and subsequent
research, and can use both the Basic Seam Carving (BSC) algorithm (the
default), the Forward Seam Carving (FSC) algorithm, or the Flow-Guiding
Seam Carving (FGSC) algorithm.

Seam carving algorithms use a gradient process to calculate a "seam" (a
wandering path of pixels traversing from left-to-right or top-to-bottom)
that can be removed from the image causing the minimal amount of damage
to the original. The seams are chosen based upon how little the image
would change if the seam is removed. In some cases, this can result in a
dramatic cropping and repositioning of the image with the main subject
left untouched; in others, it can cause significant distortion.

The seam carving algorithms are intended primarily to downscale images.
In the event that the request upscales an image, the algorithm
calculates as many seams as necessary to upscale the image and then
processes them from highest to lowest energy, for each creating an
interstitial seam that is the average of its neighbors.  As with
downsizing, this can create significant distortion.

## Project interim notes

The basic premise of this program is that a picture file is a collection
of pixels.  (This isn't necessarily true for some image formats, most
notoriously JPEG, but it's true enough to make it worthwhile, and even
JPEGs are rendered as pixels for purposes of editing.)

The steps therefore are:

- Image -> Energy Map -> Seam Collection -> Lowest Energy Seam
- LowestEnergySeam + Image -> New Image

Challenges:

Images aren't generic.  They came in Luma (greyscale) and RGB formats,
along with their alpha channel variants.  For our purposes, we're not
going to handle alpha channel, since the alpha channel doesn't make
sense for our purposes.  Mapping from/to the image in a generic way 
is our biggest headache.

The task isn't generic, either.  The image can be read-only, but we can
only work with so much in a thread-based fashion.

The other challenge is just the number of different integer types
involved in doing this work.  image-rs indexes the (x, y) pair of any
image as u32, but we know damn well that under the covers it's really a
usize-mapped index into a vec.  Making that work repeatedly is the big
challenge.


## Features

There are two features not enabled by default.

`cargo build --features=threaded` will provide the `-t --threads
[threadcount]` option, which will use as many threads as specified to
generate the energy map and seam selection list, but will not exceed the
CPU count provided by the OS.

`cargo build --features=square_root` will use a floating point-based
energy calculation function.  This is significantly slower, but it has
been reported that this creates better results when working with
relatively small original files (images less that 800x600 pixels).

## References

- [Seam Carving for Content-Aware Image Resizing](https://dl.acm.org/citation.cfm?id=1276390)
- [Improved seam carving for video retargeting](https://dl.acm.org/citation.cfm?id=1360615)
- [Optimized image resizing using flow-guided seam carving](http://citeseerx.ist.psu.edu/viewdoc/download?doi=10.1.1.372.1576&rep=rep1&type=pdf)

## LICENSE

`pnmseam` is Copyright [Elf M. Sternberg](https://elfsternberg.com) (c)
2019, and licensed with the Mozilla Public License vers. 2.0.  A copy of
the license file is included in the root folder.
