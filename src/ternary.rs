/// My ternary expression handler.  While it may seem redundant, it's
/// surprisingly useful when working with complex logic tables, such
/// as the edge (literally) cases is the seam carving algorithm.
#[macro_export]
macro_rules! cq {
    ($condition: expr, $_true: expr, $_false: expr) => {
        if $condition {
            $_true
        } else {
            $_false
        }
    };
}

