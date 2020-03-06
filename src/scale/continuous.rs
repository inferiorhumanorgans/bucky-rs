use crate::error::Result;

use std::ops::Range;

pub trait ScaleContinuous<'a, DomainType, RangeType>
where
    Self: Sized,
    DomainType: PartialOrd,
{
    fn range<RangeIntermediateType>(self, range: Range<RangeIntermediateType>) -> Result<Self>
    where
        RangeIntermediateType: Into<RangeType>;

    fn clamped(self, clamped: bool) -> Self;

    fn domain<DomainIntermediateType>(self, domain: Range<DomainIntermediateType>) -> Result<Self>
    where
        DomainIntermediateType: PartialOrd + Into<DomainType>;

    fn nice<CountType>(self, count: Option<CountType>) -> Result<Self>
    where
        CountType: Into<i32>;

    fn scale<DomainIntermediateType>(&'a self, t: DomainIntermediateType) -> RangeType
    where
        DomainIntermediateType: Into<DomainType>,
        RangeType: Copy,;

    fn ticks(&self, count: Option<i32>) -> Vec<f64>;
}
