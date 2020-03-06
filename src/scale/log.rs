use std::ops::Range;

use crate::array::ticks::Ticks;
use crate::error::{Result, ScaleError};
use crate::interpolate::*;
use crate::scale::continuous::*;

#[cfg(test)]
use crate::{assert_delta, test::DELTA};

/// Logarithmic scales are similar to linear scales, except a logarithmic
/// transform is applied to the input domain value before the output range
/// value is computed. The mapping to the range value y can be expressed as
/// a function of the domain value x: y = m log(x) + b.
///
/// As log(0) = -∞, a log scale domain must be strictly-positive or strictly-
/// negative; the domain must not include or cross zero. A log scale with a
/// positive domain has a well-defined behavior for positive values, and a log
/// scale with a negative domain has a well-defined behavior for negative
/// values. For a negative domain, input and output values are implicitly
/// multiplied by -1. The behavior of the scale is undefined if you pass a
/// negative value to a log scale with a positive domain or vice versa.
///
/// TODO: Handle negative scales.
#[derive(Clone, Debug)]
pub struct ScaleLog<RangeType, InterpolatorType> {
    pub domain: Range<f64>,
    pub range: Range<RangeType>,
    pub clamped: bool,
    pub base: f64,
    pub interpolator: InterpolatorType,
}

impl<'a, RangeType, InterpolatorType> ScaleLog<RangeType, InterpolatorType>
where
    InterpolatorType: RangeInterpolator<'a, RangeType>,
{
    pub fn interpolator<NewInterpolator>(
        self,
        interpolator: NewInterpolator,
    ) -> ScaleLog<RangeType, NewInterpolator>
    where
        NewInterpolator: RangeInterpolator<'a, RangeType>,
    {
        let ret: ScaleLog<RangeType, NewInterpolator> = ScaleLog {
            interpolator,
            domain: self.domain,
            range: self.range,
            clamped: self.clamped,
            base: self.base,
        };
        ret
    }

    pub fn base(self, base: f64) -> Self {
        Self { base, ..self }
    }

    /// Like LinearScale#tick_format, but customized for a log scale. The specified
    /// count typically has the same value as the count that is used to generate
    /// the tick values. If there are too many ticks, the formatter may return
    /// the empty string for some of the tick labels; however, note that the
    /// ticks are still shown.
    ///
    /// TODO: To disable filtering, specify a count of Infinity.
    ///
    /// TODO: When specifying a count, you may also provide a format specifier
    /// or format function. For example, to get a tick formatter that will
    /// display 20 ticks of a currency, say log.tickFormat(20, "$,f"). If the
    /// specifier does not have a defined precision, the precision will be set
    /// automatically by the scale, returning the appropriate format. This
    /// provides a convenient way of specifying a format whose precision will
    /// be automatically set by the scale. This should probably be gated by a
    /// crate feature.
    pub fn tick_format(&'a self, count: Option<i32>) -> impl FnMut(&f64) -> String {
        let count = match count {
            Some(count) => count,
            None => 10,
        };

        let base = self.base;

        let k = 1_f64.max(self.base * count as f64 / self.ticks(None).len() as f64);

        move |d| {
            let mut i = d / base.powf(d.log(base).round());
            if i * base < base - 0.5 {
                i *= base;
            }
            if i <= k {
                if base == 10.0 {
                    format!("{:.0e}", d)
                } else {
                    format!("{}", d)
                }
            } else {
                "".into()
            }
        }
    }
}

impl<'a, RangeType, DefaultInterpolator> ScaleContinuous<'a, f64, RangeType>
    for ScaleLog<RangeType, DefaultInterpolator>
where
    DefaultInterpolator: RangeInterpolator<'a, RangeType>,
{
    fn domain<DomainIntermediateType>(self, domain: Range<DomainIntermediateType>) -> Result<Self>
    where
        DomainIntermediateType: Into<f64> + PartialOrd,
    {
        let domain = domain.start.into()..domain.end.into();

        if !((domain.end < 0.0 && domain.start < 0.0) || (domain.end > 0.0 && domain.start > 0.0)) {
            return Err(ScaleError::DegenerateDomain);
        }

        Ok(Self { domain, ..self })
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

    fn nice<CountType>(self, _count: Option<CountType>) -> Result<Self>
    where
        CountType: Into<i32>,
    {
        unimplemented!()
    }

    fn scale<DomainIntermediateType>(&'a self, t: DomainIntermediateType) -> RangeType
    where
        DomainIntermediateType: Into<f64>,
        RangeType: Copy,
    {
        let t = t.into();

        if self.domain.end > self.domain.start {
            let domain = self.domain.start..self.domain.end;

            let clamped = match self.clamped {
                true => domain.start.max(domain.end.min(t)),
                false => t,
            };

            let domain_start = domain.start.ln();
            let domain_end = domain.end.ln();
            let clamped = clamped.ln();

            let normalized = (clamped - domain_start) / (domain_end - domain_start);

            return self.interpolator.interpolate_range(&self.range, normalized);
        } else {
            let domain = self.domain.end..self.domain.start;

            let clamped = match self.clamped {
                true => domain.start.max(domain.end.min(t)),
                false => t,
            };

            let domain_start = domain.start.ln();
            let domain_end = domain.end.ln();
            let clamped = clamped.ln();

            let normalized = (clamped - domain_start) / (domain_end - domain_start);

            let range = self.range.end..self.range.start;
            self.interpolator.interpolate_range(&range, normalized)
        }
    }

    /// Like ticks for LinearScale, but customized for a log scale. If the base
    /// is an integer, the returned ticks are uniformly spaced within each
    /// integer power of base; otherwise, one tick per power of base is
    /// returned. The returned ticks are guaranteed to be within the extent of
    /// the domain. If the orders of magnitude in the domain is greater than
    /// count, then at most one tick per power is returned. Otherwise, the tick
    /// values are unfiltered.  If count is not specified, it defaults to 10.
    fn ticks(&self, tick_count: Option<i32>) -> Vec<f64> {
        assert!(self.domain.end > self.domain.start);

        let i = self.domain.start.log(self.base);
        let j = self.domain.end.log(self.base);

        let n = match tick_count {
            Some(n) => n,
            None => 10,
        };

        let mut z: Vec<f64> = vec![];

        if self.base % 1.0 == 0.0 {
            let base = self.base.floor() as i32;
            if (j - i) < n as f64 {
                let i = i.floor() as i32;
                let j = j.ceil() as i32;
                if self.domain.start > 0.0 {
                    for i in i..=j {
                        let p = base.pow(i as u32);
                        for k in 1..base {
                            let t = (p * k) as f64;
                            if t < self.domain.start {
                                continue;
                            }
                            if t > self.domain.end {
                                break;
                            }
                            z.push(t);
                        }
                    }
                } else {
                    unimplemented!()
                }
                if (z.len() * 2) < n as usize {
                    return self.domain.ticks(Some(n));
                }
                return z;
            }
        } else {
            let tick_count = (j - i).min(n as f64).floor();
            return (i..j)
                .ticks(Some(tick_count as i32))
                .iter()
                .map(|n| self.base.powf(*n))
                .collect();
        }

        unimplemented!();
    }
}

impl<'a> ScaleLog<f64, NumberInterpolator> {
    pub fn new() -> Self {
        Self {
            domain: 1.0..10.0,
            range: 0.0..1.0,
            clamped: false,
            base: 10.0,
            interpolator: NumberInterpolator::new(),
        }
    }
}

#[test]
fn expected_defaults() -> Result<()> {
    let scale = ScaleLog::new();

    assert_eq!(1.0..10.0, scale.domain);
    assert_eq!(0.0..1.0, scale.range);
    assert_delta!(0.69897, scale.scale(5), DELTA);
    assert_delta!(0.5, scale.scale(3.162278), DELTA);

    Ok(())
}

// tape("log.domain(…) can take negative values", function(test) {
// tape("log.domain(…) preserves specified domain exactly, with no floating point error", function(test) {
// tape("log.interpolate(f) sets the interpolator", function(test) {

#[test]
fn log_scale_does_not_clamp_by_default() -> Result<()> {
    let scale = ScaleLog::new();

    assert_eq!(false, scale.clamped);
    assert_delta!(-0.3010299, scale.scale(0.5), DELTA);
    assert_delta!(1.1760913, scale.scale(15), DELTA);

    Ok(())
}

#[test]
fn log_scale_clamp_true_clamps_to_the_domain() -> Result<()> {
    let scale = ScaleLog::new().clamped(true);

    assert_delta!(0.0, scale.scale(-1.0), DELTA);
    assert_delta!(0.69897, scale.scale(5), DELTA);
    assert_delta!(1.0, scale.scale(15), DELTA);

    let scale = scale.domain(10..1)?;

    assert_delta!(1.0, scale.scale(-1), DELTA);
    assert_delta!(0.30103, scale.scale(5), DELTA);
    assert_delta!(0.0, scale.scale(15), DELTA);

    Ok(())
}

#[test]
fn x_is_mapped_to_y() -> Result<()> {
    let scale = ScaleLog::new().domain(1..2)?;

    assert_delta!(-1.0000000, scale.scale(0.5), DELTA);
    assert_delta!(0.0000000, scale.scale(1.0), DELTA);
    assert_delta!(0.5849625, scale.scale(1.5), DELTA);
    assert_delta!(1.0000000, scale.scale(2.0), DELTA);
    assert_delta!(1.3219281, scale.scale(2.5), DELTA);

    Ok(())
}

#[test]
fn base_changes_ticks() -> Result<()> {
    let scale = ScaleLog::new().domain(1..32)?;

    {
        let scale = scale.clone().base(2.0);
        assert_eq!(
            &[1.0, 2.0, 4.0, 8.0, 16.0, 32.0],
            scale.ticks(None).as_slice()
        );
    }

    {
        let scale = scale.clone().base(std::f64::consts::E);
        assert_eq!(
            &[
                1.0,
                2.718281828459045,
                7.3890560989306495,
                20.085536923187664
            ],
            scale.ticks(None).as_slice()
        );
    }

    Ok(())
}

#[test]
fn base_changes_ticks_and_format_gives_strings() -> Result<()> {
    let scale = ScaleLog::new().domain(1..32)?;

    {
        let scale = scale.clone().base(2.0);
        let expected_ticks = &["1", "2", "4", "8", "16", "32"];
        let ticks: Vec<_> = scale
            .ticks(None)
            .iter()
            .map(scale.tick_format(None))
            .collect();
        assert_eq!(expected_ticks, ticks.as_slice());
    }

    {
        let scale = scale.clone().base(std::f64::consts::E);
        let expected_ticks = &[
            "1",
            "2.718281828459045",
            "7.3890560989306495",
            "20.085536923187664",
        ];
        let ticks: Vec<_> = scale
            .ticks(None)
            .iter()
            .map(scale.tick_format(None))
            .collect();
        assert_eq!(expected_ticks, ticks.as_slice());
    }

    Ok(())
}

// tape("log.nice() nices the domain, extending it to powers of ten", function(test) {
// tape("log.nice() on a polylog domain only affects the extent", function(test) {
// tape("log.ticks() generates the expected power-of-ten for ascending ticks", function(test) {
// tape("log.ticks() generates the expected power-of-ten ticks for descending domains", function(test) {
// tape("log.ticks() generates the expected power-of-ten ticks for small domains", function(test) {
// tape("log.ticks() generates linear ticks when the domain extent is small", function(test) {
// tape("log.base(base).ticks() generates the expected power-of-base ticks", function(test) {
// tape("log.tickFormat() is equivalent to log.tickFormat(10)", function(test) {
// tape("log.tickFormat(count) returns a filtered \".0e\" format", function(test) {
// tape("log.tickFormat(count, format) returns the specified format, filtered", function(test) {
// tape("log.base(base).tickFormat() returns the \",\" format", function(test) {
// tape("log.base(base).tickFormat(count) returns a filtered \",\" format", function(test) {
// tape("log.ticks() generates log ticks", function(test) {
// tape("log.tickFormat(count) filters ticks to about count", function(test) {
// tape("log.ticks(count) filters powers-of-ten ticks for huge domains", function(test) {
// tape("log.ticks() generates ticks that cover the domain", function(test) {
// tape("log.ticks() generates ticks that cover the niced domain", function(test) {

#[test]
fn degenerate_domain_cannot_be_set() -> Result<()> {
    let scale = ScaleLog::new();

    assert!(scale.clone().domain(0..1).is_err());
    assert!(scale.clone().domain(1..0).is_err());
    assert!(scale.clone().domain(0..-1).is_err());
    assert!(scale.clone().domain(-1..0).is_err());
    assert!(scale.clone().domain(-1..1).is_err());
    assert!(scale.clone().domain(0..0).is_err());

    Ok(())
}

#[test]
fn ensure_normalization_works() {
    let scale = ScaleLog::new()
        .interpolator(RoundInterpolator::new())
        .domain(78.2..636.23)
        .unwrap()
        .range(550.0..10.0)
        .unwrap();

    assert_eq!(10.0, scale.base);
    assert_eq!(78.2..636.23, scale.domain);
    assert_eq!(550.0..10.0, scale.range);

    assert_eq!(547.0, scale.scale(79.0));
    assert_eq!(1080.0, scale.scale(10.0));
}
