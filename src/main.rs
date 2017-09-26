extern crate clap;

use clap::{App, AppSettings, Arg, SubCommand};

use std::process;
use std::fmt::Display;
use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use std::process::Command;

const GO: &'static str = "go";
const SET: &'static str = "set";
const KEY: &'static str = "key";
const VALUE: &'static str = "value";
const LIST: &'static str = "list";
const RUN: &'static str = "run";
const PROPS_FILE: &'static str = ".mangyprops";

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
    let matches = App::new("mangy")
        .version("1.0")
        .author("Ryan Bluth")
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
                        .index(1),
                ),
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
            Some(key) => run(key),
            None => exit_with_message("go requires a variable key"),
        }
    }
}

fn list() {
    let file_contents = get_file_contents();
    for line in file_contents.split('\n') {
        let mut split = line.split("\":\"");
        let key = split.next();
        let val = split.last();
        if val.is_some() && key.is_some() {
            println!("{}={}", key.unwrap(), val.unwrap());
        }
    }
}

fn run(key: &str) {
    match lookup(key) {
        Ok(opt) => match opt {
            Some(value) => {
                let mut value_it = value.split_whitespace();
                if let Err(e) = Command::new(value_it.next().unwrap()).args(value_it).spawn() {
                    exit_with_message(format!("Failed to execute {}. Error: {}", value, e));
                };
            }
            None => invalid_key(key),
        },
        Err(e) => exit_with_message(e.description),
    }
}

fn go(key: &str) {
    match lookup(key) {
        Ok(opt) => match opt {
            Some(value) => println!("{}", value),
            None => invalid_key(key),
        },
        Err(e) => exit_with_message(e.description),
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
            exit_with_message(format!("Failed to create file .mangyprops. Error: {}", e));
            return;
        }
    };
    if let Err(e) = file.write_all(out.as_bytes()) {
        exit_with_message(format!(
            "Failed to write value to .mangyprops. Error: {}",
            e
        ));
    };
}

fn lookup(key: &str) -> Result<Option<String>, GenericError> {
    let mut file: File = match File::open(get_file_dir()) {
        Ok(file) => file,
        Err(e) => {
            return Err(GenericError::new(
                format!("Failed to open file .mangyprops. Error: {}", e),
            ))
        }
    };
    let mut buf = String::new();
    file.read_to_string(&mut buf)?;
    let lines = buf.split('\n');
    for line in lines {
        let mut key_val = line.split("\":\"");
        if let Some(line_key) = key_val.next() {
            if key == line_key {
                let value = key_val.last().unwrap();
                return Ok(Some(String::from(value)));
            }
        }
    }
    Ok(None)
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

fn get_file_dir()->String{
    let mut exe_path = std::env::current_exe().unwrap();
    exe_path.pop();
    exe_path.push(PROPS_FILE);
    let path = String::from(exe_path.to_path_buf().to_string_lossy());
    return path;
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