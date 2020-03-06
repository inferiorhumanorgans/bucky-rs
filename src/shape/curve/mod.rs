//! Curves interpolate discrete data typically for the purpose of drawing lines.

use pathfinder_canvas::Path2D;

pub trait CurveGenerator {
    fn context(&self) -> Box<dyn CurveContext>;
    fn size_hint(&self, points_count: usize) -> usize;
}

pub trait CurveContext {
    fn start_line(&mut self);
    fn point(&mut self, x: f64, y: f64);
    fn end_line(&mut self);
    fn path(&self) -> &Path2D;
}

mod basis;
mod cardinal;
mod linear;
mod natural;
mod step;

pub use basis::*;
pub use cardinal::*;
pub use linear::*;
pub use natural::*;
pub use step::*;
