#[macro_use]
pub mod dtd;
pub mod fluent;

use clap::{App, Arg};
use dtd::dtd;
use fluent::dtd_to_fluent;
use std::{ffi::OsStr, fs, path::Path};

fn main() {
    let matches = App::new("DTD to Fluent Migrator")
        .about("Take Firefox DTD and migrate them to a Fluent syntax")
        .arg(
            Arg::with_name("DTD_FILES")
                .index(1)
                .multiple(true)
                .required(true)
                .help("The space separated list dtd paths to migrate"),
        )
        .get_matches();

    matches
        .values_of("DTD_FILES")
        .expect("At least one dtd file must be provided.")
        .map(|path_str| {
            let path = Path::new(path_str);
            assert!(
                path.is_file(),
                "The path did not appear to be a valid file: {}",
                path_str
            );
            assert_eq!(
                Some(OsStr::new("dtd")),
                path.extension(),
                "The following file does not have a .dtd extension: {}",
                path_str
            );
            path
        })
        .for_each(|path| {
            let string = fs::read_to_string(path).expect("Failed to read file.");
            let nodes = parse!(dtd, &string).1;
            // println!("\n{}\n\n{:#?}", path.display(), parse!(dtd, &string));

            println!(
                "\n# {}\n# ===================================================================",
                path.display()
            );
            println!("{}", dtd_to_fluent(&nodes));
        });

    println!("Done parsing dtds");
}
