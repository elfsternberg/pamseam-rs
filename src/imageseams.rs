/// This trait defines how we will return seams from an image.  It's a
/// primitive interface, just enough to make room for multiple seam
/// carvers as well as caching.
pub trait ImageSeams {
    /// Once an ImageSeam object has an image (or whatever it needs to
    /// make a rational decision), request a horizontal seam.
    fn horizontal_seam(&self) -> Vec<u32>;

    /// Request a vertical seam.
    fn vertical_seam(&self) -> Vec<u32>;
}
