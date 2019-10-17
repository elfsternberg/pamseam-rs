/// This trait defines how we will return seams from an image.  It's a
/// primitive interface, just enough to make room for multiple seam
/// carvers as well as caching.
pub trait SeamFinder {
    /// Once a SeamFinder has an image (or whatever it needs to make a
    /// rational decision), request a horizontal seam.
    fn find_horizontal_seam(&self) -> Vec<u32>;

    /// Request a vertical seam.
    fn find_vertical_seam(&self) -> Vec<u32>;
}
