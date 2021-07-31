extern crate serde;
extern crate csv;
extern crate minidom;
extern crate quick_xml;

use std::convert::TryFrom;
use std::io::Cursor;
use std::{iter::Iterator, str};

use minidom::Element;
use quick_xml::Writer;
use serde::Deserialize;

use bucky::{Margins, Chart};
use bucky::annotated::Annotatable;
use bucky::array::extent::Extent;
use bucky::array::histogram::Histogram;
use bucky::error::Result;
use bucky::scale::{ScaleContinuous, ScaleLinear};

#[derive(Clone, Debug, Deserialize, PartialEq)]
struct Record {
    id: String,
    state: String,
    county: String,
    rate: f64,
}

impl std::cmp::PartialOrd for Record {
    fn partial_cmp(&self, other: &Record) -> Option<std::cmp::Ordering> {
        self.rate.partial_cmp(&other.rate)
    }
}

impl From<Record> for f64 {
    fn from(record: Record) -> f64 {
        record.rate
    }
}

const CHART: Chart = Chart {
    margins: Margins {
        top: 20, bottom: 30, left: 40, right: 20,
    },
    width: 1500,
    height: 750,
};

fn load_data() -> Vec<Record> {
    let csv_data = include_str!("../source-data/d3-histogram.csv");
    let mut rdr = csv::Reader::from_reader(csv_data.as_bytes());
    rdr.deserialize().filter_map(std::result::Result::ok).collect()
}

fn main() -> Result<()> {
    let data = load_data();

    let x = ScaleLinear::<f64,_>::new()
        .domain(data.extents())?
        // Yikes https://github.com/rust-lang/rust/issues/39797
        .nice(None::<i32>)?
        .range(CHART.margins.left..CHART.width - CHART.margins.right)?;

    let bins = Histogram::new(&data)
        .thresholds(Some(&x.ticks(Some(40))))
        .histogram(|d| d.rate);

    let last_bin = bins.iter().max_by(|a, b| a.values.len().cmp(&b.values.len())).unwrap();

    // TODO: Find a more elegant solution
    let max_bin_size = match u32::try_from(last_bin.values.len()) {
        Ok(n) => n,
        Err(_e) => panic!("number of items in the histogram bin would overflow an u32 container")
    };

    let y = ScaleLinear::<f64,_>::new()
        .domain(0..max_bin_size)?
        .nice(None::<i32>)?
        .range((CHART.height - CHART.margins.bottom)..CHART.margins.top)?;

    let bars = bins.iter().annotate("rect", |builder, datum| {
        builder
            // Pantone 1585-XGC
            .attr("fill", "#FF742F")
            .attr("data-length", datum.values.len())
            .attr("data-value", format!("{:?}", datum.range))
            .attr("x", x.scale(datum.range.start))
            .attr("width", (x.scale(datum.range.end) - x.scale(datum.range.start) - 1.0).max(0.0))
            .attr("y", y.scale(datum.values.len() as u32)) // TODO: Check for overflow
            .attr("height", y.scale(0.0) - y.scale(datum.values.len() as u32)) // TODO: Check for overflow
    });

    let bar_group = Element::builder("g", "")
        .attr("class", "bars")
        .append_all(bars)        
        .build();

    let mut title = Element::builder("text", "")
        .attr("transform", format!("translate({}, {})", CHART.width - CHART.margins.right, CHART.margins.top as f64 * 1.5))
        .attr("font-family", "Gill Sans")
        .attr("font-weight", 100)
        .attr("x", 0)
        .attr("y", 0)
        .attr("text-anchor", "end");

    title.append(Element::bare("tspan", ""))
        .set_attr("text-anchor", "end")
        .set_attr("x", 0)
        .set_attr("font-size", "18pt")
        .append_text_node("Unemployment rate by county, August 2016.");

    title.append(Element::bare("tspan", ""))
        .set_attr("text-anchor", "end")
        .set_attr("x", 0)
        .set_attr("dy", "1.35em")
        .set_attr("font-size", "80%")
        .append_text_node("Source: Bureau of Labor Statistics.");


    let mut x_axis = Element::builder("g", "")
        .attr("class", "x axis")
        .attr("transform", format!("translate(0, {})", CHART.height - CHART.margins.bottom));

    x_axis.append(Element::bare("text", ""))
        .set_attr("font-family", "B612 Mono")
        .set_attr("font-size", "5pt")
        .set_attr("y", "-4em")
        .set_attr("text-anchor", "end")
        .set_attr("x", x.scale(x.domain.end))
        .set_attr("transform", format!("translate(0, {})",  y.scale(y.domain.end)))
        .append_text_node("Unemployment (%)");

    x_axis.append(Element::bare("line", ""))
        .set_attr("stroke", "#1b191d")
        .set_attr("stroke-width", "1px")
        .set_attr("x1", CHART.margins.left - 1)
        .set_attr("x2", CHART.width - CHART.margins.right + 1)
        .set_attr("y1", 1)
        .set_attr("y2", 1);

    let x_ticks = x.ticks(Some(CHART.width / 80)).into_iter().annotate("g", |builder, datum| {
        let mut tick = builder
            .attr("class", "x axis tick")
            .attr("transform", format!("translate({}, 0)",  x.scale(datum)));

        tick.append(Element::bare("text", ""))
            .set_attr("font-family", "B612 Mono")
            .set_attr("font-size", "5pt")
            .set_attr("text-anchor", "middle")
            .set_attr("y", -9)
            .append_text_node(format!("{}%", (datum).round()));

        tick
    });

    let x_tick_group = Element::builder("g", "")
        .attr("transform", format!("translate(0, {})", CHART.margins.top))
        .attr("class", "x axis tick-container")
        .append_all(x_ticks)
        .build();
    x_axis.append(x_tick_group);

    let mut y_axis = Element::builder("g", "")
        .attr("class", "y axis")
        .attr("transform", format!("translate({},0)", CHART.margins.left));

    y_axis.append(Element::bare("text", ""))
        .set_attr("font-family", "B612 Mono")
        .set_attr("font-size", "5pt")
        .set_attr("dy", "0.32em")
        .set_attr("x", 4)
        .set_attr("transform", format!("translate(0, {})",  y.scale(y.domain.end)))
        .append_text_node("Counties (#)");

    y_axis.append(Element::bare("line", ""))
        .set_attr("stroke", "#1b191d")
        .set_attr("stroke-width", "1px")
        .set_attr("x1", -1)
        .set_attr("x2", -1)
        .set_attr("y1", CHART.margins.top)
        .set_attr("y2", CHART.height - CHART.margins.bottom + 1);

    let y_ticks = y.ticks(Some(10)).into_iter().annotate("g", |builder, datum| {
        let mut tick = builder
            .attr("class", "y axis tick")
            .attr("transform", format!("translate(0, {})",  y.scale(datum)));

        tick.append(Element::bare("text", ""))
            .set_attr("fill", "#1b191d")
            .set_attr("dy", "0.32em")
            .set_attr("x", "-9")
            .set_attr("font-size", "5pt")
            .set_attr("font-family", "B612 Mono")
            .set_attr("text-anchor", "end")
            .append_text_node(format!("{}", datum.round()));

        tick
    });

    let y_tick_group = Element::builder("g", "")
        .attr("class", "x axis tick-container")
        .append_all(y_ticks)
        .build();
    y_axis.append(y_tick_group);

    let mut writer = Writer::new(Cursor::new(Vec::new()));

    Element::builder("svg", "http://www.w3.org/2000/svg")
        .attr("viewBox", &[0, 0, CHART.width, CHART.height] as &[i32])
        .extend(title.build())
        .extend(x_axis.build())
        .extend(y_axis.build())
        .extend(bar_group)
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
