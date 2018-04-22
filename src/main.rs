extern crate clap;
extern crate term_table;

use clap::{App, AppSettings, Arg, SubCommand};

use std::process;
use std::fmt::Display;
use std::fs::{metadata, File, OpenOptions};
use std::io::prelude::*;
use std::process::{Command, Stdio};
use std::collections::HashMap;
use term_table::row::Row;
use term_table::Table;
use term_table::cell::Cell;

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
where
    T: Display,
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
                Ok(metadata) => {
                    if metadata.is_dir() {
                        go(key);
                    } else {
                        run(key, matches.values_of_lossy(RUN_ARGS));
                    }
                }
                Err(_) => run(key, matches.values_of_lossy(RUN_ARGS)),
            }
        } else {
            exit_with_message(format!("Invalid key {}", key))
        }
    }
}

fn list() {
    let entries = get_entries();

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

    table.add_row(Row::new(vec![Cell::new("COMMANDS", 2)]));

    for command in commands {
        table.add_row(Row::new(vec![command.0, command.1]));
    }

    println!("{}", table.as_string());

    println!(" ");

    table = Table::new();

    table.add_row(Row::new(vec![Cell::new("PATHS", 2)]));

    for path in paths {
        table.add_row(Row::new(vec![path.0, path.1]));
    }

    println!("{}", table.as_string());
}

fn run(key: &str, args: Option<Vec<String>>) {
    match lookup(key) {
        Some(value) => for command in value.split("&") {
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
                .spawn()
            {
                Ok(mut child) => {
                    if let Err(e) = child.wait() {
                        exit_with_message(format!(
                            "Failed to wait for command {}. Error: {}",
                            command, e
                        ));
                    }
                }
                Err(e) => exit_with_message(format!("Failed to execute {}. Error: {}", command, e)),
            };
        },
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
        .open(get_file_dir())
    {
        Ok(file) => file,
        Err(e) => {
            exit_with_message(format!(
                "Failed to create file {}. Error: {}",
                STORE_FILE, e
            ));
            return;
        }
    };
    if let Err(e) = file.write_all(out.as_bytes()) {
        exit_with_message(format!(
            "Failed to write value to {}. Error: {}",
            STORE_FILE, e
        ));
    };
}

fn format_entry(key: &String, val: &String) -> String {
    return format!("{}\":\"{}\n", key, val);
}

fn exit_invalid_key(key: &str) {
    exit_with_message(format!("No value was found for key '{}'", key));
}

fn exit_with_message<T>(message: T)
where
    T: Display,
{
    println!("{}", message);
    process::exit(1);
}
