use regex::Regex;
use rust_format::{Formatter, PrettyPlease};
use std::fs::File;
use std::io::Write;

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();

    let output_path = format!("{}/generated_code.rs", out_dir);
    let exclusions = vec![
        "UObject",
        "UClass",
        "UProperty",
        "FProperty",
        "UFunction",
        "UStruct",
        "UField",
        "TArray",
        "TSoftClassPtr",
        "TLazyObjectPtr",
        "TSoftObjectPtr",
        "TWeakObjectPtr",
        "FWeakObjectPtr",
        "TMap",
        "TSet",
        "FText",
        "FString",
        "FName",
    ];

    let definitions = generator::generate_code(
        "dump",
        &exclusions,
        Some(Regex::new(r#"CoreUObject|Engine"#).unwrap()),
    )
    .expect("Failed to generate code");

    let result = PrettyPlease::default()
        .format_tokens(definitions)
        .expect("Failed to format code");

    let mut file = File::create(output_path).expect("Failed to create output file");
    write!(file, "{}", result).unwrap();

    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=dump");
}
