use std::{
    fs::File,
    io::{Read, Write},
    path::PathBuf,
};

use clap::{arg, command, value_parser, ArgAction::Append};
use regex::Regex;

const BUNDLE_LOGIC: &str =
    "local ____bundle__funcs, ____bundle__files, ____bundle__global_require = {}, {}, require
local require = function(path)
    if ____bundle__files[path] then
        return ____bundle__files[path]
    elseif ____bundle__funcs[path] then
        ____bundle__files[path] = ____bundle__funcs[path]()
        return ____bundle__files[path]
    end
    return ____bundle__global_require(path)
end
";

fn main() {
    let matches = command!() // requires `cargo` feature
        .arg(arg!(<OUTPUT> "Output file").value_parser(value_parser!(PathBuf)))
        .arg(arg!(<SOURCE_DIR> "Source directory").value_parser(value_parser!(PathBuf)))
        .arg(arg!(<MAIN> "Main file (relative)").value_parser(value_parser!(PathBuf)))
        .arg(
            arg!([PACKAGES] "Packages to bundle (relative, use the path you would use in require)")
                .value_parser(value_parser!(PathBuf))
                .action(Append),
        )
        .arg(arg!(-a --"auto-detect" "Automatically detect packages to bundle"))
        .get_matches();

    let mut packages: Vec<PathBuf> = matches
        .get_many::<PathBuf>("PACKAGES")
        .unwrap_or_default()
        .map(|x| x.clone())
        .collect();
    let output = matches.get_one::<PathBuf>("OUTPUT").unwrap();
    let source_dir = matches.get_one::<PathBuf>("SOURCE_DIR").unwrap();
    let main = &source_dir.join(matches.get_one::<PathBuf>("MAIN").unwrap());
    let auto_detect = matches.get_flag("auto-detect");

    if !source_dir.is_dir() {
        println!("Source directory has to be a directory.");
        return;
    }

    if !main.is_file() {
        println!("Main has to be a file.");
        return;
    }

    if output.exists() && !output.is_file() {
        println!("Output has to be a file.");
        return;
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

    if auto_detect {
        let require_regex = Regex::new(r#"require\("([^"]+)"\)"#).unwrap();
        for capture in require_regex.captures_iter(&main_contents) {
            let package = PathBuf::from(capture.get(1).unwrap().as_str());
            if !packages.contains(&package) {
                packages.push(package);
            }
        }
    }

    let mut output_file = match File::create(output) {
        Ok(file) => file,
        Err(e) => {
            println!("Failed to create output file: {}", e);
            return;
        }
    };

    if packages.is_empty() {
        match output_file.write_all(main_contents.as_bytes()) {
            Ok(_) => (),
            Err(e) => {
                println!("Failed to write to output file: {}", e);
                return;
            }
        };
        return;
    }

    let mut output_contents = BUNDLE_LOGIC.to_owned();

    for package in packages {
        let mut package = source_dir.join(package);
        package.set_extension("lua");
        let package = &package;

        if !package.is_file() {
            println!("Package {} is not a file.", package.display());
            continue;
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
            "____bundle__funcs[{:?}] = function()
{}
end
",
            package.file_stem().unwrap(),
            package_contents,
        ));
    }

    if main_contents.starts_with(BUNDLE_LOGIC) {
        output_contents.push_str(&main_contents[BUNDLE_LOGIC.len()..]);
    } else {
        output_contents.push_str(&main_contents);
    }

    match output_file.write_all(output_contents.as_bytes()) {
        Ok(_) => (),
        Err(e) => {
            println!("Failed to write to output file: {}", e);
            return;
        }
    };

    println!("Done!");
}
