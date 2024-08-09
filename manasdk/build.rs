use std::{fs, panic};
use std::fs::File;
use std::io::Write;

use rust_format::{Formatter, PrettyPlease};

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let classes = "dump/ClassesInfo.json";
    let structs = "dump/StructsInfo.json";
    let enums = "dump/EnumsInfo.json";
    let offsets = "dump/OffsetsInfo.json";
    let functions = "dump/FunctionsInfo.json";
    let gobjects = "dump/GObjects-Dump.txt";

    let output_path = format!("{}/generated_code.rs", out_dir);
    let exclusions = vec!["UObject", "UClass", "UFunction", "UStruct", "UField"];
    
    let definitions = generator::generate_code(classes, structs, enums, gobjects, offsets, &exclusions).expect("Failed to generate code");

    let result = PrettyPlease::default().format_tokens(definitions).expect("Failed to format code");

    let mut file = File::create(output_path).expect("Failed to create output file");
    write!(file, "{}", result).unwrap();


    println!("cargo::rerun-if-changed=build.rs");
}