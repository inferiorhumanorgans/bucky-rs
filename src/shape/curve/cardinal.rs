//! `CurveCardinal` produces a cubic cardinal spline using the specified control points.
//!
//! The spline is created using the specified control points, with one-sided
//! differences used for the first and last piece. The default tension is 0.

use pathfinder_canvas::Path2D;
use pathfinder_geometry::vector::Vector2F;

use super::{CurveContext, CurveGenerator};

use std::f64::NAN;

#[derive(Debug)]
pub struct CurveCardinal {
    tension: f64,
}

impl CurveCardinal {
    pub fn new() -> Self {
        Self { tension: 0.0 }
    }

    pub fn with_tension(tension: f64) -> Self {
        Self { tension }
    }
}

impl CurveGenerator for CurveCardinal {
    fn context(&self) -> Box<dyn CurveContext> {
        Box::new(CurveCardinalContext {
            line_state: 0,
            x: [NAN, NAN, NAN],
            y: [NAN, NAN, NAN],
            tension: self.tension,
            path: Path2D::new(),
        })
    }

    fn size_hint(&self, points_count: usize) -> usize {
        points_count
    }
}

#[derive(Debug)]
pub struct CurveCardinalContext {
    line_state: u8,
    x: [f64; 3],
    y: [f64; 3],
    tension: f64,
    path: Path2D,
}

impl CurveCardinalContext {
    fn cardinal_point(&mut self, x: f64, y: f64) {
        let k = (1.0 - self.tension) / 6.0;

        self.path.bezier_curve_to(
            Vector2F::new(
                (self.x[1] + k * (self.x[2] - self.x[0])) as f32,
                (self.y[1] + k * (self.y[2] - self.y[0])) as f32,
            ),
            Vector2F::new(
                (self.x[2] + k * (self.x[1] - x)) as f32,
                (self.y[2] + k * (self.y[1] - y)) as f32,
            ),
            Vector2F::new((self.x[2]) as f32, (self.y[2]) as f32),
        );
    }
}

impl CurveContext for CurveCardinalContext {
    fn start_line(&mut self) {
        self.line_state = 0;
        self.x = [NAN, NAN, NAN];
        self.y = [NAN, NAN, NAN];
    }

    fn point(&mut self, x: f64, y: f64) {
        match self.line_state {
            0 => {
                self.line_state += 1;
                self.path.move_to(Vector2F::new(x as f32, y as f32));
            }
            1 => {
                self.line_state += 1;
                self.x[1] = x;
                self.y[1] = y;
            }
            2 => {
                self.line_state += 1;
                self.cardinal_point(x, y);
            }
            _ => self.cardinal_point(x, y),
        }
        self.x[0] = self.x[1];
        self.x[1] = self.x[2];
        self.x[2] = x;

        self.y[0] = self.y[1];
        self.y[1] = self.y[2];
        self.y[2] = y;
    }

    fn end_line(&mut self) {
        match self.line_state {
            3 => self.cardinal_point(self.x[1], self.y[1]),
            2 => self
                .path
                .line_to(Vector2F::new(self.x[2] as f32, self.y[2] as f32)),
            1 => self.path.close_path(),
            _ => unimplemented!(),
        }

        self.line_state -= 1;
    }

    fn path(&self) -> &Path2D {
        &self.path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expected_results() {
        use crate::shape::line::Line;

        let mut line = Line::<(f64, f64)>::new()
            .x(Box::new(|datum, _i| datum.0))
            .y(Box::new(|datum, _i| datum.1))
            .curve(Box::new(CurveCardinal::new()));

        {
            // test.pathEqual(l([[0, 1]]), "M0,1Z");
            let data: &[(f64, f64)] = &[(0., 1.)];
            assert_eq!("M 0 1 L 0 1 z", line.generate(data));
        }

        {
            // test.pathEqual(l([[0, 1], [1, 3]]), "M0,1L1,3");
            let data: &[(f64, f64)] = &[(0., 1.), (1., 3.)];
            assert_eq!("M 0 1 L 1 3", line.generate(data));
        }

        {
            // test.pathEqual(l([[0, 1], [1, 3], [2, 1]]), "M0,1C0,1,0.666667,3,1,3C1.333333,3,2,1,2,1");
            let data: &[(f64, f64)] = &[(0., 1.), (1., 3.), (2., 1.)];
            assert_eq!(
                "M 0 1 C 0 1 0.6666667 3 1 3 C 1.3333334 3 2 1 2 1",
                line.generate(data)
            );
        }

        {
            // test.pathEqual(l([[0, 1], [1, 3], [2, 1], [3, 3]]), "M0,1C0,1,0.666667,3,1,3C1.333333,3,1.666667,1,2,1C2.333333,1,3,3,3,3");
            let data: &[(f64, f64)] = &[(0., 1.), (1., 3.), (2., 1.), (3., 3.)];
            assert_eq!(
                "M 0 1 C 0 1 0.6666667 3 1 3 C 1.3333334 3 1.6666666 1 2 1 C 2.3333333 1 3 3 3 3",
                line.generate(data)
            );
        }
    }

    #[test]
    fn expected_results_with_different_tension() {
        use crate::shape::line::Line;

        let mut line = Line::<(f64, f64)>::new()
            .curve(Box::new(CurveCardinal::with_tension(0.5)))
            .x(Box::new(|datum, _i| datum.0))
            .y(Box::new(|datum, _i| datum.1));

        {
            // test.pathEqual(shape.line().curve(shape.curveCardinal.tension(0.5))([[0, 1], [1, 3], [2, 1], [3, 3]]), "M0,1C0,1,0.833333,3,1,3C1.166667,3,1.833333,1,2,1C2.166667,1,3,3,3,3");
            let data: &[(f64, f64)] = &[(0., 1.), (1., 3.), (2., 1.), (3., 3.)];
            assert_eq!(
                "M 0 1 C 0 1 0.8333333 3 1 3 C 1.1666666 3 1.8333334 1 2 1 C 2.1666667 1 3 3 3 3",
                line.generate(data)
            );
        }
    }
}
