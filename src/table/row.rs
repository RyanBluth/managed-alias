use table::cell::ColumnEntry;

pub struct Row<'data> {
    pub columns: Vec<ColumnEntry<'data>>,
}

impl<'data> Row<'data> {
    pub fn new<T>(cells: Vec<T>) -> Row<'data>
        where T: Into<ColumnEntry<'data>>
    {
        let mut row = Row { columns: vec![] };

        for entry in cells {
            row.columns.push(entry.into());
        }

        return row;
    }

    pub fn column_widths(&self) -> Vec<usize> {
        let mut widths = Vec::new();
        for column in &self.columns {
            widths.push(column.width());
        }
        return widths;
    }
}
