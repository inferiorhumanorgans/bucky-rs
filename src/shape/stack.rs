//! Stack generators take a flat collection of data and return an array representing each series.

use std::marker::PhantomData;

use crate::shape::offset::{OffsetGenerator, OffsetNone};

pub type StackIntermediateRow = Vec<Vec<f64>>;
pub type StackRow = Vec<std::ops::Range<f64>>;

pub trait Stackable {
    fn values(&self) -> Vec<f64>;
}

pub struct Stack<ItemType, IteratorType> {
    offset: Box<dyn OffsetGenerator>,
    data: StackIntermediateRow,

    phantom_item: PhantomData<ItemType>,
    phantom_iterator: PhantomData<IteratorType>,
}

impl<ItemType, IteratorType> Stack<ItemType, IteratorType>
where
    ItemType: std::fmt::Debug + Stackable,
{
    pub fn new(it: IteratorType) -> Self
    where
        IteratorType: std::iter::Iterator<Item = ItemType>,
    {
        Self {
            offset: Box::new(OffsetNone {}),
            data: it.map(|d| d.values()).collect(),
            phantom_item: PhantomData {},
            phantom_iterator: PhantomData {},
        }
    }

    pub fn offset(self, offset: Box<dyn OffsetGenerator>) -> Self {
        Self { offset, ..self }
    }

    pub fn stack(&self) -> Vec<StackRow>
    where
        IteratorType: std::iter::Iterator<Item = ItemType>,
    {
        let row_len = self.data[0].len();

        let mut intermediate = vec![vec![0.0; self.data.len()]; row_len];

        for (i, row) in self.data.iter().enumerate() {
            for (j, cell) in row.iter().enumerate() {
                let last = match j {
                    0 => 0.,
                    _ => intermediate[j - 1][i],
                };

                intermediate[j][i] = last + cell;
            }
        }

        let ret = self.offset.offset(intermediate);

        ret
    }
}

#[test]
fn example_from_readme() {
    #[derive(Debug)]
    struct Datum {
        pub month: String,
        pub apples: i32,
        pub bananas: i32,
        pub cherries: i32,
        pub dates: i32,
    }
    impl Stackable for &Datum {
        fn values(&self) -> Vec<f64> {
            vec![
                self.apples.into(),
                self.bananas.into(),
                self.cherries.into(),
                self.dates.into(),
            ]
        }
    }

    let data = vec![
        Datum {
            month: "Jan".into(),
            apples: 3840,
            bananas: 1920,
            cherries: 960,
            dates: 400,
        },
        Datum {
            month: "Feb".into(),
            apples: 1600,
            bananas: 1440,
            cherries: 960,
            dates: 400,
        },
        Datum {
            month: "Mar".into(),
            apples: 640,
            bananas: 960,
            cherries: 640,
            dates: 400,
        },
        Datum {
            month: "Apr".into(),
            apples: 320,
            bananas: 480,
            cherries: 640,
            dates: 400,
        },
    ];

    let s = Stack::new(data.iter()).stack();

    assert_eq!(s[0], vec![0.0..3840.0, 0.0..1600.0, 0.0..640.0, 0.0..320.0]);
    assert_eq!(
        s[1],
        vec![3840.0..5760.0, 1600.0..3040.0, 640.0..1600.0, 320.0..800.0]
    );
    assert_eq!(
        s[2],
        vec![
            5760.0..6720.0,
            3040.0..4000.0,
            1600.0..2240.0,
            800.0..1440.0
        ]
    );
    assert_eq!(
        s[3],
        vec![
            6720.0..7120.0,
            4000.0..4400.0,
            2240.0..2640.0,
            1440.0..1840.0
        ]
    );
}
