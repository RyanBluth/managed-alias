pub mod row;
pub mod cell;

use table::row::Row;

use std::cmp::max;

#[derive(Eq, PartialEq, Copy, Clone)]
pub enum RowPosition {
    First,
    Mid,
    Last,
}

pub struct TableStyle {
    pub top_left_corner: char,
    pub top_right_corner: char,
    pub bottom_left_corner: char,
    pub bottom_right_corner: char,
    pub outer_left_vertical: char,
    pub outer_right_vertical: char,
    pub outer_bottom_horizontal: char,
    pub outer_top_horizontal: char,
    pub intersection: char,
    pub vertical: char,
    pub horizontal: char,
}

impl TableStyle {
    pub fn simple() -> TableStyle {
        return TableStyle {
                   top_left_corner: '+',
                   top_right_corner: '+',
                   bottom_left_corner: '+',
                   bottom_right_corner: '+',
                   outer_left_vertical: '+',
                   outer_right_vertical: '+',
                   outer_bottom_horizontal: '+',
                   outer_top_horizontal: '+',
                   intersection: '+',
                   vertical: '|',
                   horizontal: '-',
               };
    }

    pub fn extended() -> TableStyle {
        return TableStyle {
                   top_left_corner: '╔',
                   top_right_corner: '╗',
                   bottom_left_corner: '╚',
                   bottom_right_corner: '╝',
                   outer_left_vertical: '╠',
                   outer_right_vertical: '╣',
                   outer_bottom_horizontal: '╩',
                   outer_top_horizontal: '╦',
                   intersection: '╬',
                   vertical: '║',
                   horizontal: '═',
               };
    }

    fn start_for_position(&self, pos: RowPosition) -> char {
        match pos {
            RowPosition::First => self.top_left_corner,
            RowPosition::Mid => self.outer_left_vertical,
            RowPosition::Last => self.bottom_left_corner,
        }
    }

    fn end_for_position(&self, pos: RowPosition) -> char {
        match pos {
            RowPosition::First => self.top_right_corner,
            RowPosition::Mid => self.outer_right_vertical,
            RowPosition::Last => self.bottom_right_corner,
        }
    }

    fn intersect_for_position(&self, pos: RowPosition) -> char {
        match pos {
            RowPosition::First => self.outer_top_horizontal,
            RowPosition::Mid => self.intersection,
            RowPosition::Last => self.outer_bottom_horizontal,
        }
    }

    fn merge_intersection_for_position(&self,
                                           top: char,
                                           bottom: char,
                                           pos: RowPosition)
                                           -> char {
        if (top == self.horizontal || top == self.outer_bottom_horizontal) &&
           bottom == self.intersection {
            return self.outer_top_horizontal;
        } else if(top == self.intersection || top == self.outer_top_horizontal) &&
                  bottom == self.horizontal {
            return self.outer_bottom_horizontal;
        } else {
            return self.intersect_for_position(pos);
        }
    }
}


pub struct Table<'data> {
    pub column_titles: Vec<String>,
    pub rows: Vec<Row<'data>>,
    pub style: TableStyle,
}

impl<'data> Table<'data> {
    pub fn new() -> Table<'data> {
        return Table {
                   column_titles: Vec::new(),
                   rows: Vec::new(),
                   style: TableStyle::extended(),
               };
    }

    pub fn add_row(&mut self, row: Row<'data>) {
        self.rows.push(row);
    }

    pub fn format_row(&self, row: &Row<'data>, max_widths: &Vec<usize>) -> String {
        let mut buf = String::new();
        let mut span_count = 1;
        let mut col_idx = 0;
        for width in max_widths.into_iter() {
            if row.cells.len() > col_idx {
                if span_count == 1 {
                    let mut pad_len = 0;
                    if *width > row.cells[col_idx].width() {
                        pad_len = width - row.cells[col_idx].width();
                    }

                    buf.push_str(format!("{}{}{}",
                                         self.style.vertical,
                                         row.cells[col_idx],
                                         str::repeat(" ", pad_len))
                                         .as_str());
                } else {
                    buf.push_str(format!("{} ", str::repeat(" ", *width)).as_str());
                }
                if span_count < row.cells[col_idx].col_span {
                    span_count += 1;
                } else {
                    span_count = 1;
                    col_idx += 1;
                }
            } else {
                buf.push_str(format!("{}{}", self.style.vertical, str::repeat(" ", *width))
                                 .as_str());
            }
        }
        buf.push(self.style.vertical);
        return buf;
    }

    pub fn print(&mut self) {
        let mut print_buffer = String::new();
        let max_widths = self.calculate_max_column_widths();
        let mut previous_separator = None;
        if self.rows.len() > 0 {
            for i in 0..self.rows.len() {
                let mut row_pos = RowPosition::Mid;
                if i == 0 {
                    row_pos = RowPosition::First;
                }
                let separator =
                    self.rows[i].get_separator(&max_widths,
                                               &self.style,
                                               row_pos,
                                               previous_separator.clone());
                Table::buffer_line(&mut print_buffer, &separator);
                Table::buffer_line(&mut print_buffer,
                                   &self.format_row(&self.rows[i], &max_widths));
                previous_separator = Some(separator.clone());
            }
            let separator = self.rows
                .last()
                .unwrap()
                .get_separator(&max_widths,
                               &self.style,
                               RowPosition::Last,
                               None);
            Table::buffer_line(&mut print_buffer, &separator);
            println!("{}", print_buffer);
        }
    }

    pub fn calculate_max_column_widths(&self) -> Vec<usize> {
        let mut max_widths: Vec<usize> = Vec::new();
        for row in &self.rows {
            for i in 0..row.cells.len() {
                if max_widths.len() <= i {
                    max_widths.push(row.cells[i].width());
                } else {
                    max_widths[i] = max(max_widths[i], row.cells[i].width());
                }
            }
        }
        return max_widths;
    }

    pub fn buffer_line(buffer: &mut String, line: &String) {
        buffer.push_str(format!("{}\n", line).as_str());
    }
}
