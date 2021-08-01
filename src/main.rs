#![allow(unused_imports)]

pub mod fluent;
#[macro_use]
pub mod parser;

use clap::{App, Arg};
use fluent::nodes_to_fluent;
use parser::dtd::dtd;
use parser::properties::properties;
use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
};

struct ParsedArgs<'a> {
    files: Vec<&'a str>,
    save: bool,
    overwrite: bool,
}

enum Extension {
    Dtd,
    Properties,
}

fn main() {
    let matches = App::new("Fluent Migrator")
        .version("v1.0.0")
        .about(
            "\nTake Firefox .dtd or .property files and migrate them to the .ftl syntax.\nBy default the results are output to std out, instead of to files."
        )
        .arg(
            Arg::with_name("files")
                .index(1)
                .multiple(true)
                .required(true)
                .help("The space separated list .dtd or .properties paths to migrate"),
        )
        .arg(
            Arg::from_usage("--save... 'Save the file next to the existing one with a .ftl extension'"),
        )
        .arg(
            Arg::from_usage("--overwrite... 'Overwrite an .ftl file if it already exists'"),
        )
        .get_matches();

    let args = ParsedArgs {
        files: matches
            .values_of("files")
            .expect("At least one dtd file must be provided.")
            .collect(),
        save: matches.is_present("save"),
        overwrite: matches.is_present("overwrite"),
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
            let nodes = match extension {
                Extension::Dtd => parse!(dtd, &string).1,
                Extension::Properties => parse!(properties, &string).1,
            };
            let fluent_text = nodes_to_fluent(&nodes);
            if args.save {
                let mut save_path = PathBuf::from(path);
                assert!(save_path.set_extension("ftl"));
                if save_path.is_file() && !args.overwrite {
                    // The file exists, warn but don't overwrite.
                    println!(
                        "Skipping file as it exists, use --overwrite to replace: {}",
                        save_path.display()
                    );
                    continue;
                }
                match fs::write(save_path.clone(), fluent_text) {
                    Ok(_) => println!("Saved: {}", save_path.display()),
                    Err(err) => println!("Failed to write: {}\n{}", save_path.display(), err),
                };
            } else {
                println!(
                    "\n# {}\n# ===================================================================",
                    path.display()
                );
                println!("{}", fluent_text);
            }
        } else {
            println!(
                "The following file cannot be converted as it does not have a .dtd or .properties extension:\n{}",
                path_str,
            );
        }
    }

    println!("Done migrating files");
}
