mod number;
pub use number::*;

#[cfg(feature = "color")]
mod hsl;
#[cfg(feature = "color")]
pub use hsl::*;

// mod piecewise;
// pub use piecewise::*;

pub trait RangeInterpolator<'a, Output> {
    fn new() -> Self;
    fn interpolate_range(&'a self, range: &std::ops::Range<Output>, n: f64) -> Output;
}
