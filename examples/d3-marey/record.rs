use bucky::error::{BuckyError, Result};
use chrono::NaiveTime;
use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub enum TrainDirection {
    North,
    South,
}

impl FromStr for TrainDirection {
    type Err = bucky::error::BuckyError;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "N" => Ok(Self::North),
            "S" => Ok(Self::South),
            _ => Err(BuckyError::UnknownError)
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum RouteType {
    Normal,
    Limited,
    BabyBullet,
    Weekend,
    Saturday,
}

impl RouteType {
    pub fn color(&self) -> &str {
        match self {
            Self::Normal |
            Self::Weekend |
            Self::Saturday => "#a5a5a5",//"rgb(34, 34, 34)",
            Self::Limited => "#eee0a5", //"rgb(183, 116, 9)",
            Self::BabyBullet => "#E31837", //"rgb(192, 62, 29)",
        }
    }
}

impl FromStr for RouteType {
    type Err = bucky::error::BuckyError;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "N" => Ok(Self::Normal),
            "L" => Ok(Self::Limited),
            "B" => Ok(Self::BabyBullet),
            "W" => Ok(Self::Weekend),
            "S" => Ok(Self::Saturday),
            _ => Err(BuckyError::UnknownError)
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct RouteStop {
    pub name: String,
    pub distance: u32,
    pub zone: u8,
    pub time: Option<NaiveTime>,
}

impl RouteStop {
    pub fn with_date_time_string(self, time: &str) -> Result<Self> {
        let time = match NaiveTime::parse_from_str(time, "%I:%M%p") {
            Ok(time) => time,
            Err(_error) => return Err(BuckyError::UnknownError),
        };

        Ok(Self {
            time: Some(time),
            ..self
        })
    }
}

impl FromStr for RouteStop {
    type Err = bucky::error::BuckyError;

    fn from_str(s: &str) -> Result<Self> {
    
        if let [ref _key, ref name, ref distance, ref zone] = s.split('|').collect::<Vec<&str>>().as_slice() {
            Ok(Self {
                name: name.to_string(),
                distance: distance.parse().unwrap(),
                zone: zone.parse().unwrap(),
                time: None,
            })
        } else {
            Err(BuckyError::UnknownError)
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct TrainRun {
    pub number: u16,
    pub run_type: RouteType,
    pub direction: TrainDirection,
    pub stops: Vec<RouteStop>,
}

pub fn load_data() -> (Vec<TrainRun>, Vec<RouteStop>) {
    let tsv_data = include_str!("../source-data/d3-marey.tsv");

    // let northbound_weekday = |r: &TrainRun| {
    //     (r.run_type == RouteType::Normal ||
    //     r.run_type == RouteType::Limited ||
    //     r.run_type == RouteType::BabyBullet)
    //     && r.direction == TrainDirection::North
    // };

    // let southbound_weekday = |r: &TrainRun| {
    //     (r.run_type == RouteType::Normal ||
    //     r.run_type == RouteType::Limited ||
    //     r.run_type == RouteType::BabyBullet)
    //     && r.direction == TrainDirection::South
    // };

    let all_weekday = |r: &TrainRun| {
        r.run_type == RouteType::Normal ||
        r.run_type == RouteType::Limited ||
        r.run_type == RouteType::BabyBullet
    };

    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(b'\t')
        .from_reader(tsv_data.as_bytes());

    let headers = rdr.headers().unwrap().clone();

    let stations : Vec<RouteStop> = headers
        .iter()
        .map(|station| station.parse())
        .filter_map(std::result::Result::ok)
        .collect();

    let records : Vec<TrainRun> = rdr.records()
        .map(|result| {
            let result = result.unwrap();

            let mut number: Option<u16> = None;
            let mut run_type: Option<RouteType> = None;
            let mut direction: Option<TrainDirection> = None;
            let mut stops : Vec<RouteStop> = vec![];

            for (col, value) in headers.iter().zip(&result) {
                match col {
                    "number" => number = Some(value.parse().unwrap()),
                    "type" => run_type = Some(value.parse().unwrap()),
                    "direction" => direction = Some(value.parse().unwrap()),
                    _ if col.starts_with("stop|") => {
                        if value != "-" {
                            match col.parse::<RouteStop>().unwrap().with_date_time_string(value) {
                                Ok(stop) => stops.push(stop),
                                Err(_) => (),
                            }
                        }
                    },
                    _ => (),
                }
            }

            let number = number.expect("Number not specified or malformed");
            let run_type = run_type.expect("Run type not specified or malformed");
            let direction = direction.expect("Direction not specified or malformed");

            TrainRun { number, run_type, direction, stops }
        })
        .filter(all_weekday)
        .collect();

    (records, stations)
}
