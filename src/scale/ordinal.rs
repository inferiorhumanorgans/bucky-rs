#[derive(Clone, Debug)]
pub struct ScaleOrdinal<DomainType, RangeType> {
    pub domain: Vec<DomainType>,
    pub range: Vec<RangeType>,
}

impl<DomainType, RangeType> ScaleOrdinal<DomainType, RangeType>
where
    DomainType: PartialEq + Copy,
    RangeType: PartialEq + Copy,
{
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn domain(self, domain: Vec<DomainType>) -> Self {
        Self { domain, ..self }
    }

    pub fn range(self, range: Vec<RangeType>) -> Self {
        Self { range, ..self }
    }

    pub fn scale(&self, value: DomainType) -> RangeType {
        match self.domain.iter().position(|&x| x == value) {
            Some(idx) => self.range[(idx) % self.range.len()],
            None => panic!(),
        }
    }
}

impl<DomainType, RangeType> Default for ScaleOrdinal<DomainType, RangeType> {
    fn default() -> Self {
        Self {
            domain: vec![],
            range: vec![],
        }
    }
}
