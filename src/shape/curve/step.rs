//! `CurveStep` produces alternating horizontal and vertical lines.
//!
//! `CurveStep` creates a piecewise constant function (a.k.a. a step function)
//! consisting of alternating horizontal and vertical lines. The y-value
//! changes at the midpoint of each pair of adjacent x-values.

use pathfinder_canvas::Path2D;
use pathfinder_geometry::vector::Vector2F;

use super::{ CurveContext, CurveGenerator };

#[derive(Debug)]
pub struct CurveStep {}

impl CurveStep {
    pub fn new() -> Self {
        Self {}
    }
}

impl CurveGenerator for CurveStep {
    fn context(&self) -> Box<dyn CurveContext> {
        Box::new(CurveStepContext {
            line_state: 0,
            x: None,
            y: None,
            t: 0.5,
            path: Path2D::new()
        })
    }

    fn size_hint(&self, points_count: usize) -> usize {
        points_count
    }
}

#[derive(Debug)]
pub struct CurveStepContext {
    line_state: u8,
    x: Option<f64>,
    y: Option<f64>,
    t: f64,
    path: Path2D,
}

impl CurveContext for CurveStepContext {
    fn start_line(&mut self) {
        self.line_state = 0;
        self.x = None;
        self.y = None;
    }

    fn point(&mut self, x: f64, y: f64) {
        match self.line_state {
            0 => {
                self.line_state += 1;
                self.path.move_to(Vector2F::new(x as f32, y as f32));    
            },
            _ => {
                if self.line_state == 1 {
                    self.line_state += 1;
                }

                let x1 : f32 = (self.x.unwrap() * (1.0 - self.t) + x * self.t) as f32;
                let y_previous : f32 = self.y.unwrap() as f32;
                let y : f32 = y as f32;

                self.path.line_to(Vector2F::new(x1, y_previous));
                self.path.line_to(Vector2F::new(x1, y));
            },
        }

        self.x = Some(x);
        self.y = Some(y);
    }

    fn end_line(&mut self) {
        if 0.0 < self.t && self.t < 1.0 && self.line_state == 2 {
            self.path.line_to(Vector2F::new(self.x.unwrap() as f32, self.y.unwrap() as f32));
        }

        if self.line_state == 1 {
            self.path.close_path();
        }
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
        .curve(Box::new(CurveStep::new()));

    {
        let data : &[(f64, f64)] = &[(0., 1.)];
        assert_eq!("M 0 1 L 0 1 z", line.generate(data));
    }

    {
        let data : &[(f64, f64)] = &[(0., 1.), (2., 3.)];
        assert_eq!("M 0 1 L 1 1 L 1 3 L 2 3", line.generate(data));
    }

    {
        let data : &[(f64, f64)] = &[(0., 1.), (2., 3.), (4., 5.)];
        assert_eq!("M 0 1 L 1 1 L 1 3 L 3 3 L 3 5 L 4 5", line.generate(data));
    }

}
