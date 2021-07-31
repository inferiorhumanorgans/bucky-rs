#![feature(test)]
#[cfg(test)]
extern crate test;

extern crate csv;
extern crate minidom;
extern crate quick_xml;

mod record;

use record::*;

use std::io::Cursor;
use chrono::{NaiveTime, NaiveDateTime, Timelike};
use minidom::Element;
use quick_xml::Writer;

use bucky::{Margins, Chart};
use bucky::annotated::Annotatable;
use bucky::array::extent::Extent;
use bucky::error::Result;
use bucky::scale::{ScaleContinuous, ScaleLinear, ScaleTime};
use bucky::shape::line::Line;

const CHART: Chart = Chart {
    margins: Margins {
        top: 120, right: 30, bottom: 120, left: 50
    },
    width: 954,
    height: 2400,
};

const RFC_3339_FMT: &str = "%Y-%m-%dT%H:%M:%S";

fn main() -> Result<()> {
    let (data, stations) = load_data();

    let distance_extents = stations.iter().map(|d| d.distance).collect::<Vec<_>>().extents();

    let y = ScaleTime::new()
        .domain(
            NaiveDateTime::parse_from_str("2000-01-01T04:30:00", RFC_3339_FMT).unwrap()
            ..
            NaiveDateTime::parse_from_str("2000-01-02T01:30:00", RFC_3339_FMT).unwrap()
        )?
        .range(CHART.margins.top..(CHART.height - CHART.margins.bottom))?;

    let x = ScaleLinear::<f64, _>::new()
        .domain(distance_extents)?
        // Yikes https://github.com/rust-lang/rust/issues/39797
        .nice(None::<i32>)?
        .range((CHART.margins.left + 10)..(CHART.width - CHART.margins.right))?;

    let mut y_axis = Element::builder("g", "")
        .attr("class", "y axis")
        .attr("transform", format!("translate({},0)", CHART.margins.left));
    {
        // TODO: Handle generating ticks by interval as well as by count
        let tick_values = {
            let scale = y.clone()
            .domain
            (NaiveDateTime::parse_from_str("2000-01-01T04:00:00", RFC_3339_FMT).unwrap()
                ..
            NaiveDateTime::parse_from_str("2000-01-02T01:00:00", RFC_3339_FMT).unwrap())?;
            scale.ticks(Some(21))
        }[1..].to_vec();

        let y_ticks = tick_values.into_iter().annotate("g", |builder, datum| {
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
                .append_text_node(datum.format("%l%P").to_string());

            tick.append(Element::bare("line", ""))
                .set_attr("stroke-opacity", 0.2)
                .set_attr("stroke", "rgb(27, 30, 35)")
                .set_attr("x2", CHART.width);

            tick
        });

        let y_tick_group = Element::builder("g", "")
            .attr("class", "y axis tick-container")
            .append_all(y_ticks)
            .build();
        y_axis.append(y_tick_group);
    }

    let mut x_axis = Element::builder("g", "")
        .attr("class", "x axis");

    {
        let x_ticks = stations.into_iter().annotate("g", |builder, datum| {
            let mut tick = builder
                .attr("class", "x axis tick")
                .attr("stroke", "rgb(27, 30, 35)")
                .attr("transform", format!("translate({}, 0)",  x.scale(datum.distance)));

            tick.append(Element::bare("line", ""))
                .set_attr("y1", CHART.margins.top - 6)
                .set_attr("y2", CHART.margins.top);

            tick.append(Element::bare("line", ""))
                .set_attr("y1", CHART.height - CHART.margins.bottom + 6)
                .set_attr("y2", CHART.height - CHART.margins.bottom);

            tick.append(Element::bare("line", ""))
                .set_attr("stroke-opacity", 0.2)
                .set_attr("stroke-dasharray", "1.5,2")
                .set_attr("y1", CHART.margins.top)
                .set_attr("y2", CHART.height - CHART.margins.bottom);

            tick.append(Element::bare("text", ""))
                .set_attr("transform", format!("translate(0,{}) rotate(-90)", CHART.margins.top))
                .set_attr("stroke", "none")
                .set_attr("x", 12)
                .set_attr("dy", "0.35em")
                .append_text_node(datum.name.as_str());

            tick.append(Element::bare("text", ""))
                .set_attr("text-anchor", "end")
                .set_attr("transform", format!("translate(0,{}) rotate(-90)", CHART.height - CHART.margins.top))
                .set_attr("stroke", "none")
                .set_attr("x", -12)
                .set_attr("dy", "0.35em")
                .append_text_node(datum.name.as_str());

            tick
        });

        let x_tick_group = Element::builder("g", "")
            .attr("class", "x axis tick-container")
            .append_all(x_ticks)
            .build();
        x_axis.append(x_tick_group);
    }

    fn format_dt(time: &NaiveTime) -> NaiveDateTime {
        if time.hour() < 4 {
            let dt_string = time.format("2000-01-02T%H:%M:00").to_string();
            NaiveDateTime::parse_from_str(dt_string.as_str(), RFC_3339_FMT).unwrap()
        } else {
            let dt_string = time.format("2000-01-01T%H:%M:00").to_string();
            NaiveDateTime::parse_from_str(dt_string.as_str(), RFC_3339_FMT).unwrap()
        }
    }

    let mut line = Line::<RouteStop>::new()
        .x(Box::new(|datum, _i| x.scale(datum.distance)))
        .y(Box::new(|datum, _i| {
            let time = datum.time.unwrap();
            y.scale(format_dt(&time))
        }))
        .defined(Box::new(|datum, _i| datum.time.is_some()));

    let trains = Element::builder("g", "")
        .attr("class", "trains")
        .attr("stroke-width", 1.5)
        .append_all(
            data.iter().annotate("g", |builder, datum| {
                let mut run = builder;

                run.append(Element::bare("path", ""))
                    .set_attr("fill", "none")
                    .set_attr("stroke", datum.run_type.color())
                    .set_attr("d", line.generate(datum.stops.as_slice()));

                run.append_all(datum.stops.iter().annotate("circle", |circle_builder, stop| {
                    let circle = circle_builder
                        .attr("stroke", "white")
                        .attr("fill", datum.run_type.color())
                        .attr("transform", format!("translate({},{})",
                            x.scale(stop.distance),
                            y.scale(format_dt(&stop.time.unwrap()))
                        ))
                        .attr("r", 2.5);

                    circle
                }))
            })
        );

    let mut writer = Writer::new(Cursor::new(Vec::new()));

    Element::builder("svg", "http://www.w3.org/2000/svg")
        .attr("viewBox", &[0, 0, CHART.width, CHART.height] as &[i32])
        .attr("overflow", "visible")
        .attr("font-family", "B612 Mono")
        .attr("font-size", "6pt")
        .extend(y_axis.build())
        .extend(x_axis.build())
        .extend(trains.build())
        .build()
        .to_writer(&mut writer)
        .unwrap();

    // https://stackoverflow.com/questions/19076719/how-do-i-convert-a-vector-of-bytes-u8-to-a-string
    let buf = writer.into_inner().into_inner();
    let s = match std::str::from_utf8(&buf) {
        Ok(v) => v,
        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    };
    println!("{}", s);

    Ok(())
}
