#[derive(Clone, Debug)]
pub struct ScaleQuantile<'a, RangeType> {
    pub domain: Vec<f64>,
    pub range: &'a [RangeType],
    pub thresholds: Vec<f64>,
}

impl<'a, RangeType> ScaleQuantile<'a, RangeType>
{
    pub fn domain<DomainIntermediateType>(self, domain: &[DomainIntermediateType]) -> Self
    where DomainIntermediateType: Into<f64> + Clone
    {

        let mut domain : Vec<f64> = domain.iter()
            .cloned()
            .map(|datum| datum.into())
            .filter(|datum: &f64| !datum.is_nan())
            .collect();
        domain.sort_by(|a, b| a.partial_cmp(b).expect("We already filtered out NaN. We shouldn't be here."));

        Self { domain, ..self }.rescale()
    }

    pub fn range(self, range: &'a [RangeType]) -> Self {
        Self { range, ..self }.rescale()
    }

    fn rescale(self) -> Self {
        let thresholds = {
            use crate::array::quantile::Quantileable;

            let n = std::cmp::max(1, self.range.len());
            let mut thresholds = vec![0_f64; n - 1];
            for i in 0..thresholds.len() {
                thresholds[i] = self.domain.quantile((i + 1) as f64 / n as f64);
            }
            thresholds
        };

        Self { thresholds, ..self }
    }

    pub fn scale<DomainIntermediateType>(&self, n: DomainIntermediateType) -> &'a RangeType
    where
        DomainIntermediateType: Into<f64>
    {
        let n = n.into();

        if n.is_nan() {
            unimplemented!("TODO: Implement 'unknown' values")
        }

        let position = self.thresholds.binary_search_by(|probe| {
            probe.partial_cmp(&n).expect("NaN is not allowed here")
        });
        let position: usize = match position {
            Ok(n) => n + 1,
            Err(n) => n,
        };

        &self.range[position]
    }
}

impl<'a, T> ScaleQuantile<'a, T> {
    pub fn new() -> Self {
        Self {
            domain: vec![],
            range: &[],
            thresholds: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults() {
        let scale = ScaleQuantile::<f64>::new();

        assert!(scale.domain.is_empty());
        assert!(scale.range.is_empty());
        // test.equal(s.unknown(), undefined);
    }

    #[test]
    fn quantile_uses_r7() {
        let scale = ScaleQuantile::<i32>::new()
            .domain(&[3, 6, 7, 8, 8, 10, 13, 15, 16, 20])
            .range(&[0, 1, 2, 3]);

        for n in vec![3_f64, 6.0, 6.9, 7.0, 7.1].into_iter() {
            assert_eq!(0, *scale.scale(n));
        }

        for n in vec![8_f64, 8.9].into_iter() {
            assert_eq!(1, *scale.scale(n));
        }

        for n in vec![9_f64, 9.1, 10.0, 13.0].into_iter() {
            assert_eq!(2, *scale.scale(n));
        }

        for n in vec![14.9, 15.0, 15.1, 16.0, 20_f64].into_iter() {
            assert_eq!(3, *scale.scale(n));
        }

        let scale = scale
            .domain(&[3, 6, 7, 8, 8, 9, 10, 13, 15, 16, 20])
            .range(&[0, 1, 2, 3]);

        for n in vec![3_f64, 6.0, 6.9, 7.0, 7.1].into_iter() {
            assert_eq!(0, *scale.scale(n));
        }

        for n in vec![8_f64, 8.9].into_iter() {
            assert_eq!(1, *scale.scale(n));
        }

        for n in vec![9_f64, 9.1, 10.0, 13.0].into_iter() {
            assert_eq!(2, *scale.scale(n));
        }

        for n in vec![14.9, 15.0, 15.1, 16.0, 20_f64].into_iter() {
            assert_eq!(3, *scale.scale(n));
        }
    }

    // tape("quantile(x) returns undefined if the input value is NaN", function(test)

    #[test]
    fn domain_values_are_sorted() {
        let scale = ScaleQuantile::<i32>::new()
            .domain(&[6, 3, 7, 8, 8, 13, 20, 15, 16, 10]);
        assert_eq!(&[3., 6., 7., 8., 8., 10., 13., 15., 16., 20.], scale.domain.as_slice());
    }

    #[test]
    fn domain_values_can_be_zero() {
        let scale = ScaleQuantile::<i32>::new()
            .domain(&[1, 2, 0, 0]);
        assert_eq!(&[0., 0., 1., 2.], scale.domain.as_slice());
    }

    #[test]
    fn non_numeric_domain_values_are_ignored() {
        let scale = ScaleQuantile::<f64>::new()
            .domain(&[6., 3., std::f64::NAN, 7., 8., 8., 13., 20., 15., 16., 10., std::f64::NAN]);
        assert_eq!(&[3., 6., 7., 8., 8., 10., 13., 15., 16., 20.], scale.domain.as_slice());

    }

    #[test]
    fn quantile_returns_inner_thresholds() {
        let scale = ScaleQuantile::<f64>::new()
            .domain(&[3, 6, 7, 8, 8, 10, 13, 15, 16, 20])
            .range(&[0., 1., 2., 3.]);

        assert_eq!(&[7.25, 9.0, 14.5], scale.thresholds.as_slice());

        let scale = scale
            .domain(&[3, 6, 7, 8, 8, 9, 10, 13, 15, 16, 20])
            .range(&[0., 1., 2., 3.]);

        assert_eq!(&[7.5, 9.0, 14.0], scale.thresholds.as_slice());
    }

    #[test]
    fn range_cardinality_determines_the_number_of_quantiles() {
        let scale = ScaleQuantile::<f64>::new()
            .domain(&[3, 6, 7, 8, 8, 10, 13, 15, 16, 20]);

        {
            let scale = scale.clone().range(&[0., 1., 2., 3.]);
            assert_eq!(&[7.25, 9.0, 14.5], scale.thresholds.as_slice());
        }

        {
            let scale = scale.clone().range(&[0., 1.]);
            assert_eq!(&[9.0], scale.thresholds.as_slice());
        }

        {
            use std::f64::NAN;
            let scale = scale.clone().range(&[NAN, NAN, NAN, NAN, NAN]);
            assert_eq!(&[6.8, 8.0, 11.2, 15.2], scale.thresholds.as_slice());
        }

        {
            use std::f64::NAN;
            let scale = scale.clone().range(&[NAN, NAN, NAN, NAN, NAN, NAN]);
            assert_eq!(&[6.5, 8.0, 9.0, 13.0, 15.5], scale.thresholds.as_slice());
        }
    }
}
