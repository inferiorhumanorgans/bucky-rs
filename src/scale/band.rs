#[derive(Clone, Debug)]
pub struct ScaleBand<DomainType> {
    pub domain: Vec<DomainType>,
    pub range: std::ops::Range<f64>,
    pub padding_inner: f64,
    pub padding_outer: f64,
    pub align: f64,
    pub values: Vec<f64>,
    pub band_width: f64,
}

impl<DomainType> ScaleBand<DomainType>
where
    DomainType: PartialEq + Copy + Clone,
{
    pub fn domain(self, domain: Vec<DomainType>) -> Self {
        Self { domain, ..self }
    }

    pub fn range<RangeIntermediateType>(self, range: std::ops::Range<RangeIntermediateType>) -> Self
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

    pub fn padding(self, padding: f64) -> Self {
        let padding_outer = self.padding_outer + padding;
        let padding_inner = self.padding_inner; //1_f64.min(padding_outer + padding);

        Self {
            padding_inner,
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

impl<DomainType> Default for ScaleBand<DomainType> {
    fn default() -> Self {
        Self {
            domain: vec![],
            range: 0.0..1.0,
            padding_inner: 0_f64,
            padding_outer: 0_f64,
            align: 0.5,
            values: vec![],
            band_width: 0_f64,
        }
    }
}
