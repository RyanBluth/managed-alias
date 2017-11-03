extern crate clap;

use clap::{App, AppSettings, Arg, SubCommand};

use std::process;
use std::fmt::Display;
use std::fs::{File, OpenOptions, metadata};
use std::io::prelude::*;
use std::process::Command;

const GO: &'static str = "go";
const SET: &'static str = "set";
const KEY: &'static str = "key";
const VALUE: &'static str = "value";
const LIST: &'static str = "list";
const RUN: &'static str = "run";
const RUN_ARGS: &'static str = "run_args";
const DELETE: &'static str = "delete";
const PROPS_FILE: &'static str = ".managed-alias-store";

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
            None => exit_with_message("Delete requires a variable key")
        }
    } else if let Some(key) = matches.value_of(KEY) {
        let value = lookup(key);

        if let Some(value) = value {
            let metadata = metadata(value);
            match metadata {
                Ok(_) => go(key),
                Err(_) => run(key, matches.values_of_lossy(RUN_ARGS))
            }
        } else {
            exit_with_message(format!("Invalid key {}", key))
        }
    }
}

fn list() {
    let file_contents = get_file_contents();
    let mut keys = vec![];
    let mut values = vec![];
    let mut longest_key = 0;
    for line in file_contents.split('\n') {
        let mut split = line.split("\":\"");
        let key_opt = split.next();
        let val_opt = split.last();
        if val_opt.is_some() && key_opt.is_some() {
            let key = key_opt.unwrap();
            let val = val_opt.unwrap();
            if key.len() > longest_key {
                longest_key = key.len();
            }
            keys.push(key);
            values.push(val);
        }
    }
    for pair in keys.iter().zip(values) {
        let key = pair.0;
        let val = pair.1;
        let padding = str::repeat(" ", longest_key - key.len());
        println!("{}{} = {}", padding, key, val);
    }
}

fn run(key: &str, args: Option<Vec<String>>) {
    match lookup(key) {
        Some(value) => {
            let mut sub = value.clone();
            if let Some(arg_vec) = args {
                for i in 0..arg_vec.len() {
                    let token = format!("${}", i);
                    sub = sub.replace(token.as_str(), arg_vec[i].as_str());
                }
            }
            let mut value_it = sub.split_whitespace();
            if let Err(e) = Command::new(value_it.next().unwrap()).args(value_it).spawn() {
                exit_with_message(format!("Failed to execute {}. Error: {}", value, e));
            };
        }
        None => invalid_key(key)
    }
}

fn go(key: &str) {
    match lookup(key) {
        Some(value) => println!("*{}", value),
        None => invalid_key(key),
    }
}

fn set(key: &str, mut values: clap::Values) {
    let buf = get_file_contents();
    let mut combined = String::from(values.next().unwrap());
    for v in values {
        combined.push_str(" ");
        combined.push_str(v);
    }
    let mut out = String::new();
    let mut overwritten = false;
    for line in buf.lines() {
        if line.starts_with(key) {
            out += &format_key_val(key, combined.as_str());
            overwritten = true;
        } else {
            out += &format!("{}\n", line);
        }
    }
    if !overwritten {
        out += &format_key_val(key, combined.as_str());
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
                exit_with_message(format!("Failed to create file .managedaliasprops. Error: {}", e));
                return;
            }
        };
    if let Err(e) = file.write_all(out.as_bytes()) {
        exit_with_message(format!(
            "Failed to write value to .managedaliasprops. Error: {}",
            e
        ));
    };
}

fn delete(key: &str) {
    let file_contents = get_file_contents();
}

fn lookup(key: &str) -> Option<String> {
    let key_value_pairs = get_key_value_pairs();
    for pair in key_value_pairs {
        if pair.0 == key {
            return Some(pair.1);
        }
    }
    None
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
    exe_path.push(PROPS_FILE);
    let path = String::from(exe_path.to_path_buf().to_string_lossy());
    return path;
}

fn get_key_value_pairs() -> Vec<(String, String)> {
    let mut result = vec![];
    let contents = get_file_contents();
    let lines = contents.split('\n');
    for line in lines {
        let mut key_val = line.split("\":\"");
        let key = key_val.next();
        let val = key_val.next();
        if key.is_some() && val.is_some() {
            result.push((String::from(key.unwrap()), String::from(val.unwrap())));
        }
    }
    return result;
}

fn format_key_val<'a>(key: &str, val: &str) -> String {
    format!("{}\":\"{}\n", key, val)
}

fn invalid_key(key: &str) {
    exit_with_message(format!("No value was found for key '{}'", key));
}


fn exit_with_message<T>(message: T)
    where
        T: Display,
{
    println!("{}", message);
    process::exit(1);
}