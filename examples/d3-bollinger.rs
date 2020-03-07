#![feature(test)]
#[cfg(test)]
extern crate test;

extern crate serde;
extern crate csv;
extern crate minidom;
extern crate quick_xml;

use std::io::Cursor;
use std::{iter::Iterator, str};

use chrono::{NaiveDate, NaiveTime, NaiveDateTime};
use minidom::Element;
use quick_xml::Writer;
use serde::Deserialize;

use bucky::{Margins, Chart};
use bucky::annotated::Annotatable;
use bucky::array::extent::Extent;
use bucky::error::Result;
use bucky::scale::{ScaleContinuous, ScaleTime, ScaleLog};
use bucky::shape::line::Line;
use bucky::interpolate::{RangeInterpolator, RoundInterpolator};

#[derive(Clone, Debug, Deserialize, PartialEq)]
struct Record {
    date: NaiveDate,
    close: f64,
}

impl std::cmp::PartialOrd for Record {
    fn partial_cmp(&self, other: &Record) -> Option<std::cmp::Ordering> {
        self.close.partial_cmp(&other.close)
    }
}

const CHART: Chart = Chart {
    margins: Margins {
        top: 10, right: 20, bottom: 30, left: 40
    },
    width: 954,
    height: 600,
};

#[derive(Debug)]
pub struct BollingerBand {
    pub upper: Vec<f64>,
    pub mid: Vec<f64>,
    pub lower: Vec<f64>,
    pub std_dev: i32,
    pub periods: i32,
}

#[allow(dead_code)]
impl BollingerBand {
    pub fn new<DataType, AccessorType>(n: i32, k: i32, data: &[DataType], accessor: AccessorType) -> Self
    where
        AccessorType: FnMut(&DataType) -> f64,
    {
        let values : Vec<f64> = data.iter().map(accessor).collect();
        let length = data.len() as usize;
        let init_n = std::cmp::min(n - 1, length as i32) as usize;
        let mut sum = 0_f64;
        let mut sum2 = 0_f64;

        let mut upper = vec![std::f64::NAN; length];
        let mut mid = vec![std::f64::NAN; length];
        let mut lower = vec![std::f64::NAN; length];

        for value in values[0..init_n].iter() {
            sum += value;
            sum2 += value.powf(2.0);
        }

        for i in init_n..length {
            let value = values[i];

            sum += value;
            sum2 += value.powf(2.0);

            let mean = sum / f64::from(n);
            let deviation : f64 = (
                    (sum2 - sum.powf(2.0) / (n as f64)) /
                    (n as f64 - 1.0)
                ).sqrt();

            lower[i] = mean + deviation * -(k as f64);
            mid[i] = mean;
            upper[i] = mean + deviation * (k as f64);

            let idx = i as i32 - n + 1;
            let value0 = values[idx as usize];
            sum -= value0;
            sum2 -= value0.powf(2.0);
        }

        Self {
            upper, mid, lower, std_dev: k, periods: n
        }
    }
}

fn load_data() -> Vec<Record> {
    let csv_data = include_str!("d3-bollinger.csv");
    let mut rdr = csv::Reader::from_reader(csv_data.as_bytes());
    rdr.deserialize().filter_map(std::result::Result::ok).collect()
}

fn main() -> Result<()> {
    let data = load_data();

    let periods = 20;
    let std_dev = 2;
    let values : Vec<f64> = data.iter().map(|datum| datum.close).collect();
    let bands = BollingerBand::new(periods, std_dev, &data, |d| d.close);

    let beginning_of_day = NaiveTime::from_hms(0, 0, 0);

    let x = ScaleTime::new()
        .domain(data.iter().map(|datum| datum.date.and_time(beginning_of_day)).collect::<Vec<NaiveDateTime>>().extents())?
        .interpolator(RoundInterpolator::new())
        .range(CHART.margins.left..CHART.width - CHART.margins.right)?;

    let y = ScaleLog::new()
        .domain(data.iter().map(|datum| datum.close).collect::<Vec<f64>>().extents())?
        .interpolator(RoundInterpolator::new())
        .range((CHART.height - CHART.margins.bottom - 20)..CHART.margins.top)?;

    let mut x_axis = Element::builder("g")
        .attr("class", "x axis")
        .attr("transform", format!("translate(0, {})", CHART.height - CHART.margins.bottom));
    {
        // TODO: Don't call to_vec
        let tick_values : Vec<NaiveDateTime> = x.ticks(Some(CHART.width / 80))[1..].to_vec();

        let x_ticks = tick_values.into_iter().annotate("g", |builder, datum| {
            let mut tick = builder
                .attr("class", "x axis tick")
                .attr("transform", format!("translate({}, 0)",  x.scale(datum)));

            tick.append(Element::bare("line"))
                .set_attr("stroke", "rgb(27, 30, 35)")
                .set_attr("y2", 6);

            tick.append(Element::bare("text"))
                .set_attr("font-family", "B612 Mono")
                .set_attr("font-size", "5pt")
                .set_attr("text-anchor", "middle")
                .set_attr("y", 9)
                .set_attr("dy", "0.71em")
                .append_text_node((|| {
                    use chrono::Datelike;
                    if datum.month0() == 0 {
                        datum.format("%Y").to_string()
                    } else {
                        datum.format("%B").to_string()
                    }
                })());

            tick
        });

        let x_tick_group = Element::builder("g")
            .attr("transform", format!("translate(0, {})", CHART.margins.top))
            .attr("class", "x axis tick-container")
            .append_all(x_ticks)
            .build();
        x_axis.append(x_tick_group);
    }

    let mut y_axis = Element::builder("g")
        .attr("class", "y axis")
        .attr("transform", format!("translate({},0)", CHART.margins.left));

    {
        let tick_values = y.domain.ticks(Some(10));

        y_axis.append(Element::bare("text"))
            .set_attr("font-family", "B612 Mono")
            .set_attr("font-size", "5pt")
            .set_attr("font-weight", "bold")
            .set_attr("dy", "0.32em")
            .set_attr("x", 4)
            .set_attr("transform", format!("translate(0, {})", y.scale(tick_values.last().cloned().unwrap())))
            .append_text_node("$ Close");

        use bucky::array::ticks::Ticks;
        let y_ticks = tick_values.into_iter().annotate("g", |builder, datum| {
            let mut tick = builder
                .attr("class", "y axis tick")
                .attr("transform", format!("translate(0, {})",  y.scale(datum)));

            tick.append(Element::bare("text"))
                .set_attr("fill", "#1b191d")
                .set_attr("dy", "0.32em")
                .set_attr("x", "-9")
                .set_attr("font-size", "5pt")
                .set_attr("font-family", "B612 Mono")
                .set_attr("text-anchor", "end")
                .append_text_node(format!("{}", datum.round()));

            tick.append(Element::bare("line"))
                .set_attr("x2", -6)
                .set_attr("stroke-width", "1px")
                // .set_attr("stroke-opacity", 0.1)
                .set_attr("stroke", "rgb(27, 30, 35)");

            tick.append(Element::bare("line"))
                .set_attr("x2", CHART.width - CHART.margins.left - CHART.margins.right)
                .set_attr("stroke-width", "1px")
                .set_attr("stroke-opacity", 0.1)
                .set_attr("stroke", "rgb(27, 30, 35)");

            tick
        });

        let y_tick_group = Element::builder("g")
            .attr("class", "x axis tick-container")
            .append_all(y_ticks)
            .build();
        y_axis.append(y_tick_group);
    }

    let mut line = Line::<f64>::new()
        .x(Box::new(|_datum, i| x.scale(data[i].date.and_time(beginning_of_day))))
        .y(Box::new(|datum, _i| y.scale(*datum)))
        .defined(Box::new(|datum, _i| !datum.is_nan()));

    let neutral_line = Element::builder("path")
        .attr("stroke", "grey")
        .attr("fill", "none")
        .attr("stroke-width", 1.5)
        .attr("stroke-linejoin", "round")
        .attr("stroke-linecap", "round")
        .attr("d", line.generate(values.as_slice()));

    let green_line = Element::builder("path")
        .attr("stroke", "green")
        .attr("fill", "none")
        .attr("stroke-width", 1.5)
        .attr("stroke-linejoin", "round")
        .attr("stroke-linecap", "round")
        .attr("d", line.generate(bands.lower.as_slice()));

    let blue_line = Element::builder("path")
        .attr("stroke", "blue")
        .attr("fill", "none")
        .attr("stroke-width", 1.5)
        .attr("stroke-linejoin", "round")
        .attr("stroke-linecap", "round")
        .attr("d", line.generate(bands.mid.as_slice()));

    let red_line = Element::builder("path")
        .attr("stroke", "red")
        .attr("fill", "none")
        .attr("stroke-width", 1.5)
        .attr("stroke-linejoin", "round")
        .attr("stroke-linecap", "round")
        .attr("d", line.generate(bands.upper.as_slice()));

    let mut writer = Writer::new(Cursor::new(Vec::new()));
    
    Element::builder("svg")
        .ns("http://www.w3.org/2000/svg")
        .attr("viewBox", &[0, 0, CHART.width, CHART.height] as &[i32])
        .attr("overflow", "visible")
        .extend(x_axis.build())
        .extend(y_axis.build())
        .extend(neutral_line.build())
        .extend(green_line.build())
        .extend(blue_line.build())
        .extend(red_line.build())
        .build()
        .to_writer(&mut writer)
        .unwrap();

    // https://stackoverflow.com/questions/19076719/how-do-i-convert-a-vector-of-bytes-u8-to-a-string
    let buf = writer.into_inner().into_inner();
    let s = match str::from_utf8(&buf) {
        Ok(v) => v,
        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    };
    println!("{}", s);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bands_are_as_expected() {
        let periods = 2;
        let std_dev = 2;
        let bands = BollingerBand::new(periods, std_dev, &[0., 1., 2., 3., 4., 5.], |d| *d);
    
        assert_eq!(&[-0.9142135623730951, 0.08578643762690485, 1.0857864376269049, 2.085786437626905, 3.085786437626905 ], &bands.lower[1..6]);
        assert_eq!(&[ 0.5, 1.5, 2.5, 3.5, 4.5 ], &bands.mid[1..6]);
        assert_eq!(&[ 1.9142135623730951, 2.914213562373095, 3.914213562373095, 4.914213562373095, 5.914213562373095 ], &bands.upper[1..6]);
    }
}
    
#[cfg(test)]
mod benches {
    use test::Bencher;
    use super::*;
    const ITERATIONS : usize = 10000;

    #[bench]
    fn bands(b: &mut Bencher) {
        let periods = 2;
        let std_dev = 2;
        let values = &[0., 1., 2., 3., 4., 5.];

        b.iter(||{
            for _ in 0..ITERATIONS {
                BollingerBand::new(periods, std_dev, values, |d| *d);
            }
        });
    }    
}
