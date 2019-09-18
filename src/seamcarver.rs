use image::{GenericImageView, ImageBuffer, Pixel, Primitive};
use crate::energy::{calculate_horizontal_seam, calculate_vertical_seam};

// The one tiny inefficiency here is that the seam is copied, into the
// new image, and then the path of pixels immediately to the right of
// the seam are copied over it.
fn remove_vertical_seam<I, P, S>(image: &I, seam: &Vec<u32>) -> ImageBuffer<P, Vec<S>>
where
    I: GenericImageView<Pixel = P>,
    P: Pixel<Subpixel = S> + 'static,
    S: Primitive + 'static,
{
    let (width, height) = image.dimensions();
    let mut imgbuf = image::ImageBuffer::new(width - 1, height);
    for y in 0..height {
        for x in 0..width {
            let pixel = image.get_pixel(x, y);
            imgbuf.put_pixel(if x < seam[y as usize] || x == 0 { x } else { x - 1 }, y, pixel);
        }
    }
    imgbuf
}

// The one tiny inefficiency here is that the seam is copied, into the
// new image, and then the path of pixels immediately below the seam
// are copied over it.
fn remove_horizontal_seam<I, P, S>(image: &I, seam: &Vec<u32>) -> ImageBuffer<P, Vec<S>>
where
    I: GenericImageView<Pixel = P>,
    P: Pixel<Subpixel = S> + 'static,
    S: Primitive + 'static,
{
    let (width, height) = image.dimensions();
    let mut imgbuf = image::ImageBuffer::new(width, height - 1);
    for y in 0..height {
        for x in 0..width {
            let pixel = image.get_pixel(x, y);
            imgbuf.put_pixel(x, if y < seam[x as usize] || y == 0 { y } else { y - 1 }, pixel);
        }
    }
    imgbuf
}

#[derive(PartialEq, Copy, Clone)]
enum Carve {
    Width,
    Height
}

fn carveonce<I, P, S>(image: &I, direction: Carve) -> ImageBuffer<P, Vec<S>>
where
    I: GenericImageView<Pixel = P>,
    P: Pixel<Subpixel = S> + 'static,
    S: Primitive + 'static,
{
    if direction == Carve::Height {
        let seam = calculate_horizontal_seam(image);
        remove_horizontal_seam(image, &seam)
    } else {
        let seam = calculate_vertical_seam(image);
        remove_vertical_seam(image, &seam)
    }
}

pub fn seamcarve<I, P, S>(image: &I, newwidth: u32, newheight: u32) -> Result<ImageBuffer<P, Vec<S>>, String> 
where
    I: GenericImageView<Pixel = P>,
    P: Pixel<Subpixel = S> + 'static,
    S: Primitive + 'static,
{
    let (mut width, mut height) = image.dimensions();
    if width < newwidth || height < newheight {
        return Err("seamcarve cannot upscale an image".to_string());
    }

    let mut direction = Carve::Width;
    let mut scratch = ImageBuffer::<P, Vec<S>>::new(width, height);
    for p in image.pixels() {
        scratch[(p.0, p.1)] = p.2.clone()
    }
        
    while width > newwidth && height > newheight {
        scratch = carveonce(&scratch, direction);
        direction = if direction == Carve::Height { Carve::Width } else { Carve::Height };
        width = scratch.width();
        height = scratch.height();
        println!("B: {}, {}", width, height);
    }

    while width > newwidth {
        scratch = carveonce(&scratch, Carve::Width);
        width = scratch.width();
        println!("W: {}, {}", width, height);
    }

    while height > newheight {
        scratch = carveonce(&scratch, Carve::Height);
        height = scratch.height();
        println!("H: {}, {}", width, height);
    }

    Ok(scratch)
}
    
