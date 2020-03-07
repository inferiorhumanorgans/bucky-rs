use std::ops::Range;

use crate::array::ticks::{TickIncrement, Ticks};
#[cfg(feature = "color")]
use crate::color::Hsl;
use crate::error::{Result, ScaleError};
use crate::interpolate::*;
use crate::scale::continuous::*;

#[derive(Clone, Debug)]
pub struct ScaleLinear<RangeType, InterpolatorType> {
    pub domain: Range<f64>,
    pub range: Range<RangeType>,
    pub clamped: bool,
    pub interpolator: InterpolatorType,
}

impl<'a, RangeType, InterpolatorType> ScaleContinuous<'a, f64, RangeType>
    for ScaleLinear<RangeType, InterpolatorType>
where
    InterpolatorType: RangeInterpolator<'a, RangeType>,
{
    fn domain<DomainIntermediateType>(self, domain: Range<DomainIntermediateType>) -> Result<Self>
    where
        DomainIntermediateType: Into<f64> + PartialOrd,
    {
        if domain.start > domain.end {
            return Err(ScaleError::DescendingScale);
        }

        Ok(Self {
            domain: domain.start.into()..domain.end.into(),
            ..self
        })
    }

    fn range<RangeIntermediateType>(self, range: Range<RangeIntermediateType>) -> Result<Self>
    where
        RangeIntermediateType: Into<RangeType>,
    {
        Ok(Self {
            range: range.start.into()..range.end.into(),
            ..self
        })
    }

    fn clamped(self, clamped: bool) -> Self {
        Self { clamped, ..self }
    }

    fn nice<CountType>(self, count: Option<CountType>) -> Result<Self>
    where
        CountType: Into<i32>,
    {
        assert!(self.domain.end > self.domain.start);

        let count = match count {
            Some(count) => count.into(),
            None => 10,
        };

        let mut start = self.domain.start;
        let mut stop = self.domain.end;
        let mut step = self.domain.tick_increment(count);

        if step > 0.0 {
            start = (start / step).floor() * step;
            stop = (stop / step).ceil() * step;
            step = (start..stop).tick_increment(count);
        } else if step < 0.0 {
            start = (start * step).ceil() / step;
            stop = (stop * step).floor() / step;
            step = (start..stop).tick_increment(count);
        }

        if step > 0.0 {
            start = (start / step).floor() * step;
            stop = (stop / step).ceil() * step;
        } else if step < 0.0 {
            start = (start * step).ceil() / step;
            stop = (stop * step).floor() / step;
        }

        self.domain(start..stop)
    }

    fn scale<DomainIntermediateType>(&'a self, t: DomainIntermediateType) -> RangeType
    where
        DomainIntermediateType: Into<f64>,
        RangeType: Copy,
    {
        let t = t.into();

        let domain;
        let range;

        if self.domain.end < self.domain.start {
            domain = self.domain.end..self.domain.start;
            range = self.range.end..self.range.start;
        } else {
            domain = self.domain.start..self.domain.end;
            range = self.range.clone();
        };

        let clamped = match self.clamped {
            true => domain.start.max(domain.end.min(t)),
            false => t,
        };

        let normalized = (clamped - domain.start) / (domain.end - domain.start);

        self.interpolator.interpolate_range(&range, normalized)
    }

    fn ticks(&self, tick_count: Option<i32>) -> Vec<f64> {
        self.domain.ticks(tick_count)
    }
}

impl ScaleLinear<f64, NumberInterpolator> {
    pub fn new() -> Self {
        Self {
            domain: 0.0..1.0,
            range: 0.0..1.0,
            clamped: false,
            interpolator: NumberInterpolator::new(),
        }
    }
}

#[cfg(feature = "color")]
impl ScaleLinear<Hsl, HslInterpolator> {
    pub fn new() -> Self {
        Self {
            domain: 0.0..1.0,
            range: Hsl {
                hue: 0.0,
                saturation: 1.0,
                lightness: 0.5,
            }..Hsl {
                hue: 240.0,
                saturation: 1.0,
                lightness: 0.5,
            },
            clamped: false,
            interpolator: HslInterpolator::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults() -> Result<()> {
        let scale = ScaleLinear::<f64, _>::new();
        assert_eq!(scale.scale(2.0), 2.0);

        Ok(())
    }

    #[test]
    fn defaults_with_conversion() -> Result<()> {
        let scale = ScaleLinear::<f64, _>::new();
        assert_eq!(scale.scale(2), 2.0);

        Ok(())
    }

    #[test]
    fn clamped() -> Result<()> {
        const MIN: f64 = 0.0;
        const MAX: f64 = 1.0;

        let scale = ScaleLinear::<f64, _>::new().domain(MIN..MAX)?.clamped(true);

        assert_eq!(scale.scale(MAX * 2.0), scale.scale(MAX));

        Ok(())
    }

    #[cfg(feature = "color")]
    #[test]
    fn color() -> Result<()> {
        use crate::color::Hsl;

        let scale = ScaleLinear::<Hsl, _>::new();

        assert_eq!(
            scale.scale(1.0),
            Hsl {
                hue: 240.0,
                saturation: 1.0,
                lightness: 0.5
            }
        );

        Ok(())
    }

    #[cfg(feature = "color")]
    #[test]
    fn color_clamped() -> Result<()> {
        use crate::color::Hsl;

        const MIN: f64 = 0.0;
        const MAX: f64 = 1.0;

        let scale = ScaleLinear::<Hsl, _>::new().domain(MIN..MAX)?.clamped(true);
        assert_eq!(scale.scale(MAX * 2.5), scale.scale(MAX));

        Ok(())
    }

    #[cfg(feature = "color")]
    #[test]
    fn color_mid() -> Result<()> {
        use crate::color::Hsl;

        let scale = ScaleLinear::<Hsl, _>::new();

        assert_eq!(
            scale.scale(0.330),
            Hsl {
                hue: 320.4,
                saturation: 1.0,
                lightness: 0.5
            }
        );

        Ok(())
    }
}
