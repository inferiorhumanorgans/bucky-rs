//! Applies a zero baseline.

use crate::shape::offset::OffsetGenerator;
use crate::shape::stack::{StackIntermediateRow, StackRow};

#[derive(Debug)]
pub struct OffsetNone {}
impl OffsetGenerator for OffsetNone {
    fn offset(&self, intermediate: StackIntermediateRow) -> Vec<StackRow> {
        let row_len = intermediate[0].len();

        let mut ret = vec![vec![0.0..0.0; row_len]; intermediate.len()];

        for (i, row) in intermediate.iter().enumerate() {
            for (j, cell) in row.iter().enumerate() {
                let start = match i {
                    0 => 0.,
                    _ => ret[i - 1][j].end,
                };

                ret[i][j] = start..*cell
            }
        }

        ret
    }
}
