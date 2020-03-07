pub trait Extent<T> {
    fn extents(&self) -> std::ops::Range<T>;
}

impl<T> Extent<T> for Vec<T>
where
    T: PartialOrd + Clone,
{
    fn extents(&self) -> std::ops::Range<T> {
        // This his hilariously inefficient
        let v_max: T = self
            .iter()
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .expect("max_by failed")
            .to_owned();
        let v_min: T = self
            .iter()
            .min_by(|a, b| a.partial_cmp(b).unwrap())
            .expect("min_by failed")
            .to_owned();
        return v_min..v_max;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name() {
        assert_eq!(vec![1.0, 3.0, 2.0].extents(), 1.0..3.0);
    }
}
