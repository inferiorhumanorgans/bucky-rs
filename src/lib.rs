#![feature(test)]

extern crate thiserror;
extern crate order_stat;

#[cfg(feature = "time")]
extern crate chrono;
#[cfg(feature = "time")]
extern crate date_iterator;

#[cfg(feature = "dom-minidom")]
extern crate minidom;

use std::ops::Range;

/// Convenience structure to store a chart's margins
#[derive(Debug)]
pub struct Margins {
    pub top: i32,
    pub bottom: i32,
    pub left: i32,
    pub right: i32,
}

/// Convenience structure to store a chart's metadata
#[derive(Debug)]
pub struct Chart {
    pub margins: Margins,
    pub width: i32,
    pub height: i32,
}

impl Chart {
    /// Returns the range of X coordinates that the chart should span
    pub fn inner_width_range(&self) -> Range<f64> {
        (self.margins.left as f64)..((self.width - self.margins.right - self.margins.left) as f64)
    }

    /// Returns the range of Y coordinates that the chart should span
    pub fn inner_height_range(&self) -> Range<f64> {
        ((self.height - self.margins.bottom) as f64)..(self.margins.top as f64)
    }

    // Returns the X coordinate at the bottom of the chart
    pub fn bottom(&self) -> f64 {
        (self.height - self.margins.bottom) as f64
    }
}

#[cfg(test)]
mod test {
    pub(crate) const DELTA: f64 = 1e-6;

    #[macro_export]
    macro_rules! assert_delta {
        ( $lhs:expr, $rhs:expr, $delta:expr ) => {
            assert!(
                $lhs - $delta < $rhs && $rhs < $lhs + $delta,
                "{} is not between {} and {}",
                $rhs,
                $lhs - $delta,
                $lhs + $delta
            );
        };
    }
}

#[cfg(feature = "dom-minidom")]
pub mod annotated;
pub mod array;
pub mod axis;
#[cfg(feature = "color")]
pub mod color;
pub mod error;
pub mod interpolate;
pub mod scale;
pub mod shape;
