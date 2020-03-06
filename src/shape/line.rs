//! Line generators create path strings.

use std::marker::PhantomData;

use crate::shape::curve::{CurveGenerator, CurveLinear};

/// `Line` is a Cartesian line generator.
pub struct Line<'a, SourceType> {
    pub curve: Box<dyn CurveGenerator>,
    x: Box<dyn FnMut(&SourceType, usize) -> f64 + 'a>,
    y: Box<dyn FnMut(&SourceType, usize) -> f64 + 'a>,
    defined: Box<dyn FnMut(&SourceType, usize) -> bool + 'a>,
    phantom: PhantomData<SourceType>,
}

impl<'a, SourceType> Line<'a, SourceType> {
    /// Constructs a new line generator.
    pub fn new() -> Self {
        Self {
            curve: Box::new(CurveLinear {}),
            x: Box::new(|_, _| unimplemented!("X accessor not implemented")),
            y: Box::new(|_, _| unimplemented!("Y accessor not implemented")),
            defined: Box::new(|_, _| true),
            phantom: PhantomData {},
        }
    }

    /// Sets the curve factory used by the the line generator.
    pub fn curve(self, curve: Box<dyn CurveGenerator>) -> Self {
        Self { curve, ..self }
    }

    // TODO: Create a set of closures that take objects that implement a trait to
    // allow for easy peasy default behavior.

    /// Sets the `x` accessor to the specified closure and returns line generator.
    ///
    /// The `x` accessor is invoked for each defined element in the input data
    /// collection.  The accessor takes two parameters: the element (of type
    /// SourceType) and the index.  The default accessor calls the
    /// `unimplemented!` macro resulting in a panic.
    pub fn x(self, x: Box<dyn FnMut(&SourceType, usize) -> f64 + 'a>) -> Self {
        Self { x, ..self }
    }

    /// Sets the `y` accessor to the specified closure and returns line generator.
    ///
    /// The `y` accessor is invoked for each defined element in the input data
    /// collection.  The accessor takes two parameters: the element (of type
    /// SourceType) and the index.  The default accessor calls the
    /// `unimplemented!` macro resulting in a panic.
    pub fn y(self, y: Box<dyn FnMut(&SourceType, usize) -> f64 + 'a>) -> Self {
        Self { y, ..self }
    }

    /// Sets the `defined` accessor to the specified closure and returns the
    /// line generator.
    ///
    /// The default accessor thus assumes that the input data is always defined.
    /// The defined accessor is invoked for each element in the input data
    /// collection. The accessor takes two arguments: element (of type
    /// SourceType) and the index. If the given element is defined (i.e., if
    /// the defined accessor returns a truthy value for this element), the x
    /// and y accessors will subsequently be evaluated and the point will be
    /// added to the current line segment. Otherwise, the element will be
    /// skipped, the current line segment will be ended, and a new line segment
    /// will be generated for the next defined point. As a result, the generated
    /// line may have several discrete segments
    pub fn defined(self, defined: Box<dyn FnMut(&SourceType, usize) -> bool + 'a>) -> Self {
        Self { defined, ..self }
    }

    pub fn generate(&mut self, data: &[SourceType]) -> String {
        let mut context = self.curve.context();

        context.start_line();

        for (i, datum) in data.iter().enumerate() {
            if (self.defined)(datum, i) {
                context.point((self.x)(datum, i), (self.y)(datum, i));
            }
        }
        context.end_line();
        let path_string = context.path().clone().into_outline();

        format!("{:?}", path_string)
    }
}

pub trait LineSource {
    fn x(&self) -> f64;
    fn y(&self) -> f64;
    fn defined(&self) -> bool;
}

// pub struct CurveBasis {}
// impl Curve for CurveBasis {}

// pub struct CurveBundle {}
// impl Curve for CurveBundle {}

// pub struct CurveCardinal {}
// impl Curve for CurveCardinal {}

// pub struct CurveCatmullRom {}
// impl Curve for CurveCatmullRom {}

// pub struct CurveMonotoneX {}
// impl Curve for CurveMonotoneX {}

// pub struct CurveMonotoneY {}
// impl Curve for CurveMonotoneY {}

#[cfg(test)]
impl LineSource for (f64, f64) {
    fn x(&self) -> f64 {
        self.0
    }

    fn y(&self) -> f64 {
        self.1
    }

    fn defined(&self) -> bool {
        true
    }
}
