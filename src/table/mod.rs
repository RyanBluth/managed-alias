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
        } else if top == self.outer_bottom_horizontal && bottom == self.horizontal{
            return self.horizontal;
        }else {
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


    fn format_row(&self, row: &Row<'data>, max_widths: &Vec<usize>) -> String {
        let mut buf = String::new();

        // Number of columns spanned since the last cell
        let mut columns_spanned = 0;

        // The index of the current column. The length of max_widths may be larger
        // than the length of the row's cells if any of the cells have a col_span value > 1
        let mut current_column = 0;

        for width in max_widths.iter() {

            // This row may not have as many cells as there are columns
            if row.cells.len() > current_column {

                // We print the cell value when a new cell begins
                if columns_spanned == 0 {

                    let mut pad_len = 0;

                    // Pad the cell if the length of the text is less than the max column width
                    if *width > row.cells[current_column].width() {
                        pad_len = width - row.cells[current_column].width();
                    }

                    buf.push_str(format!("{}{}{}",
                                         self.style.vertical,
                                         row.cells[current_column],
                                         str::repeat(" ", pad_len))
                                         .as_str());
                } else {
                    // If the cell spans multiple columns, just fill the remaining space with whitespace
                    buf.push_str(format!("{} ", str::repeat(" ", *width)).as_str());
                }

                columns_spanned += 1;

                // Check to see if we have spanned the cell yet
                if columns_spanned == row.cells[current_column].col_span{
                    columns_spanned = 0;
                    current_column += 1;
                }
            } else {
                // This is just prints a blank cell since we don't have a value
                buf.push_str(format!("{}{}", self.style.vertical, str::repeat(" ", *width))
                                 .as_str());
            }
        }
        buf.push(self.style.vertical);
        return buf;
    }

    fn calculate_max_column_widths(&self) -> Vec<usize> {
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

    fn buffer_line(buffer: &mut String, line: &String) {
        buffer.push_str(format!("{}\n", line).as_str());
    }
}
