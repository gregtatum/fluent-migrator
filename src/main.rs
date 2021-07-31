#![allow(unused_imports)]

pub mod fluent;
#[macro_use]
pub mod parser;

use clap::{App, Arg};
use fluent::dtd_to_fluent;
use parser::dtd::dtd;
use parser::properties::properties;
use std::{ffi::OsStr, fs, path::Path};

struct ParsedArgs<'a> {
    files: Vec<&'a str>,
}

enum Extension {
    Dtd,
    Properties,
}

fn main() {
    let matches = App::new("DTD to Fluent Migrator")
        .about("Take Firefox DTD and migrate them to a Fluent syntax")
        .arg(
            Arg::with_name("FILES")
                .index(1)
                .multiple(true)
                .required(true)
                .help("The space separated list dtd paths to migrate"),
        )
        .get_matches();

    let args = ParsedArgs {
        files: matches
            .values_of("FILES")
            .expect("At least one dtd file must be provided.")
            .collect(),
    };

    for path_str in args.files {
        let path = Path::new(path_str);
        assert!(
            path.is_file(),
            "The path did not appear to be a valid file: {}",
            path_str
        );
        let extension = match path.extension() {
            Some(extension) => {
                if extension == OsStr::new("dtd") {
                    Some(Extension::Dtd)
                } else if extension == OsStr::new("properties") {
                    Some(Extension::Properties)
                } else {
                    None
                }
            }
            None => None,
        };

        if let Some(extension) = extension {
            let string = fs::read_to_string(path).expect("Failed to read file.");
            println!(
                "\n# {}\n# ===================================================================",
                path.display()
            );
            match extension {
                Extension::Dtd => {
                    let nodes = parse!(dtd, &string).1;
                    println!("{}", dtd_to_fluent(&nodes));
                }
                Extension::Properties => {
                    let nodes = parse!(properties, &string).1;
                    println!("{:#?}", nodes);
                }
            };
        } else {
            println!(
                "The following file does not have a .dtd extension: {}",
                path_str,
            );
        }
    }

    println!("Done parsing files");
}
