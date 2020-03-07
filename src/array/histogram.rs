use std::convert::TryInto;
use std::ops::Range;

use crate::array::extent::Extent;
use crate::array::threshold::{SturgesThreshold, Threshold};
use crate::array::ticks::TickStep;

#[derive(Debug, PartialEq)]
pub struct HistogramBin<DataType> {
    pub range: Range<f64>,
    pub values: Vec<DataType>,
}

impl<DataType> HistogramBin<DataType> {
    pub fn new(range: Range<f64>) -> Self {
        Self {
            range: range,
            values: vec![],
        }
    }

    pub fn from_range_and_values(range: Range<f64>, values: Vec<DataType>) -> Self {
        Self { range, values }
    }

    pub fn add(self, _value: DataType) -> Self {
        unimplemented!()
    }

    pub fn append(&mut self, value: DataType) {
        self.values.push(value)
    }
}

#[derive(Debug)]
pub struct Histogram<'a, DataType> {
    data: &'a [DataType],
    domain: Option<Range<f64>>,
    thresholds: Option<&'a [f64]>,
}

impl<'a, DataType> Histogram<'a, DataType>
where
    DataType: Clone + std::fmt::Debug,
{
    pub fn new(data: &'a [DataType]) -> Self {
        Self {
            data,
            domain: None,
            thresholds: None,
        }
    }

    pub fn thresholds(self, thresholds: Option<&'a [f64]>) -> Self {
        Self { thresholds, ..self }
    }

    pub fn domain(self, domain: Option<Range<f64>>) -> Self {
        Self { domain, ..self }
    }

    pub fn data(self, data: &'a [DataType]) -> Self {
        Self { data, ..self }
    }

    pub fn histogram<AccessorType>(&self, accessor: AccessorType) -> Vec<HistogramBin<DataType>>
    where
        AccessorType: FnMut(&DataType) -> f64,
    {
        let values: Vec<f64> = self.data.iter().map(accessor).collect();

        let _extents;
        let extents: &Range<f64> = match &self.domain {
            Some(domain) => domain,
            // Gross, but avoids a clone or copy
            None => {
                _extents = values.extents();
                &_extents
            }
        };

        let _thresholds: Vec<f64>;
        let thresholds: &[f64] = match &self.thresholds {
            Some(thresholds) => thresholds,
            None => {
                let threshold_count =
                    SturgesThreshold {}.threshold(&values, extents.start, extents.end);

                // Convert number of thresholds into uniform thresholds.
                // Remove any thresholds outside the domain.
                let tick_step = extents.tick_step(threshold_count.try_into().expect("Overflow"));
                let lower = (extents.start / tick_step).ceil() * tick_step;
                let upper = extents.end;

                let step_count = ((upper - lower) / tick_step).ceil() as i32;
                _thresholds = (0..step_count)
                    .map(|i| lower + i as f64 * tick_step)
                    .filter(|i| i > &lower && i <= &upper)
                    .collect();
                &_thresholds
            }
        };

        // Create bins.
        let mut bins: Vec<HistogramBin<DataType>> = thresholds
            .iter()
            .enumerate()
            .map(|(i, cur_threshold)| {
                if i == 0 {
                    HistogramBin::new(extents.start..*cur_threshold)
                } else if i >= thresholds.len() {
                    HistogramBin::new(thresholds[i - 1]..extents.end)
                } else {
                    HistogramBin::new(thresholds[i - 1]..*cur_threshold)
                }
            })
            .collect();
        bins.push(HistogramBin::new(*thresholds.last().unwrap()..extents.end));

        // Assign data to bins by value, ignoring any outside the domain.
        for (i, value) in values.iter().enumerate() {
            let value = *value;
            if extents.start <= value && value <= extents.end {
                let position = thresholds.binary_search_by(|probe| {
                    probe.partial_cmp(&value).expect("NaN is not allowed here")
                });
                let position: usize = match position {
                    Ok(n) => n + 1,
                    Err(n) => n,
                };
                bins[position].append(self.data[i].clone());
            }
        }

        bins
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple() {
        let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let hist = Histogram::new(&data).histogram(|d| (*d).into());

        eprintln!("HIST: {:?}", hist);

        assert_eq!(
            hist[0],
            HistogramBin::<i32>::from_range_and_values(1.0..4.0, vec![1, 2, 3])
        );
        assert_eq!(
            hist[1],
            HistogramBin::<i32>::from_range_and_values(4.0..6.0, vec![4, 5])
        );
        assert_eq!(
            hist[2],
            HistogramBin::<i32>::from_range_and_values(6.0..8.0, vec![6, 7])
        );
        assert_eq!(
            hist[3],
            HistogramBin::<i32>::from_range_and_values(8.0..10.0, vec![8, 9, 10])
        );
    }

    #[test]
    fn complex() {
        #[derive(Debug, Clone, Copy, PartialEq)]
        struct Item<'a> {
            location: &'a str,
            area: i32,
        }

        let data = vec![
            Item {
                location: "alaska",
                area: 1_518_800,
            },
            Item {
                location: "texas",
                area: 696_200,
            },
            Item {
                location: "california",
                area: 414_000,
            },
            Item {
                location: "montana",
                area: 380_850,
            },
            Item {
                location: "new_mexico",
                area: 314_460,
            },
            Item {
                location: "arizona",
                area: 295_000,
            },
            Item {
                location: "nevada",
                area: 286_350,
            },
            Item {
                location: "colorado",
                area: 269_837,
            },
            Item {
                location: "oregon",
                area: 254_810,
            },
            Item {
                location: "wyoming",
                area: 253_350,
            },
        ];

        let hist = Histogram::new(&data).histogram(|d| d.area.into());

        eprintln!("BINS:");
        for bin in hist.iter() {
            eprintln!("\t{:?}", bin)
        }

        assert_eq!(
            hist[0],
            HistogramBin::<Item>::from_range_and_values(
                253350.0..600000.0,
                vec![data[2], data[3], data[4], data[5], data[6], data[7], data[8], data[9]]
            )
        );
        assert_eq!(
            hist[1],
            HistogramBin::<Item>::from_range_and_values(600000.0..800000.0, vec![data[1]])
        );
        assert_eq!(
            hist[5],
            HistogramBin::<Item>::from_range_and_values(1400000.0..1518800.0, vec![data[0]])
        );
    }

    #[test]
    fn computes_bins_of_specified_array() {
        let data = vec![0, 0, 0, 10, 20, 20];
        let hist = Histogram::new(&data).histogram(|d| (*d).into());

        eprintln!("HIST: {:?}", hist);

        assert_eq!(
            hist[0],
            HistogramBin::<i32>::from_range_and_values(0.0..5.0, vec![0, 0, 0])
        );
        assert_eq!(
            hist[1],
            HistogramBin::<i32>::from_range_and_values(5.0..10.0, vec![])
        );
        assert_eq!(
            hist[2],
            HistogramBin::<i32>::from_range_and_values(10.0..15.0, vec![10])
        );
        assert_eq!(
            hist[3],
            HistogramBin::<i32>::from_range_and_values(15.0..20.0, vec![20, 20])
        );
    }

    #[test]
    fn domain_sets_the_domain() {
        let data = vec![1, 2, 2, 10, 18, 18];
        let mut hist = Histogram::new(&data);

        let bins1 = hist.histogram(|d| (*d).into());

        hist = hist.domain(Some(0.0..20.0));

        let bins2 = hist.histogram(|d| (*d).into());

        assert_ne!(bins1, bins2);

        assert_eq!(
            bins2[0],
            HistogramBin::<i32>::from_range_and_values(0.0..5.0, vec![1, 2, 2])
        );
        assert_eq!(
            bins2[1],
            HistogramBin::<i32>::from_range_and_values(5.0..10.0, vec![])
        );
        assert_eq!(
            bins2[2],
            HistogramBin::<i32>::from_range_and_values(10.0..15.0, vec![10])
        );
        assert_eq!(
            bins2[3],
            HistogramBin::<i32>::from_range_and_values(15.0..20.0, vec![18, 18])
        );
    }
}
