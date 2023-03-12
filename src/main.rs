use std::{
    fs::File,
    io::{Read, Write},
    path::PathBuf,
};

use clap::{arg, command, value_parser, ArgAction::Append};

fn main() {
    let matches = command!() // requires `cargo` feature
        .arg(arg!([OUTPUT] "Output file").value_parser(value_parser!(PathBuf)))
        .arg(arg!([MAIN] "Main file").value_parser(value_parser!(PathBuf)))
        .arg(
            arg!([PACKAGES] "Packages to bundle")
                .value_parser(value_parser!(PathBuf))
                .action(Append),
        )
        .get_matches();

    let packages: Vec<&PathBuf> = match matches.get_many::<PathBuf>("PACKAGES") {
        Some(packages) => packages.collect(),
        None => {
            println!("You probably want at least one package.");
            return;
        }
    };
    let output = matches.get_one::<PathBuf>("OUTPUT").unwrap();
    let main = matches.get_one::<PathBuf>("MAIN").unwrap();

    if !main.is_file() {
        println!("Main has to be a file.");
        return;
    }

    if !output.is_file() {
        println!("Output has to be a file.");
        return;
    }

    let mut output_file = match File::create(output) {
        Ok(file) => file,
        Err(e) => {
            println!("Failed to create output file: {}", e);
            return;
        }
    };

    let mut output_contents = "local ____bundle__files = {}
local ____bundle__global_require = require
local require = function(path)
    return ____bundle__files[path] or ____bundle__global_require(path)
end
"
    .to_owned();

    for package in packages {
        if !package.is_file() {
            println!("Packages have to be files. ({})", package.display());
            return;
        }

        let mut package_file = match File::open(package) {
            Ok(file) => file,
            Err(e) => {
                println!("Failed to open package file: {}", e);
                return;
            }
        };

        let mut package_contents = String::new();
        match package_file.read_to_string(&mut package_contents) {
            Ok(_) => (),
            Err(e) => {
                println!("Failed to read package file: {}", e);
                return;
            }
        };

        output_contents.push_str(&format!(
            "____bundle__files[{:?}] = (function()
{}
end)()
____bundle__files[{:?}] = ____bundle__files[{:?}]
",
            package.file_stem().unwrap(),
            package_contents,
            package.file_name().unwrap(),
            package.file_stem().unwrap()
        ));
    }

    let mut main_file = match File::open(main) {
        Ok(file) => file,
        Err(e) => {
            println!("Failed to open main file: {}", e);
            return;
        }
    };

    let mut main_contents = String::new();
    match main_file.read_to_string(&mut main_contents) {
        Ok(_) => (),
        Err(e) => {
            println!("Failed to read main file: {}", e);
            return;
        }
    };

    output_contents.push_str(main_contents.as_str());

    match output_file.write_all(output_contents.as_bytes()) {
        Ok(_) => (),
        Err(e) => {
            println!("Failed to write to output file: {}", e);
            return;
        }
    };

    println!("Done!");
}
