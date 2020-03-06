//! `CurveBasis` produces a cubic basis spline using the specified control points.
//!
//! The first and last points are triplicated such that the spline starts at the
//! first point and ends at the last point, and is tangent to the line between
//! the first and second points, and to the line between the penultimate and
//! last points.

use pathfinder_canvas::Path2D;
use pathfinder_geometry::vector::Vector2F;

use super::{CurveContext, CurveGenerator};

use std::f64::NAN;

#[derive(Debug)]
pub struct CurveBasis {}

impl CurveBasis {
    pub fn new() -> Self {
        Self {}
    }
}

impl CurveGenerator for CurveBasis {
    fn context(&self) -> Box<dyn CurveContext> {
        Box::new(CurveBasisContext {
            line_state: 0,
            x0: NAN,
            x1: NAN,
            y0: NAN,
            y1: NAN,
            path: Path2D::new(),
        })
    }

    fn size_hint(&self, points_count: usize) -> usize {
        points_count
    }
}

#[derive(Debug)]
pub struct CurveBasisContext {
    line_state: u8,
    x0: f64,
    x1: f64,
    y0: f64,
    y1: f64,
    path: Path2D,
}

impl CurveBasisContext {
    fn basis_point(&mut self, x: f64, y: f64) {
        self.path.bezier_curve_to(
            Vector2F::new(
                ((2.0 * self.x0 + self.x1) / 3.0) as f32,
                ((2.0 * self.y0 + self.y1) / 3.0) as f32,
            ),
            Vector2F::new(
                ((self.x0 + 2.0 * self.x1) / 3.0) as f32,
                ((self.y0 + 2.0 * self.y1) / 3.0) as f32,
            ),
            Vector2F::new(
                ((self.x0 + 4.0 * self.x1 + x) / 6.0) as f32,
                ((self.y0 + 4.0 * self.y1 + y) / 6.0) as f32,
            ),
        );
    }
}

impl CurveContext for CurveBasisContext {
    fn start_line(&mut self) {
        self.line_state = 0;
        self.x0 = NAN;
        self.x1 = NAN;
        self.y0 = NAN;
        self.y1 = NAN;
    }

    fn point(&mut self, x: f64, y: f64) {
        match self.line_state {
            0 => {
                self.line_state += 1;
                self.path.move_to(Vector2F::new(x as f32, y as f32));
            }
            1 => self.line_state += 1,
            2 => {
                self.line_state += 1;
                self.path.line_to(Vector2F::new(
                    ((5.0 * self.x0 + self.x1) / 6.0) as f32,
                    ((5.0 * self.y0 + self.y1) / 6.0) as f32,
                ));
                self.basis_point(x, y);
            }
            _ => self.basis_point(x, y),
        }
        self.x0 = self.x1;
        self.x1 = x;
        self.y0 = self.y1;
        self.y1 = y;
    }

    fn end_line(&mut self) {
        // TODO: There has got to be a more elegant way…
        if self.line_state == 3 {
            self.basis_point(self.x1, self.y1);
        }

        if self.line_state == 2 || self.line_state == 3 {
            self.path
                .line_to(Vector2F::new(self.x1 as f32, self.y1 as f32));
        }

        if self.line_state == 1 {
            self.path.close_path();
        }
        self.line_state -= 1;
    }

    fn path(&self) -> &Path2D {
        &self.path
    }
}

#[test]
fn expected_results() {
    use crate::shape::line::Line;

    let mut line = Line::<(f64, f64)>::new()
        .x(Box::new(|datum, _i| datum.0))
        .y(Box::new(|datum, _i| datum.1))
        .curve(Box::new(CurveBasis::new()));

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
        // test.pathEqual(l([[0, 1], [1, 3], [2, 1]]), "M0,1L0.166667,1.333333C0.333333,1.666667,0.666667,2.333333,1,2.333333C1.333333,2.333333,1.666667,1.666667,1.833333,1.333333L2,1");
        let data: &[(f64, f64)] = &[(0., 1.), (1., 3.), (2., 1.)];
        assert_eq!("M 0 1 L 0.16666667 1.3333334 C 0.33333334 1.6666666 0.6666667 2.3333333 1 2.3333333 C 1.3333334 2.3333333 1.6666666 1.6666666 1.8333334 1.3333334 L 2 1", line.generate(data));
    }
}
