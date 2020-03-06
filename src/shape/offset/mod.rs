//! Offset functions update the lower and upper values in an array of series.

use crate::shape::stack::{StackIntermediateRow, StackRow};

/// Implement this trait to provide a new offset generator.
pub trait OffsetGenerator {
    fn offset(&self, intermediate: StackIntermediateRow) -> Vec<StackRow>;
}

mod expand;
mod none;

pub use expand::*;
pub use none::*;
