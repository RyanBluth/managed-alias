use std::borrow::Cow;
use std::fmt::{Display, Result, Formatter};

pub struct Cell<'data> {
    pub data: Cow<'data, str>,
    pub col_span: usize,
}

impl<'data> Cell<'data> {
    pub fn new<C>(data: C, col_span: usize) -> Cell<'data>
        where C: Into<Cow<'data, str>>
    {
        return Cell {
                   data: data.into(),
                   col_span,
               };
    }

    pub fn width(&self) -> usize {
        return format!("{}", self).len();
    }
}

impl<'data, T> From<&'data T> for Cell<'data>
    where T: Display
{
    fn from(x: &'data T) -> Cell<'data> {
        return Cell::new(format!("{}", x), 1);
    }
}

impl<'data> Display for Cell<'data> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, " {} ", self.data)
    }
}
