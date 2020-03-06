// use std::cmp;
// use std::marker::PhantomData;
// use std::fmt::Debug;

// use crate::interpolate::Interpolate;

// pub struct Piecewise<Interpolator, Output> where Interpolator: Interpolate<Output> {
//   interpolators: Vec<Interpolator>,
//   phantom: PhantomData<Output>,
// }

// impl<Interpolator, Output> Piecewise<Interpolator, Output>
//   where
//     Self: Sized,
//     Interpolator: Interpolate<Output>,
//     Output: Copy + Debug {
//   pub fn piecewise(p: Vec<Output>) -> Self {
//     let mut iter = p.iter().peekable();

//     let mut interpolators : Vec<Interpolator> = vec![];

//     while let Some(val) = iter.next() {
//       if let Some(&next) = iter.peek() {
//         interpolators.push(Interpolator::bounded(*val, *next));
//       }
//     }

//     Self {
//       interpolators,
//       phantom: PhantomData,
//     }
//   }

//   pub fn interpolate(&self, t: f64) -> Output {
//     let n = self.interpolators.len();
//     let t = t * n as f64;
//     let i = cmp::max(0, cmp::min(n - 1, t.floor() as usize));
//     self.interpolators[i].interpolate(t - i as f64)
//   }
// }

// #[test]
// fn hsl() {
//   use crate::color::*;
//   use crate::interpolate::*;

//   let p : Piecewise<InterpolateContainer<Hsl>, Hsl> = Piecewise::piecewise(vec![
//     Hsl { hue: 0.0, saturation: 1.0, lightness: 0.5 }, // Red
//     Hsl { hue: 120.0, saturation: 1.0, lightness: 0.25098039215686274 }, // Green
//     Hsl { hue: 240.0, saturation: 1.0, lightness: 0.5 }, // Blue
//   ]);
//   assert_eq!(p.interpolate(0.0).to_rgb(), Rgb { red: 255, green: 0, blue: 0 });
//   assert_eq!(p.interpolate(0.3).to_rgb(), Rgb { red: 143, green: 179, blue: 0 });
//   assert_eq!(p.interpolate(0.5).to_rgb(), Rgb { red: 0, green: 128, blue: 0 });
// }
