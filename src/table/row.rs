use table::cell::Cell;
use table::{TableStyle, RowPosition};

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

    pub fn get_separator(&self, max_widths: &Vec<usize>, style: &TableStyle, row_position:RowPosition) -> String {
        let mut buf = String::new();
        let mut span_count = 0;
        if row_position == RowPosition::First{
            buf.push(style.top_left_corner);
        }else if row_position == RowPosition::Last{
            buf.push(style.bottom_left_corner);
        }else {
            buf.push(style.outer_left_vertical);
        }
        for i in 0..max_widths.len() {
            if buf.len() > 1 && span_count == self.cells[i - span_count].col_span {
                if row_position == RowPosition::First{
                    buf.push(style.outer_top_horizontal);
                }else if row_position == RowPosition::Last{
                    buf.push(style.outer_bottom_horizontal);
                }else {
                    buf.push(style.intersection);
                }
            } else if span_count > 0 {
                buf.push(style.horizontal);
            }
            buf.push_str(str::repeat(style.horizontal.to_string().as_str(), max_widths[i])
                .as_str());
            span_count += 1;
        }
        if row_position == RowPosition::First{
            buf.push(style.top_right_corner);
        }else if row_position == RowPosition::Last{
            buf.push(style.bottom_right_corner);
        }else {
            buf.push(style.outer_right_vertical);
        }

        return buf;
    }
}
