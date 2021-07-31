mod number;
#[cfg(feature = "time")]
mod time;
#[cfg(feature = "time")]
mod time_duration;

pub use number::*;
#[cfg(feature = "time")]
pub use time::*;
#[cfg(feature = "time")]
pub use time_duration::*;

pub trait Ticks {
    fn ticks(&self, count: Option<i32>) -> Vec<f64>;
}

pub trait TickIncrement<DomainType, ReturnType> {
    /// Like tick_step, except requires that *start* is always less than or
    /// equal to *step*, and if the tick step for the given *start*, *stop*
    /// and *count* would be less than one, returns the negative inverse tick
    /// step instead. This method is always guaranteed to return an integer,
    /// and is used by d3.ticks to guarantee that the returned tick
    /// values are represented as precisely as possible in IEEE 754 floating
    /// point.
    fn tick_increment(&self, count: i32) -> ReturnType;
}

pub trait TickStep {
    /// Returns the difference between adjacent tick values if the same
    /// arguments were passed to d3.ticks: a nicely-rounded value that is a
    /// power of ten multiplied by 1, 2 or 5. Note that due to the limited
    /// precision of IEEE 754 floating point, the returned value may not be
    /// exact decimals.
    fn tick_step(&self, count: i32) -> f64;
}
