use crate::error::Result;

use std::ops::Range;

/// Continuous scales map a continuous, quantitative input domain to a
/// continuous output range. If the range is also numeric, the mapping may be
/// inverted. The ScaleContinuous trait is implemented by the: [linear](crate::scale::ScaleLinear), power,
/// [log](crate::scale::ScaleLog), identity, radial, [time](crate::scale::ScaleTime) and sequential color scale structs.
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
        RangeType: Copy;

    fn ticks(&self, count: Option<i32>) -> Vec<DomainType>;
}
