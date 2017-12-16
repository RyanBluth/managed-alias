extern crate clap;

use clap::{App, AppSettings, Arg, SubCommand};

use std::process;
use std::fmt::{Display, Result, Formatter};
use std::fs::{File, OpenOptions, metadata};
use std::io::prelude::*;
use std::process::{Command, Stdio};
use std::collections::HashMap;
use std::cmp::max;
use std::borrow::Cow;

const GO: &'static str = "go";
const SET: &'static str = "set";
const KEY: &'static str = "key";
const VALUE: &'static str = "value";
const LIST: &'static str = "list";
const RUN: &'static str = "run";
const RUN_ARGS: &'static str = "run_args";
const DELETE: &'static str = "delete";
const STORE_FILE: &'static str = ".managed-alias-store";

#[derive(Debug)]
struct GenericError {
    description: String,
}

impl GenericError {
    pub fn new(description: String) -> GenericError {
        return GenericError { description };
    }
}

impl<T> From<T> for GenericError
    where T: Display
{
    fn from(x: T) -> Self {
        return GenericError::new(format!("{}", x));
    }
}


fn main() {
    let matches = App::new("managed-alias")
        .version("1.0")
        .author("Ryan Bluth")
        .setting(AppSettings::ArgsNegateSubcommands)
        .arg(
            Arg::with_name(KEY)
                .help("Variable key")
                .required(false)
                .index(1)
        )
        .arg(
            Arg::with_name(RUN_ARGS)
                .help("Arguments to pass to the command stored in the variable matching the provided key")
                .required(false)
                .multiple(true)
        )
        .subcommand(
            SubCommand::with_name(GO)
                .alias("g")
                .about("Navigates to the value of the specified key")
                .arg(
                    Arg::with_name(KEY)
                        .help("Variable key")
                        .required(true)
                        .index(1),
                ),
        )
        .subcommand(
            SubCommand::with_name(LIST)
                .alias("l")
                .about("Lists all variables"),
        )
        .subcommand(
            SubCommand::with_name(RUN)
                .alias("r")
                .about("Execute the matching value for the provided key")
                .arg(
                    Arg::with_name(KEY)
                        .help("Variable key")
                        .required(true)
                        .index(1), )
                .arg(
                    Arg::with_name(RUN_ARGS)
                        .help("Arguments to pass to the command stored in the variable matching the provided key")
                        .required(false)
                        .multiple(true)
                )
        )
        .subcommand(
            SubCommand::with_name(SET)
                .alias("s")
                .about("Sets the specified key to the specified value")
                .setting(AppSettings::AllowLeadingHyphen)
                .arg(Arg::with_name(KEY).help("Variable key").required(true))
                .arg(
                    Arg::with_name(VALUE)
                        .help("Variable value")
                        .required(true)
                        .multiple(true)
                        .allow_hyphen_values(true),
                ),
        )
        .subcommand(SubCommand::with_name(DELETE)
            .alias("d")
            .about("Delete a key value pair")
            .arg(Arg::with_name(KEY).help("Variable key").required(true))
        )
        .get_matches();

    if let Some(sub_matches) = matches.subcommand_matches(GO) {
        match sub_matches.value_of(KEY) {
            Some(key) => go(key),
            None => exit_with_message("go requires a variable key"),
        }
    } else if let Some(sub_matches) = matches.subcommand_matches(SET) {
        if sub_matches.is_present(KEY) && sub_matches.is_present(VALUE) {
            let key = sub_matches.value_of(KEY).unwrap();
            let values = sub_matches.values_of(VALUE).unwrap();
            set(key, values);
        } else {
            exit_with_message("A key and value must be provided")
        }
    } else if matches.is_present(LIST) {
        list();
    } else if let Some(sub_matches) = matches.subcommand_matches(RUN) {
        match sub_matches.value_of(KEY) {
            Some(key) => run(key, sub_matches.values_of_lossy(RUN_ARGS)),
            None => exit_with_message("go requires a variable key"),
        }
    } else if let Some(sub_matches) = matches.subcommand_matches(DELETE) {
        match sub_matches.value_of(KEY) {
            Some(key) => delete(key),
            None => exit_with_message("Delete requires a variable key"),
        }
    } else if let Some(key) = matches.value_of(KEY) {
        let value = lookup(key);

        if let Some(value) = value {
            let metadata = metadata(value);
            match metadata {
                Ok(_) => go(key),
                Err(_) => run(key, matches.values_of_lossy(RUN_ARGS)),
            }
        } else {
            exit_with_message(format!("Invalid key {}", key))
        }
    }
}

struct ColumnEntry<'data> {
    data: Cow<'data, str>,
    col_span: usize,
}

impl<'data> ColumnEntry<'data> {
    fn new<C>(data: C, col_span: usize) -> ColumnEntry<'data>
        where C: Into<Cow<'data, str>>
    {
        return ColumnEntry {
                   data: data.into(),
                   col_span,
               };
    }

    fn width(&self) -> usize {
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

/*
impl<'data, T> From<Vec<T>> for ColumnEntry<'data> where T: Display{

    fn from(x: &'data T) -> ColumnEntry<'data>{
        return ColumnEntry::new(format!("{}", x), 1);
    }
}
*/


/*
impl<'data, T> Into<ColumnEntry<'data>> for T where T: Display{

    fn into(self) -> ColumnEntry<'data> {
        return ColumnEntry::new(format!("{}", self), 1);
    }
}
*/
impl<'data> Display for ColumnEntry<'data> {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, " {} ", self.data)
    }
}

struct Row<'data> {
    columns: Vec<ColumnEntry<'data>>,
}

impl<'data> Row<'data> {
    fn new<T>(column_entries: Vec<T>) -> Row<'data>
        where T: Into<ColumnEntry<'data>>
    {
        let mut row = Row { columns: vec![] };

        for entry in column_entries {
            row.columns.push(entry.into());
        }

        return row;
    }

    fn column_widths(&self) -> Vec<usize> {
        let mut widths = Vec::new();
        for column in &self.columns {
            widths.push(column.width());
        }
        return widths;
    }
}

struct Table<'data> {
    column_titles: Vec<String>,
    rows: Vec<Row<'data>>,
}

impl<'data> Table<'data> {
    fn new() -> Table<'data> {
        return Table {
                   column_titles: Vec::new(),
                   rows: Vec::new(),
               };
    }

    fn add_row(&mut self, row: Row<'data>) {
        self.rows.push(row);
    }

    fn print(&mut self) {
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

    fn format_row(&self, row: &Row<'data>, max_widths: &Vec<usize>) -> String {
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

    fn gen_separator(max_widths: &Vec<usize>) -> String {
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

    fn calculate_max_column_widths(&self) -> Vec<usize> {
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

    fn buffer_line(buffer: &mut String, line: &String) {
        buffer.push_str(format!("{}\n", line).as_str());
    }
}

fn list() {
    let entries = get_entries();
    let mut longest_key = 0;
    let mut longest_total = 0;

    let mut commands = Vec::new();
    let mut paths = Vec::new();

    for entry in entries.iter().collect::<Vec<(&String, &String)>>() {
        let metadata = metadata(entry.1);
        match metadata {
            Ok(_) => paths.push(entry),
            Err(_) => commands.push(entry),
        }
    }

    let mut table = Table::new();

    table.add_row(Row::new(vec![ColumnEntry::new("COMMANDS", 2)]));

    for command in commands {
        table.add_row(Row::new(vec![command.0, command.1]));
    }

    table.add_row(Row::new(vec![ColumnEntry::new("PATHS", 2)]));

    for path in paths {
        table.add_row(Row::new(vec![path.0, path.1]));
    }

    table.print();
}

fn print_header(max_len: usize, val: &str) {
    let sep = str::repeat("-", max_len);
    let padding = str::repeat(" ", max_len - val.len() - 4);
    println!("{}", sep);
    println!("| {}{} |", val, padding);
    println!("{}", sep);
}

fn print_key_val(max_len: usize, max_key_len: usize, key: &String, val: &String) {
    let padding_start = str::repeat(" ", max_key_len - key.len());
    let padding_end = str::repeat(" ",
                                  max_len - padding_start.len() - key.len() - val.len() - 7);
    println!("| {}{} = {}{} |", padding_start, key, val, padding_end);
}

fn run(key: &str, args: Option<Vec<String>>) {
    match lookup(key) {
        Some(value) => {
            for command in value.split("&") {
                let mut out_args: Vec<String> = command
                    .split_whitespace()
                    .map(|s| String::from(s))
                    .collect::<Vec<String>>();
                if let Some(arg_vec) = args.clone() {
                    let mut joined_args = String::new();
                    for arg in &arg_vec {
                        joined_args.push_str(arg.clone().as_str());
                        joined_args.push(' ');
                    }
                    joined_args.pop();

                    for arg in out_args.clone().iter().enumerate() {
                        let mut current = arg.1.clone();
                        for i in 0..arg_vec.len() {
                            let token = format!("${}", i);
                            current = current.replace(token.as_str(), arg_vec[i].as_str());
                        }
                        current = current.replace("$*", joined_args.as_str());
                        out_args[arg.0] = current;
                    }
                }
                let mut arg_iter = out_args.iter();
                match Command::new(arg_iter.next().unwrap())
                          .args(arg_iter)
                          .stdout(Stdio::inherit())
                          .spawn() {
                    Ok(mut child) => {
                        if let Err(e) = child.wait() {
                            exit_with_message(format!("Failed to wait for command {}. Error: {}",
                                                      command,
                                                      e));
                        }
                    }
                    Err(e) => {
                        exit_with_message(format!("Failed to execute {}. Error: {}", command, e))
                    }
                };
            }
        }
        None => exit_invalid_key(key),
    }
}

fn go(key: &str) {
    match lookup(key) {
        Some(value) => println!("*{}", value),
        None => exit_invalid_key(key),
    }
}

fn set(key: &str, mut values: clap::Values) {
    let mut entries = get_entries();
    let mut combined = String::from(values.next().unwrap());
    for v in values {
        combined.push_str(" ");
        combined.push_str(v);
    }
    entries.insert(String::from(key), combined);
    write_entries(entries);
}

fn delete(key: &str) {
    let mut entries = get_entries();
    entries.remove(&String::from(key));
    write_entries(entries);
}

fn lookup(key: &str) -> Option<String> {
    let entries = get_entries();
    match entries.get(&String::from(key)) {
        None => None,
        Some(entry) => Some(entry.clone()),
    }
}

fn get_file_contents() -> String {
    let mut file: File = match File::open(get_file_dir()) {
        Ok(file) => file,
        Err(_) => {
            return String::new();
        }
    };
    let mut buf = String::new();
    file.read_to_string(&mut buf).unwrap();
    return buf;
}

fn get_file_dir() -> String {
    let mut exe_path = std::env::current_exe().unwrap();
    exe_path.pop();
    exe_path.push(STORE_FILE);
    let path = String::from(exe_path.to_path_buf().to_string_lossy());
    return path;
}

fn get_entries() -> HashMap<String, String> {
    let mut result: HashMap<String, String> = HashMap::new();
    let contents = get_file_contents();
    let lines = contents.split('\n');
    for line in lines {
        let mut pair = line.split("\":\"");
        let key = pair.next();
        let val = pair.next();
        if key.is_some() && val.is_some() {
            result.insert(String::from(key.unwrap()), String::from(val.unwrap()));
        }
    }
    return result;
}

fn write_entries(entries: HashMap<String, String>) {
    let mut out = String::new();
    for entry in entries {
        out.push_str(format_entry(&entry.0, &entry.1).as_str());
    }
    let mut file = match OpenOptions::new()
              .write(true)
              .create(true)
              .truncate(true)
              .read(true)
              .open(get_file_dir()) {
        Ok(file) => file,
        Err(e) => {
            exit_with_message(format!("Failed to create file {}. Error: {}", STORE_FILE, e));
            return;
        }
    };
    if let Err(e) = file.write_all(out.as_bytes()) {
        exit_with_message(format!("Failed to write value to {}. Error: {}", STORE_FILE, e));
    };
}

fn format_entry(key: &String, val: &String) -> String {
    return format!("{}\":\"{}\n", key, val);
}

fn exit_invalid_key(key: &str) {
    exit_with_message(format!("No value was found for key '{}'", key));
}

fn exit_with_message<T>(message: T)
    where T: Display
{
    println!("{}", message);
    process::exit(1);
}
