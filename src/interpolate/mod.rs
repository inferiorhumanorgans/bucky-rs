mod number;

pub use number::*;

#[cfg(feature = "color")]
mod hsl;
#[cfg(feature = "color")]
pub use hsl::*;

mod piecewise;
pub use piecewise::*;

/// This is a trait for interpolating over a range object.
pub trait RangeInterpolator<'a, Output> {

    fn new() -> Self;

    /// Given a starting value a and an ending value b, this function takes a
    /// parameter n in the domain [0, 1] and returns the corresponding
    /// interpolated value between a and b. An interpolator typically returns a
    /// value equivalent to a at t = 0 and a value equivalent to b at t = 1.
    /// 
    /// Typically a is equal to the range's starting value and b to the range's
    /// ending value.
    fn interpolate_range(&'a self, range: &std::ops::Range<Output>, n: f64) -> Output;
}
