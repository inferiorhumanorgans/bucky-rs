use std::convert::TryFrom;
use std::ops::Range;

use super::{TickIncrement, TickStep, Ticks};

impl<T> Ticks for Range<T>
where
    T: Into<f64> + Copy,
{
    fn ticks(&self, count: Option<i32>) -> Vec<f64> {
        let start = self.start.into();
        let stop = self.end.into();
        let count = count.unwrap_or(0);

        // let reverse : bool;
        // let mut i : f64 = (-1).into();
        let ticks: Vec<f64>;

        if start == stop && count > 0 {
            return vec![start];
        }

        // TODO: Implement reversing
        // if (reverse = stop < start) n = start, start = stop, stop = n;

        let step = self.tick_increment(count);
        if step == 0.0 {
            return vec![];
        }

        if step > 0.0 {
            let tick_start = (start / step).ceil();
            let tick_stop = (stop / step).floor();
            let n: i32 = (tick_stop - tick_start + 1.0).ceil() as i32;
            ticks = (0..n).map(|i| (tick_start + i as f64) * step).collect();
        } else {
            let tick_start = (start * step).floor();
            let tick_stop = (stop * step).ceil();
            let n: i32 = (tick_start - tick_stop + 1.0).ceil() as i32;
            ticks = (0..n).map(|i| (tick_start - i as f64) / step).collect();
        }

        return ticks;
    }
}

impl<T> TickStep for Range<T>
where
    T: Into<f64> + Copy,
{
    fn tick_step(&self, count: i32) -> f64 {
        use std::f64::consts::*;

        let start: f64 = self.start.into();
        let stop: f64 = self.end.into();

        let step0: f64 = (stop - start).abs() / std::cmp::max(0, count) as f64;
        let step1: f64 = 10_f64.powf((step0.ln() / LN_10).floor());
        let error = step0 / step1;
        let step: f64;

        let e2 = SQRT_2;
        let e5: f64 = 10_f64.sqrt();
        let e10: f64 = 50_f64.sqrt();

        if error >= e10 {
            step = step1 * 10.0
        } else if error >= e5 {
            step = step1 * 5.0;
        } else if error >= e2 {
            step = step1 * 2.0;
        } else {
            step = step1
        }

        if stop < start {
            return -step;
        } else {
            return step;
        }
    }
}

impl<T> TickIncrement<T, f64> for Range<T>
where
    T: Into<f64> + Copy,
{
    fn tick_increment(&self, count: i32) -> f64 {
        use std::f64::consts::*;

        let e2 = SQRT_2;
        let e5: f64 = 10_f64.sqrt();
        let e10: f64 = 50_f64.sqrt();

        let start: f64 = self.start.into();
        let stop: f64 = self.end.into();

        let step: f64 = (stop - start)
            / f64::try_from(std::cmp::max(0, count))
                .expect("Can't fit that many ticks into an f64");

        if !step.is_finite() {
            return step;
        }

        // TODO: Find a more graceful way to do this
        let power = (step.ln() / LN_10).floor();
        if power != (power as i32) as f64 {
            panic!("Couldn't convert power ({:?}) to i32", power);
        };
        let power = power as i32;

        let error: i32 = match step / 10_f64.powi(power) {
            e if e >= e10 => 10,
            e if e >= e5 => 5,
            e if e >= e2 => 2,
            _ => 1,
        };

        if power >= 0 {
            (error * 10_i32.pow(power as u32)).into()
        } else {
            let power = -power;
            let ret: f64 = (10_i32.pow(power as u32) / error).into();
            -ret
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_if_equal_bounds() {
        let empty_ticks: Vec<f64> = vec![];

        assert_eq!(empty_ticks, (1.0..1.0).ticks(Some(-1)));
        assert_eq!(empty_ticks, (1.0..1.0).ticks(Some(0)));
        assert_eq!(empty_ticks, (1.0..1.0).ticks(None));
    }

    #[test]
    fn start_if_bounds_equal_positive_count() {
        let ticks: Vec<f64> = vec![1.0];

        assert_eq!(ticks, (1.0..1.0).ticks(Some(1)));
        assert_eq!(ticks, (1.0..1.0).ticks(Some(10)));
    }

    #[test]
    fn approx_count_plus_1_when_start_lt_stop() {
        assert_eq!(
            vec![0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0],
            (0..1).ticks(Some(10))
        );
        assert_eq!(
            (0..1).ticks(Some(9)),
            vec![0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0]
        );
        assert_eq!(
            (0..1).ticks(Some(8)),
            vec![0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0]
        );
        assert_eq!((0..1).ticks(Some(7)), vec![0.0, 0.2, 0.4, 0.6, 0.8, 1.0]);
        assert_eq!((0..1).ticks(Some(6)), vec![0.0, 0.2, 0.4, 0.6, 0.8, 1.0]);
        assert_eq!((0..1).ticks(Some(5)), vec![0.0, 0.2, 0.4, 0.6, 0.8, 1.0]);
        assert_eq!((0..1).ticks(Some(4)), vec![0.0, 0.2, 0.4, 0.6, 0.8, 1.0]);
        assert_eq!((0..1).ticks(Some(3)), vec![0.0, 0.5, 1.0]);
        assert_eq!((0..1).ticks(Some(2)), vec![0.0, 0.5, 1.0]);
        assert_eq!((0..1).ticks(Some(1)), vec![0.0, 1.0]);
        assert_eq!(
            (0..10).ticks(Some(10)),
            vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0]
        );
        assert_eq!(
            (0..10).ticks(Some(9)),
            vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0]
        );
        assert_eq!(
            (0..10).ticks(Some(8)),
            vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0]
        );
        assert_eq!((0..10).ticks(Some(7)), vec![0.0, 2.0, 4.0, 6.0, 8.0, 10.0]);
        assert_eq!((0..10).ticks(Some(6)), vec![0.0, 2.0, 4.0, 6.0, 8.0, 10.0]);
        assert_eq!((0..10).ticks(Some(5)), vec![0.0, 2.0, 4.0, 6.0, 8.0, 10.0]);
        assert_eq!((0..10).ticks(Some(4)), vec![0.0, 2.0, 4.0, 6.0, 8.0, 10.0]);
        assert_eq!((0..10).ticks(Some(3)), vec![0.0, 5.0, 10.0]);
        assert_eq!((0..10).ticks(Some(2)), vec![0.0, 5.0, 10.0]);
        assert_eq!((0..10).ticks(Some(1)), vec![0.0, 10.0]);
        assert_eq!(
            (-10..10).ticks(Some(10)),
            vec![-10.0, -8.0, -6.0, -4.0, -2.0, 0.0, 2.0, 4.0, 6.0, 8.0, 10.0]
        );
        assert_eq!(
            (-10..10).ticks(Some(9)),
            vec![-10.0, -8.0, -6.0, -4.0, -2.0, 0.0, 2.0, 4.0, 6.0, 8.0, 10.0]
        );
        assert_eq!(
            (-10..10).ticks(Some(8)),
            vec![-10.0, -8.0, -6.0, -4.0, -2.0, 0.0, 2.0, 4.0, 6.0, 8.0, 10.0]
        );
        assert_eq!(
            (-10..10).ticks(Some(7)),
            vec![-10.0, -8.0, -6.0, -4.0, -2.0, 0.0, 2.0, 4.0, 6.0, 8.0, 10.0]
        );
        assert_eq!((-10..10).ticks(Some(6)), vec![-10.0, -5.0, 0.0, 5.0, 10.0]);
        assert_eq!((-10..10).ticks(Some(5)), vec![-10.0, -5.0, 0.0, 5.0, 10.0]);
        assert_eq!((-10..10).ticks(Some(4)), vec![-10.0, -5.0, 0.0, 5.0, 10.0]);
        assert_eq!((-10..10).ticks(Some(3)), vec![-10.0, -5.0, 0.0, 5.0, 10.0]);
        assert_eq!((-10..10).ticks(Some(2)), vec![-10.0, 0.0, 10.0]);
        assert_eq!((-10..10).ticks(Some(1)), vec![0.0,]);
    }
}
