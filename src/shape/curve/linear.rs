//! `CurveLinear` produces a polyline through the specified points.

use pathfinder_canvas::Path2D;
use pathfinder_geometry::vector::Vector2F;

use super::{CurveContext, CurveGenerator};

#[derive(Debug)]
pub struct CurveLinear {}

impl CurveLinear {
    pub fn new() -> Self {
        Self {}
    }
}

impl CurveGenerator for CurveLinear {
    fn context(&self) -> Box<dyn CurveContext> {
        Box::new(CurveLinearContext {
            line_state: 0,
            path: Path2D::new(),
        })
    }

    fn size_hint(&self, points_count: usize) -> usize {
        points_count
    }
}

#[derive(Debug)]
pub struct CurveLinearContext {
    line_state: u8,
    path: Path2D,
}

impl CurveContext for CurveLinearContext {
    fn start_line(&mut self) {
        self.line_state = 0;
    }

    fn point(&mut self, x: f64, y: f64) {
        match self.line_state {
            0 => {
                self.line_state += 1;
                self.path.move_to(Vector2F::new(x as f32, y as f32));
            }
            1 => {
                self.line_state += 1;
                self.path.line_to(Vector2F::new(x as f32, y as f32));
            }
            _ => self.path.line_to(Vector2F::new(x as f32, y as f32)),
        }
    }

    fn end_line(&mut self) {
        if self.line_state == 1 {
            self.path.close_path();
        }
        self.line_state -= 1;
    }

    fn path(&self) -> &Path2D {
        &self.path
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn expected_results() {
        use crate::shape::line::Line;

        let mut line = Line::<(f64, f64)>::new()
            .x(Box::new(|datum, _i| datum.0))
            .y(Box::new(|datum, _i| datum.1))
            .defined(Box::new(|datum, _i| !datum.0.is_nan() && !datum.1.is_nan()));

        {
            // test.pathEqual(l([[0, 1]]), "M0,1Z");
            let data: &[(f64, f64)] = &[(0., 1.)];
            assert_eq!("M 0 1 L 0 1 z", line.generate(data));
        }

        {
            // test.pathEqual(l([[0, 1], [2, 3]]), "M0,1L2,3");
            let data: &[(f64, f64)] = &[(0., 1.), (2., 3.)];
            assert_eq!("M 0 1 L 2 3", line.generate(data));
        }

        {
            // test.pathEqual(l([[0, 1], [2, 3], [4, 5]]), "M0,1L2,3L4,5");
            let data: &[(f64, f64)] = &[(0., 1.), (2., 3.), (4., 5.)];
            assert_eq!("M 0 1 L 2 3 L 4 5", line.generate(data));
        }
    }
}
