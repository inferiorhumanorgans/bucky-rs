//! Applies a zero baseline and normalizes the values for each point such that
//! the topline is always one.

use crate::shape::offset::OffsetGenerator;
use crate::shape::stack::{StackIntermediateRow, StackRow};

#[derive(Debug)]
pub struct OffsetExpand {}

impl OffsetGenerator for OffsetExpand {
    fn offset(&self, intermediate: StackIntermediateRow) -> Vec<StackRow> {
        let row_len = intermediate[0].len();

        let mut ret = vec![vec![0.0..0.0; row_len]; intermediate.len()];

        let bounds = intermediate.last().unwrap();

        for (i, row) in intermediate.iter().enumerate() {
            for (j, cell) in row.iter().enumerate() {
                let start = match i {
                    0 => 0.,
                    _ => {
                        let value = intermediate[i - 1][j];
                        if value.is_nan() {
                            if i > 1 {
                                intermediate[i - 2][j]
                            } else {
                                0.0
                            }
                        } else {
                            value
                        }
                    }
                };

                let max = bounds[j];
                ret[i][j] = (start / max)..(cell / max)
            }
        }

        ret
    }
}

#[test]
fn expands_to_fill_0_1_range() {
    use crate::shape::stack::StackIntermediateRow;

    let series: StackIntermediateRow = vec![
        vec![1.0, 2.0, 1.0],
        vec![4.0, 6.0, 3.0],
        vec![9.0, 8.0, 7.0],
    ];

    let stacked = (OffsetExpand {}).offset(series);

    let expected: &[&[std::ops::Range<f64>]] = &[
        &[
            (0.0 / 9.0)..(1.0 / 9.0),
            (0.0 / 8.0)..(2.0 / 8.0),
            (0.0 / 7.0)..(1.0 / 7.0),
        ],
        &[
            (1.0 / 9.0)..(4.0 / 9.0),
            (2.0 / 8.0)..(6.0 / 8.0),
            (1.0 / 7.0)..(3.0 / 7.0),
        ],
        &[
            (4.0 / 9.0)..(9.0 / 9.0),
            (6.0 / 8.0)..(8.0 / 8.0),
            (3.0 / 7.0)..(7.0 / 7.0),
        ],
    ];

    assert_eq!(expected[0], stacked[0].as_slice());
    assert_eq!(expected[1], stacked[1].as_slice());
    assert_eq!(expected[2], stacked[2].as_slice());
}

#[test]
fn treats_nan_as_zero() {
    use crate::shape::stack::StackIntermediateRow;

    let series: StackIntermediateRow = vec![
        vec![1.0, 2.0, 1.0],
        vec![4.0, std::f64::NAN, 3.0],
        vec![9.0, 4.0, 7.0],
    ];

    let stacked = (OffsetExpand {}).offset(series);

    let expected: &[&[std::ops::Range<f64>]] = &[
        &[
            (0.0 / 9.0)..(1.0 / 9.0),
            (0.0 / 4.0)..(2.0 / 4.0),
            (0.0 / 7.0)..(1.0 / 7.0),
        ],
        &[
            (1.0 / 9.0)..(4.0 / 9.0),
            (2.0 / 4.0)..std::f64::NAN,
            (1.0 / 7.0)..(3.0 / 7.0),
        ],
        &[
            (4.0 / 9.0)..(9.0 / 9.0),
            (2.0 / 4.0)..(4.0 / 4.0),
            (3.0 / 7.0)..(7.0 / 7.0),
        ],
    ];

    assert_eq!(expected[0], stacked[0].as_slice());

    // This is expected to fail as you can't compare NaN to NaN
    // should figure a better way of replacing NaN
    assert_ne!(expected[1], stacked[1].as_slice());
    assert_eq!(expected[1][0], stacked[1][0]);
    assert_eq!(expected[1][1].end.is_nan(), true);
    assert_eq!(expected[1][2], stacked[1][2]);

    assert_eq!(expected[2], stacked[2].as_slice());
}
