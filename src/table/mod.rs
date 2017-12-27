pub mod row;
pub mod cell;

use table::cell::ColumnEntry;
use table::row::Row;

use std::cmp::max;

pub struct Table<'data> {
    pub column_titles: Vec<String>,
    pub rows: Vec<Row<'data>>,
}

impl<'data> Table<'data> {
    pub fn new() -> Table<'data> {
        return Table {
            column_titles: Vec::new(),
            rows: Vec::new(),
        };
    }

    pub fn add_row(&mut self, row: Row<'data>) {
        self.rows.push(row);
    }

    pub fn print(&mut self) {
        let mut print_buffer = String::new();
        let max_widths = self.calculate_max_column_widths();
        let total_width = max_widths.iter().sum::<usize>() + 4;
        let separator = Table::gen_separator(&max_widths);
        Table::buffer_line(&mut print_buffer, &separator);
        for row in &self.rows {
            Table::buffer_line(&mut print_buffer, &self.format_row(&row, &max_widths));
            Table::buffer_line(&mut print_buffer, &separator);
        }
        //Table::buffer_line(&mut print_buffer, &separator);
        println!("{}", print_buffer);
    }

    pub fn format_row(&self, row: &Row<'data>, max_widths: &Vec<usize>) -> String {
        let mut buf = String::new();
        let mut span_count = 1;
        let mut col_idx = 0;
        for en in max_widths.into_iter().enumerate() {
            if row.columns.len() > col_idx {
                if span_count == 1 {
                    let mut pad_len = 0;
                    if *en.1 > row.columns[col_idx].width(){
                        pad_len = en.1 - row.columns[col_idx].width();
                    }
                    if 0 == 1 {
                        let pad_front_len = f32::ceil(pad_len as f32 / 2f32) as usize;
                        let pad_front = str::repeat(" ", pad_front_len);
                        let pad_end_len = pad_len - pad_front_len;
                        let pad_end = str::repeat(" ", pad_end_len);
                        buf.push_str(format!("|{}{}{}", pad_front, row.columns[col_idx], pad_end)
                            .as_str());
                    } else {
                        buf.push_str(format!("|{}{}", row.columns[col_idx], str::repeat(" ", pad_len))
                            .as_str());
                    }
                }else{
                    buf.push_str(format!("{} ", str::repeat(" ", *en.1)).as_str());
                }
                if span_count < row.columns[col_idx].col_span {
                    span_count += 1;
                }else{
                    span_count = 1;
                    col_idx += 1;
                }
            } else {
                buf.push_str(format!("| {}", str::repeat(" ", *en.1 - 1)).as_str());
            }
        }
        buf.push_str("|");
        return buf;
    }

    pub fn gen_separator(max_widths: &Vec<usize>) -> String {
        let mut buf = String::new();
        buf.push('+');
        for width in max_widths {
            if buf.len() > 1 {
                buf.push('+');
            }
            buf.push_str(str::repeat("-", *width).as_str());
        }
        buf.push('+');
        return buf;
    }

    pub fn calculate_max_column_widths(&self) -> Vec<usize> {
        let mut max_widths: Vec<usize> = Vec::new();
        for row in &self.rows {
            for i in 0..row.columns.len() {
                if max_widths.len() <= i {
                    max_widths.push(row.columns[i].width());
                } else {
                    max_widths[i] = max(max_widths[i], row.columns[i].width());
                }
            }
        }
        return max_widths;
    }

    pub fn buffer_line(buffer: &mut String, line: &String) {
        buffer.push_str(format!("{}\n", line).as_str());
    }
}
