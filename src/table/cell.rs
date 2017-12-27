use std::borrow::Cow;
use std::fmt::{Display, Result, Formatter};

pub struct ColumnEntry<'data> {
    pub data: Cow<'data, str>,
    pub col_span: usize,
}

impl<'data> ColumnEntry<'data> {
    pub fn new<C>(data: C, col_span: usize) -> ColumnEntry<'data>
        where C: Into<Cow<'data, str>>
    {
        return ColumnEntry {
            data: data.into(),
            col_span,
        };
    }

    pub fn width(&self) -> usize {
        return format!("{}", self).len();
    }
}

impl<'data, T> From<&'data T> for ColumnEntry<'data>
    where T: Display
{
    fn from(x: &'data T) -> ColumnEntry<'data> {
        return ColumnEntry::new(format!("{}", x), 1);
    }
}

impl<'data> Display for ColumnEntry<'data> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, " {} ", self.data)
    }
}
