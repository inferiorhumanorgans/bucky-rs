pub trait Quantileable<DataType> {
    fn quantile(&self, p: f64) -> f64;
    fn quantile_by_key<AccessorType>(&self, p: f64, accessor: AccessorType) -> f64
    where AccessorType: FnMut(&DataType) -> f64;
}

pub struct Quantile {

}

impl Quantile {
    pub fn quantile<I>(values: I, p: f64) -> f64 where I: Iterator<Item = f64>{
        let mut values : Vec<f64> = values.collect();

        assert!(values.len() > 0);

        if p <= 0.0 || values.len() < 2 {
            return values
                .iter()
                .copied()
                .min_by(|x, y| x.partial_cmp(y).expect("You promised no naan today"))
                .expect("Couldn't find minimum value")
        }

        if p >= 1.0 {
            return values
                .iter()
                .copied()
                .max_by(|x, y| x.partial_cmp(y).expect("You promised no naan today"))
                .expect("Couldn't find maximum value")
        }

        let i = (values.len() as f64 - 1.0) * p;
        let i0 = i.floor() as usize;
        order_stat::kth_by(&mut values, i0, |x, y| x.partial_cmp(y).expect("Float comparison failed"));

        let value0 = values[0..i0+1]
            .iter()
            .copied()
            .max_by(|x, y| x.partial_cmp(y).expect("You promised no naan today"))
            .expect("Couldn't find maximum value");

        let value1 = values[(i0+1)..]
            .iter()
            .copied()
            .min_by(|x, y| x.partial_cmp(y).expect("You promised no naan today"))
            .expect("Couldn't find minimum value");

        value0 + (value1 - value0) * (i - i0 as f64)
    }
}

impl<DataType> Quantileable<DataType> for Vec<DataType>
where DataType: Into<f64> + Copy {
    fn quantile(&self, p: f64) -> f64 {
        self.quantile_by_key(p, |d| (*d).into())
    }

    fn quantile_by_key<AccessorType>(&self, p: f64, accessor: AccessorType) -> f64
    where AccessorType: FnMut(&DataType) -> f64
    {
        Quantile::quantile(self.iter().map(accessor), p)
    }
   
}

#[test]
fn quantile() {
    let data = vec![0, 10, 30];

    assert_eq!(data.quantile(0.0), 0.0);
    assert_eq!(data.quantile(0.5), 10.0);
    assert_eq!(data.quantile(1.0), 30.0);
    assert_eq!(data.quantile(0.25), 5.0);
    assert_eq!(data.quantile(0.75), 20.0);
    assert_eq!(data.quantile(0.1), 2.0);
}

#[test]
fn quantile_by_key() {
    {
        let data = vec![0_i32, 10, 30];

        // If types can be inferred this works:
        assert_eq!(data.quantile_by_key(0.0, |d| (*d).into()), 0.0);

        // Otherwise we need to be explicit:
        let accessor = |d : &i32| (*d).into();
        assert_eq!(data.quantile_by_key(0.5, accessor), 10.0);
        assert_eq!(data.quantile_by_key(1.0, accessor), 30.0);
        assert_eq!(data.quantile_by_key(0.25, accessor), 5.0);
        assert_eq!(data.quantile_by_key(0.75, accessor), 20.0);
        assert_eq!(data.quantile_by_key(0.1, accessor), 2.0);
    }
}
