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
    fn tick_increment(&self, count: i32) -> ReturnType;
}

pub trait TickStep {
    fn tick_step(&self, count: i32) -> f64;
}
