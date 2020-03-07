#[derive(Clone, Debug)]
pub struct ScaleOrdinal<'a, DomainType, RangeType> {
    pub domain: &'a [DomainType],
    pub range: &'a [RangeType],
}

impl<'a, DomainType, RangeType> ScaleOrdinal<'a, DomainType, RangeType>
where
    DomainType: PartialEq + Copy,
    RangeType: PartialEq + Copy,
{
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn domain(self, domain: &'a [DomainType]) -> Self {
        Self { domain, ..self }
    }

    pub fn range(self, range: &'a [RangeType]) -> Self {
        Self { range, ..self }
    }

    pub fn scale(&self, value: DomainType) -> RangeType {
        match self.domain.iter().position(|&x| x == value) {
            Some(idx) => self.range[(idx) % self.range.len()],
            None => panic!(),
        }
    }
}

impl<'a, DomainType, RangeType> Default for ScaleOrdinal<'a, DomainType, RangeType> {
    fn default() -> Self {
        Self {
            domain: &[],
            range: &[],
        }
    }
}
