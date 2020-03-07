use std::ops::Range;

use super::RangeInterpolator;

/// This interpolator is similar to NumberInterpolator, except it will round the
/// resulting value to the nearest integer.
#[derive(Clone, Debug)]
pub struct RoundInterpolator {}

impl<'a> RangeInterpolator<'a, f64> for RoundInterpolator {
    fn new() -> Self {
        Self {}
    }

    fn interpolate_range(&'a self, range: &Range<f64>, n: f64) -> f64 {
        (range.start * (1.0 - n) + range.end * n).round()
    }
}

/// NumberInterpolator interpolates between two floating point (f64) values.
#[derive(Clone, Debug)]
pub struct NumberInterpolator {}

impl<'a> RangeInterpolator<'a, f64> for NumberInterpolator {
    fn new() -> Self {
        Self {}
    }

    fn interpolate_range(&'a self, range: &Range<f64>, n: f64) -> f64 {
        range.start * (1.0 - n) + range.end * n
    }
}

#[cfg(test)]
mod tests {
    use crate::{assert_delta, test::DELTA};

    use super::*;

    #[test]
    fn number() {
        let i = 10.0..42.0;
        let interp = NumberInterpolator {};

        assert_delta!(10.0, interp.interpolate_range(&i, 0.0), DELTA);
        assert_delta!(13.2, interp.interpolate_range(&i, 0.1), DELTA);
        assert_delta!(16.4, interp.interpolate_range(&i, 0.2), DELTA);
        assert_delta!(19.6, interp.interpolate_range(&i, 0.3), DELTA);
        assert_delta!(22.8, interp.interpolate_range(&i, 0.4), DELTA);
        assert_delta!(26.0, interp.interpolate_range(&i, 0.5), DELTA);
        assert_delta!(29.2, interp.interpolate_range(&i, 0.6), DELTA);
        assert_delta!(32.4, interp.interpolate_range(&i, 0.7), DELTA);
        assert_delta!(35.6, interp.interpolate_range(&i, 0.8), DELTA);
        assert_delta!(38.8, interp.interpolate_range(&i, 0.9), DELTA);
        assert_delta!(42.0, interp.interpolate_range(&i, 1.0), DELTA);
    }

    #[test]
    fn new_trait() {
        let range = 10.0..42.0;
        let interp = RoundInterpolator {};

        assert_eq!(10.0, interp.interpolate_range(&range, 0.0));
        assert_eq!(13.0, interp.interpolate_range(&range, 0.1));
        assert_eq!(16.0, interp.interpolate_range(&range, 0.2));
        assert_eq!(20.0, interp.interpolate_range(&range, 0.3));
        assert_eq!(42.0, interp.interpolate_range(&range, 1.0));
    }
}
