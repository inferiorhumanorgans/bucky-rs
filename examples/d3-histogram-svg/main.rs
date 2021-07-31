extern crate serde;
extern crate svg;
extern crate csv;

use std::convert::TryFrom;
use std::io;

use serde::Deserialize;
use svg::node::element;
use svg::{Document, Node};

use bucky::{ Margins, Chart };
use bucky::array::extent::Extent;
use bucky::array::histogram::Histogram;
use bucky::array::ticks::Ticks;
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
        .thresholds(Some(&x.domain.ticks(Some(40))))
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

    let mut document = Document::new()
        .set("viewBox", (0, 0, CHART.width, CHART.height));

    // svg-rs doesn't support tspsan nodes easily
    let title = {
        let mut group = element::Group::new()
            .set("transform", format!("translate({}, {})", CHART.width - CHART.margins.right, CHART.margins.top as f64 * 1.5));

        let title_node = svg::node::Text::new(r#"<tspan text-anchor="end" x="0" font-size="18pt">Unemployment rate by county, August 2016.</tspan><tspan text-anchor="end" x="0" dy="1.35em" font-size="80%">Source: Bureau of Labor Statistics.</tspan>"#);
        let text = element::Text::new()
            .set("font-family", "Gill Sans")
            .set("font-weight", 100)
            .set("x", 0)
            .set("y", 0)
            .set("text-anchor", "end")
            .add(title_node);

        group.append(text);

        group
    };
    document.append(title);

    let x_axis = {
        let mut group = element::Group::new()
            .set("transform", format!("translate(0, {})", CHART.height - CHART.margins.bottom))
            .set("class", "x axis");

        let title_node = svg::node::Text::new("Unemployment (%)");
        let title = element::Text::new()
            .set("font-family", "B612 Mono")
            .set("font-size", "5pt")
            .set("y", "-4em")
            .set("text-anchor", "end")
            .set("x", x.scale(x.domain.end))
            .set("transform", format!("translate(0, {})",  y.scale(y.domain.end)))
            .add(title_node);
        group.append(title);

        let line = element::Line::new()
            .set("stroke", "#1b191d")
            .set("stroke-width", "1px")
            .set("x1", CHART.margins.left - 1)
            .set("x2", CHART.width - CHART.margins.right + 1)
            .set("y1", 1)
            .set("y2", 1);
        group.append(line);

        let tick_values = x.domain.ticks(Some(CHART.width / 80));
        let mut ticks = element::Group::new()
            .set("transform", format!("translate(0, {})", CHART.margins.top))
            .set("class", "x axis tick-container");
        ticks = tick_values.iter().fold(ticks, |acc, d| {
            let mut tick = element::Group::new()
                .set("class", "x axis tick")
                .set("transform", format!("translate({}, 0)",  x.scale(*d)));

            let text_node = svg::node::Text::new(format!("{}%", (*d).round()));
            let text = element::Text::new()
                .add(text_node)
                .set("font-family", "B612 Mono")
                .set("font-size", "5pt")
                .set("text-anchor", "middle")
                .set("y", -9);

            tick.append(text);

            acc.add(tick)
        });
        group.append(ticks);

        group
    };

    let y_axis = {
        let mut group = element::Group::new()
            .set("transform", format!("translate({},0)", CHART.margins.left))
            .set("class", "y axis ticks-container");

        let title_node = svg::node::Text::new("Counties (#)");
        let title = element::Text::new()
            .add(title_node)
            .set("font-family", "B612 Mono")
            .set("font-size", "5pt")
            .set("dy", "0.32em")
            .set("x", 4)
            .set("transform", format!("translate(0, {})",  y.scale(y.domain.end)));
        group.append(title);

        let line = element::Line::new()
            .set("stroke", "#1b191d")
            .set("stroke-width", "1px")
            .set("x1", -1)
            .set("x2", -1)
            .set("y1", CHART.margins.top)
            .set("y2", CHART.height - CHART.margins.bottom + 1);

        group.append(line);

        let tick_values = y.domain.ticks(Some(10));
        let mut ticks = element::Group::new()
            .set("class", "y axis tick-container");
        ticks = tick_values.iter().fold(ticks, |acc, d| {
            let mut tick = element::Group::new()
                .set("transform", format!("translate(0, {})",  y.scale(*d)));

            let text_node = svg::node::Text::new(format!("{}", d));
            let text = element::Text::new()
                .add(text_node)
                .set("fill", "#1b191d")
                .set("dy", "0.32em")
                .set("x", "-9")
                .set("font-size", "5pt")
                .set("font-family", "B612 Mono")
                .set("text-anchor", "end");
            tick.append(text);

            acc.add(tick)
        });
        group.append(ticks);

        group
    };

    document.append(x_axis);
    document.append(y_axis);

    // Pantone 1585-XGC
    let g = bins.iter().fold(element::Group::new().set("fill", "#FF742F"),
        |acc, d| {
            let bar = element::Rectangle::new()
                .set("data-length", d.values.len())
                .set("data-value", format!("{:?}", d.range))
                .set("x", x.scale(d.range.start))
                .set("width", (x.scale(d.range.end) - x.scale(d.range.start) - 1.0).max(0.0))
                .set("y", y.scale(d.values.len() as u32)) // TODO: Check for overflow
                .set("height", y.scale(0.0) - y.scale(d.values.len() as u32)); // TODO: Check for overflow
            acc.add(bar)
        });

    document.append(g);

    svg::write(io::stdout(), &document).unwrap();
    println!("");

    Ok(())
}
