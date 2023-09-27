use std::{
    error::Error,
    fmt::{Debug, Display},
};

#[derive(Debug, derive_new::new)]
pub struct LinedError<T>
where
    T: Debug + Display,
{
    pub line: usize,
    pub lines_amount: usize,
    pub related_text: String,
    pub content: T,
}

impl<T> Error for LinedError<T> where T: Debug + Display {}
impl<T> std::fmt::Display for LinedError<T>
where
    T: Debug + Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let line_normal = self.lines_amount - (self.line - 1);
        let line_from_bottom = self.line;
        let content = &self.content;
        let related_text = &self.related_text;
        return write!(
            f,
            "Error on line:{line_normal} (from bottom:{line_from_bottom})\n\"{related_text}\"\n{content}"
        );
    }
}
pub fn build_step<T, TN, F>(a: &[T], f: F) -> Vec<TN>
where
    F: Fn(&[T]) -> Option<(&[T], TN)>,
{
    let mut a = a;
    let mut vec = Vec::new();
    loop {
        let result = f(a);
        match result {
            None => break,
            Some(v) => {
                a = v.0;
                vec.push(v.1);
            }
        }
    }
    return vec;
}

pub trait IteratorExtension<I, Item, E>
where
    I: Iterator<Item = Item>,
    Item: PartialEq<E>,
{
    fn find_i(self, element: E) -> Option<usize>;
}

impl<I, Item, E> IteratorExtension<I, Item, E> for I
where
    I: Iterator<Item = Item>,
    Item: PartialEq<E>,
{
    fn find_i(mut self, element: E) -> Option<usize> {
        return self.position(|e| e == element);
    }
}
trait FlagExtension<T> {
    fn use_if(self, b: bool) -> T;
}
impl<T> FlagExtension<T> for T
where
    T: bitflags::Flags,
{
    fn use_if(self, b: bool) -> Self {
        if b {
            return self;
        }
        return Self::empty();
    }
}


