use table::cell::Cell;

pub struct Row<'data> {
    pub cells: Vec<Cell<'data>>,
}

impl<'data> Row<'data> {
    pub fn new<T>(cells: Vec<T>) -> Row<'data>
        where T: Into<Cell<'data>>
    {
        let mut row = Row { cells: vec![] };

        for entry in cells {
            row.cells.push(entry.into());
        }

        return row;
    }

    pub fn column_widths(&self) -> Vec<usize> {
        let mut widths = Vec::new();
        for column in &self.cells {
            widths.push(column.width());
        }
        return widths;
    }

    pub fn get_separator(&self, max_widths: &Vec<usize>) -> String {
        let mut buf = String::new();
        let mut span_count = 0;
        buf.push('+');
        for i in 0..max_widths.len() {
            if buf.len() > 1 && span_count == self.cells[i - span_count].col_span{
                buf.push('+');
            }else if span_count > 0{
                buf.push('-');
            }
            buf.push_str(str::repeat("-", max_widths[i]).as_str());
            span_count += 1;
        }
        buf.push('+');
        return buf;
    }
}
