use std::fmt;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Hsl {
    pub hue: f64,
    pub saturation: f64,
    pub lightness: f64,
}

impl fmt::Display for Hsl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "hsl({}, {}%, {}%)",
            self.hue,
            self.saturation * 100.0,
            self.lightness * 100.0
        )
    }
}

impl Hsl {
    pub fn to_rgb(self) -> Rgb {
        self.into()
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Rgb {
    pub red: u16,
    pub green: u16,
    pub blue: u16,
}

impl From<Hsl> for Rgb {
    fn from(hsl: Hsl) -> Self {
        let s: f64 = hsl.saturation;
        let l: f64 = hsl.lightness;

        let a: f64 = s * l.min(1.0 - l);

        let foo = |n: u16| {
            let k: f64 = (n as f64 + hsl.hue / 30.0) % 12.0;
            let positions = [k - 3.0, 9.0 - k, 1.0];
            let min = positions
                .iter()
                .copied()
                .min_by(|x, y| x.partial_cmp(y).expect("You promised no naan today"))
                .unwrap();

            (l - a * min.max(-1.0)) * 255.0
        };
        Rgb {
            red: foo(0).round() as u16,
            green: foo(8).round() as u16,
            blue: foo(4).round() as u16,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hsl_formatting() {
        let hsl = Hsl {
            hue: 300.0,
            saturation: 0.5,
            lightness: 0.4,
        };
        assert_eq!(format!("{}", hsl), "hsl(300, 50%, 40%)")
    }
}
