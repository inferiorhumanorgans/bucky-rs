//! `CurveNatural` produces a natural cubic spline with the second derivative of
//! the spline set to zero at the endpoints.

use pathfinder_canvas::Path2D;
use pathfinder_geometry::vector::Vector2F;

use super::{CurveContext, CurveGenerator};

#[derive(Debug)]
pub struct CurveNatural {}

impl CurveNatural {
    pub fn new() -> Self {
        Self {}
    }
}

impl CurveGenerator for CurveNatural {
    fn context(&self) -> Box<dyn CurveContext> {
        Box::new(CurveNaturalContext {
            x: vec![],
            y: vec![],
            path: Path2D::new(),
        })
    }

    fn size_hint(&self, points_count: usize) -> usize {
        points_count
    }
}

#[derive(Debug)]
pub struct CurveNaturalContext {
    x: Vec<f64>,
    y: Vec<f64>,
    path: Path2D,
}

// See https://www.particleincell.com/2012/bezier-splines/ for derivation.
impl CurveNaturalContext {
    fn control_points(x: &[f64]) -> (Vec<f64>, Vec<f64>) {
        let n = x.len() - 1;

        assert!(n > 0);

        let mut a = vec![std::f64::NAN; n];
        let mut b = vec![std::f64::NAN; n];
        let mut r = vec![std::f64::NAN; n];
        let mut m: f64;

        a[0] = 0.0;
        b[0] = 2.0;
        r[0] = x[0] + 2.0 * x[1];

        for i in 1..n {
            a[i] = 1.0;
            b[i] = 4.0;
            r[i] = 4.0 * x[i] + 2.0 * x[i + 1];
        }

        a[n - 1] = 2.0;
        b[n - 1] = 7.0;
        r[n - 1] = 8.0 * x[n - 1] + x[n];

        for i in 1..n {
            m = a[i] / b[i - 1];
            b[i] -= m;
            r[i] -= m * r[i - 1];
        }

        a[n - 1] = r[n - 1] / b[n - 1];

        for i in (0..=(n - 2)).rev().step_by(1) {
            a[i] = (r[i] - a[i + 1]) / b[i];
        }

        b[n - 1] = (x[n] + a[n - 1]) / 2.;

        for i in 0..(n - 1) {
            b[i] = 2.0 * x[i + 1] - a[i + 1];
        }

        (a, b)
    }
}

impl CurveContext for CurveNaturalContext {
    fn start_line(&mut self) {
        self.x = vec![];
        self.y = vec![];
    }

    fn point(&mut self, x: f64, y: f64) {
        self.x.push(x);
        self.y.push(y);
    }

    fn end_line(&mut self) {
        let n = self.x.len();

        if n > 0 {
            self.path
                .move_to(Vector2F::new(self.x[0] as f32, self.y[0] as f32));
            if n == 2 {
                self.path
                    .line_to(Vector2F::new(self.x[1] as f32, self.y[1] as f32));
            } else if n > 1 {
                let px = CurveNaturalContext::control_points(self.x.as_slice());
                let py = CurveNaturalContext::control_points(self.y.as_slice());

                for (i1, i0) in (1..n).zip(0..) {
                    let ctrl0 = Vector2F::new(px.0[i0] as f32, py.0[i0] as f32);
                    let ctrl1 = Vector2F::new(px.1[i0] as f32, py.1[i0] as f32);
                    let to = Vector2F::new(self.x[i1] as f32, self.y[i1] as f32);
                    self.path.bezier_curve_to(ctrl0, ctrl1, to);
                }
            }
        }
        if n == 1 {
            self.path.close_path();
        }
        self.x = vec![];
        self.y = vec![];
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
            .curve(Box::new(CurveNatural::new()));

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

        // NOTE: Rust string formatting rounds with more precision than Javascript by default
        {
            // test.pathEqual(l([[0, 1], [1, 3], [2, 1]]), "M0,1C0.333333,2,0.666667,3,1,3C1.333333,3,1.666667,2,2,1");
            let data: &[(f64, f64)] = &[(0., 1.), (1., 3.), (2., 1.)];
            assert_eq!(
                "M 0 1 C 0.33333334 2 0.6666667 3 1 3 C 1.3333334 3 1.6666666 2 2 1",
                line.generate(data)
            );
        }

        {
            // test.pathEqual(l([[0, 1], [1, 3], [2, 1], [3, 3]]), "M0,1C0.333333,2.111111,0.666667,3.222222,1,3C1.333333,2.777778,1.666667,1.222222,2,1C2.333333,0.777778,2.666667,1.888889,3,3");
            let data: &[(f64, f64)] = &[(0., 1.), (1., 3.), (2., 1.), (3., 3.)];
            assert_eq!("M 0 1 C 0.33333334 2.1111112 0.6666667 3.2222223 1 3 C 1.3333334 2.7777777 1.6666666 1.2222222 2 1 C 2.3333333 0.7777778 2.6666667 1.8888888 3 3", line.generate(data));
        }
    }
}
