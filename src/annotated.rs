#[derive(Clone)]
pub struct Annotated<'a, IteratorType, FuncType>
where
    IteratorType: Iterator,
    FuncType: FnMut(minidom::ElementBuilder, IteratorType::Item) -> minidom::ElementBuilder,
{
    pub iter: IteratorType,
    pub element_type: &'a str,
    pub f: FuncType,
}

impl<'a, I, F> Annotated<'a, I, F>
where
    I: Iterator,
    F: FnMut(minidom::ElementBuilder, I::Item) -> minidom::ElementBuilder,
{
    pub fn new(iter: I, element_type: &'a str, f: F) -> Annotated<'a, I, F> {
        Annotated {
            iter,
            element_type,
            f,
        }
    }
}

impl<'a, I: Iterator, F> std::iter::Iterator for Annotated<'a, I, F>
where
    F: FnMut(minidom::ElementBuilder, I::Item) -> minidom::ElementBuilder,
{
    type Item = minidom::Element;

    fn next(&mut self) -> Option<minidom::Element> {
        self.iter
            .next()
            .map(|d| (self.f)(Self::Item::builder(self.element_type), d).build())
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

pub trait Annotatable<'a>
where
    Self: Iterator + Sized,
{
    fn annotate<FuncType>(self, element_type: &'a str, f: FuncType) -> Annotated<'a, Self, FuncType>
    where
        Self: Sized,
        FuncType: FnMut(minidom::ElementBuilder, Self::Item) -> minidom::ElementBuilder,
    {
        Annotated::new(self, element_type, f)
    }
}

impl<'a, I> Annotatable<'a> for I where I: Iterator {}
