use super::RangeInterpolator;
use crate::color::*;
use std::ops::Range;

#[derive(Clone, Debug)]
pub struct HslInterpolator {}

impl<'a> RangeInterpolator<'a, Hsl> for HslInterpolator {
    fn new() -> Self {
        Self {}
    }

    fn interpolate_range(&'a self, range: &Range<Hsl>, n: f64) -> Hsl {
        let hue = {
            let mut d = range.end.hue - range.start.hue;

            if (d > 180_f64) || (d < -180_f64) {
                d = d - 360_f64 * (d / 360_f64).round();
            }

            match range.start.hue + (n * d) {
                n if n < -0.0 => 360.0 + n,
                n => n,
            }
        };

        let saturation = {
            let d = range.end.saturation - range.start.saturation;
            range.start.saturation + n * d
        };

        let lightness = {
            let d = range.end.lightness - range.start.lightness;
            range.start.lightness + n * d
        };

        Hsl {
            hue,
            saturation,
            lightness,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hsl_to_hsl() {
        let hsl_start = Hsl {
            hue: 10.0,
            saturation: 0.5,
            lightness: 0.5,
        };
        let hsl_end = Hsl {
            hue: 350.0,
            saturation: 0.5,
            lightness: 0.5,
        };

        let range = hsl_start..hsl_end;
        let interp = HslInterpolator::new();

        //   test.equal(i(0.0), "rgb(191, 85, 64)");
        assert_eq!(
            Hsl {
                hue: 10.0,
                saturation: 0.5,
                lightness: 0.5
            },
            interp.interpolate_range(&range, 0.0)
        );

        //   test.equal(i(0.2), "rgb(191, 77, 64)");
        assert_eq!(
            Hsl {
                hue: 6.0,
                saturation: 0.5,
                lightness: 0.5
            },
            interp.interpolate_range(&range, 0.2)
        );

        //   test.equal(i(0.4), "rgb(191, 68, 64)");
        assert_eq!(
            Hsl {
                hue: 2.0,
                saturation: 0.5,
                lightness: 0.5
            },
            interp.interpolate_range(&range, 0.4)
        );

        //   test.equal(i(0.6), "rgb(191, 64, 68)");
        assert_eq!(
            Hsl {
                hue: 358.0,
                saturation: 0.5,
                lightness: 0.5
            },
            interp.interpolate_range(&range, 0.6)
        );

        //   test.equal(i(0.8), "rgb(191, 64, 77)");
        assert_eq!(
            Hsl {
                hue: 354.0,
                saturation: 0.5,
                lightness: 0.5
            },
            interp.interpolate_range(&range, 0.8)
        );

        //   test.equal(i(1.0), "rgb(191, 64, 85)");
        assert_eq!(
            Hsl {
                hue: 350.0,
                saturation: 0.5,
                lightness: 0.5
            },
            interp.interpolate_range(&range, 1.0)
        );
    }

    #[test]
    fn hsl_to_rgb() {
        let hsl_start = Hsl {
            hue: 10.0,
            saturation: 0.5,
            lightness: 0.5,
        };
        let hsl_end = Hsl {
            hue: 350.0,
            saturation: 0.5,
            lightness: 0.5,
        };

        let range = hsl_start..hsl_end;

        let interp = HslInterpolator::new();

        //   test.equal(i(0.0), "rgb(191, 85, 64)");
        assert_eq!(
            Rgb {
                red: 191,
                green: 85,
                blue: 64
            },
            interp.interpolate_range(&range, 0.0).into()
        );

        //   test.equal(i(0.2), "rgb(191, 77, 64)");
        assert_eq!(
            Rgb {
                red: 191,
                green: 76,
                blue: 64
            },
            interp.interpolate_range(&range, 0.2).into()
        );

        //   test.equal(i(0.4), "rgb(191, 68, 64)");
        assert_eq!(
            Rgb {
                red: 191,
                green: 68,
                blue: 64
            },
            interp.interpolate_range(&range, 0.4).into()
        );

        //   test.equal(i(0.6), "rgb(191, 64, 68)");
        assert_eq!(
            Rgb {
                red: 191,
                green: 64,
                blue: 68
            },
            interp.interpolate_range(&range, 0.6).into()
        );

        //   test.equal(i(0.8), "rgb(191, 64, 77)");
        assert_eq!(
            Rgb {
                red: 191,
                green: 64,
                blue: 76
            },
            interp.interpolate_range(&range, 0.8).into()
        );

        //   test.equal(i(1.0), "rgb(191, 64, 85)");
        assert_eq!(
            Rgb {
                red: 191,
                green: 64,
                blue: 85
            },
            interp.interpolate_range(&range, 1.0).into()
        );
    }
}
