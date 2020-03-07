use std::ops::Range;

/// Band scales are like [ordinal](crate::scale::ScaleOrdinal) scales except the output range is continuous
/// and numeric. Discrete output values are automatically computed by the scale
/// by dividing the continuous range into uniform bands. Band scales are
/// typically used for bar charts with an ordinal or categorical dimension.
#[derive(Clone, Debug)]
pub struct ScaleBand<'a, DomainType> {
    pub domain: &'a [DomainType],
    pub range: Range<f64>,
    pub padding_inner: f64,
    pub padding_outer: f64,
    pub align: f64,
    pub values: Vec<f64>,
    pub band_width: f64,
    pub step: f64,
}

impl<'a, DomainType> ScaleBand<'a, DomainType>
where
    DomainType: PartialEq + Copy + Clone,
{
    pub fn domain(self, domain: &'a [DomainType]) -> Self {
        Self { domain, ..self }.recalc()
    }

    pub fn range<RangeIntermediateType>(self, range: Range<RangeIntermediateType>) -> Self
    where
        RangeIntermediateType: Into<f64>,
    {
        Self {
            range: range.start.into()..range.end.into(),
            ..self
        }
        .recalc()
    }

    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    /// A convenience method for setting the inner and outer padding to the same
    /// padding value. If padding is not specified, returns the inner padding.
    pub fn padding(self, padding: f64) -> Self {
        let padding_inner = 1_f64.min(padding);
        let padding_outer = padding;

        Self {
            padding_inner,
            padding_outer,
            ..self
        }
        .recalc()
    }

    /// The inner padding specifies the proportion of the range that is reserved
    /// for blank space between bands; a value of 0 means no blank space between
    /// bands, and a value of 1 means a bandwidth of zero.  Inner padding must
    /// be less than or equal to 1.
    pub fn padding_inner(self, padding: f64) -> Self {
        let padding_inner = 1_f64.min(padding);

        Self {
            padding_inner,
            ..self
        }
        .recalc()
    }

    /// The outer padding specifies the amount of blank space, in terms of
    /// multiples of the step, to reserve before the first band and after the
    /// last band. Outer padding is typically in the range [0, 1].
    pub fn padding_outer(self, padding: f64) -> Self {
        let padding_outer = padding;

        Self {
            padding_outer,
            ..self
        }
        .recalc()
    }

    fn recalc(self) -> Self {
        let n = self.domain.len();
        let step = (self.range.end - self.range.start) as f64
            / 1_f64.max(n as f64 - self.padding_inner + self.padding_outer * 2_f64);
        let mut start = self.range.start;
        start +=
            (self.range.end - self.range.start - (step as f64) * (n as f64 - self.padding_inner))
                * self.align;

        let band_width = step * (1_f64 - self.padding_inner);

        let values = (0..n).map(|x| start + step as f64 * x as f64).collect();

        Self {
            band_width,
            step,
            values,
            ..self
        }
    }

    pub fn scale(&self, value: DomainType) -> f64 {
        match self.domain.iter().position(|&x| x == value) {
            Some(idx) => self.values[(idx) % self.values.len()],
            None => panic!(),
        }
    }
}

impl<'a, DomainType> Default for ScaleBand<'a, DomainType>
where
    DomainType: PartialEq + Copy + Clone,
{
    fn default() -> Self {
        Self {
            domain: &[],
            range: 0.0..1.0,
            padding_inner: 0_f64,
            padding_outer: 0_f64,
            align: 0.5,
            values: vec![],
            band_width: std::f64::NAN,
            step: std::f64::NAN,
        }.recalc()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expected_defaults() {
        let scale = ScaleBand::<i32>::new();

        let annotation : &[i32] = &[];
        assert_eq!(annotation, scale.domain);
        assert_eq!(0.0..1.0, scale.range);
        assert_eq!(1.0, scale.band_width);
        assert_eq!(1.0, scale.step);
        assert_eq!(0.0, scale.padding_inner);
        assert_eq!(0.0, scale.padding_outer);
        assert_eq!(0.5, scale.align);

        //     test.equal(s.round(), false);
    }

    #[test]
    fn computes_discrete_bands_in_a_continuous_range() {
        let scale = ScaleBand::<&str>::new()
            .range(0..960);

        {
            let scale = scale.clone().domain(&["foo", "bar"]);
            assert_eq!(0.0, scale.scale("foo"));
            assert_eq!(480.0, scale.scale("bar"));
        }

        {
            let domain = &["a", "b", "c"];
            let expected1 = &[0.0, 40.0, 80.0];
            let expected2 = &[7.5, 45.0, 82.5];

            let scale = scale.clone()
                .domain(domain)
                .range(0..120);

            for (d, r) in domain.iter().zip(expected1) {
                assert_eq!(*r, scale.scale(d))
            }

            assert_eq!(scale.band_width, 40.0);

            let scale = scale.padding(0.2);

            for (d, r) in domain.iter().zip(expected2) {
                assert_eq!(*r, scale.scale(d))
            }
            assert_eq!(scale.band_width, 30.0);
        }
    }

    #[test]
    fn step_returns_the_distance_between_the_starts_of_adjacent_bands() {
        let scale = ScaleBand::<&str>::new()
            .range(0..960);

        {
            let scale = scale.clone().domain(&["foo"]);
            assert_eq!(scale.step, 960.0);
        }

        {
            let scale = scale.clone().domain(&["foo", "bar"]);
            assert_eq!(scale.step, 480.0);
        }

        {
            let scale = scale.clone().domain(&["foo", "bar", "baz"]);
            assert_eq!(scale.step, 320.0);   
        }

        let scale = scale.padding(0.5);

        {
            let scale = scale.clone().domain(&["foo"]);
            assert_eq!(scale.step, 640.0);
        }

        {
            let scale = scale.clone().domain(&["foo", "bar"]);
            assert_eq!(scale.step, 384.0);
        }
    }

    #[test]
    fn band_width_returns_the_width_of_the_band() {
        let scale = ScaleBand::<&str>::new()
            .range(0..960);

        {
            let scale = scale.clone().domain(&[]);
            assert_eq!(scale.band_width, 960.0);
        }

        {
            let scale = scale.clone().domain(&["foo"]);
            assert_eq!(scale.band_width, 960.0);
        }

        {
            let scale = scale.clone().domain(&["foo", "bar"]);
            assert_eq!(scale.band_width, 480.0);
        }

        {
            let scale = scale.clone().domain(&["foo", "bar", "baz"]);
            assert_eq!(scale.band_width, 320.0);
        }

        let scale = scale.padding(0.5);

        {
            let scale = scale.clone().domain(&[]);
            assert_eq!(scale.band_width, 480.0);
        }

        {
            let scale = scale.clone().domain(&["foo"]);
            assert_eq!(scale.band_width, 320.0);
        }

        {
            let scale = scale.clone().domain(&["foo", "bar"]);
            assert_eq!(scale.band_width, 192.0);
        }

    }
}
