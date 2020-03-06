pub trait Threshold<DomainType>
where
    DomainType: Into<f64> + Clone + Copy,
{
    fn threshold(&self, domain: &[DomainType], min: DomainType, max: DomainType) -> usize;
}

#[derive(Debug)]
pub struct SturgesThreshold {}

impl<DomainType> Threshold<DomainType> for SturgesThreshold
where
    DomainType: Into<f64> + Clone + Copy,
{
    fn threshold(&self, domain: &[DomainType], _min: DomainType, _max: DomainType) -> usize {
        let mut work: Vec<f64> = domain
            .clone()
            .iter()
            .map(|d| (*d).into())
            .filter(|d: &f64| !d.is_nan())
            .collect::<Vec<f64>>();

        work.sort_by(|a, b| a.partial_cmp(b).unwrap());
        work.dedup();

        let count: f64 = match work.len() {
            c if c > ((1 << 53) - 1) => panic!("Float overflow"),
            c @ _ => c as f64,
        };

        (count.ln() / std::f64::consts::LN_2).ceil() as usize + 1
    }
}

#[test]
fn sturges() {
    let threshold = SturgesThreshold {};

    {
        let domain = vec![1, 2, 3, 4, 5];
        assert_eq!(threshold.threshold(&domain, 0, 0), 4);
    }

    {
        let domain = vec![1, 2, 3];
        assert_eq!(threshold.threshold(&domain, 0, 0), 3);
    }

    {
        let domain = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
        assert_eq!(threshold.threshold(&domain, 0, 0), 5);
    }
}
