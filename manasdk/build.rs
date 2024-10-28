use std::collections::HashSet;
use regex::Regex;
use rust_format::{Formatter, PrettyPlease};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

fn main() {
    let out_dir: PathBuf = std::env::var("OUT_DIR").unwrap().try_into().unwrap();
    let output_path = out_dir.join("generated_code");

    std::fs::create_dir_all(&output_path).unwrap();

    let exclusions = vec![
        ("UObject", Some("core_u_object")),
        ("UClass", Some("core_u_object")),
        ("UProperty", Some("core_u_object")),
        ("FProperty", None),
        ("UFunction", Some("core_u_object")),
        ("UStruct", Some("core_u_object")),
        ("UField", Some("core_u_object")),
        ("TArray", None),
        ("TSoftClassPtr", None),
        ("TLazyObjectPtr", None),
        ("TSoftObjectPtr", None),
        ("TWeakObjectPtr", None),
        ("FWeakObjectPtr", None),
        ("TMap", None),
        ("TSet", None),
        ("FText", None),
        ("FString", None),
        ("FName", None),
        ("FInputKeyEventArgs", Some("engine")),
        ("FKey", Some("input_core")),
    ];
    
    let excluded_classes = exclusions.iter().map(|it| it.0).collect::<Vec<_>>();
    let modified_packages = exclusions.into_iter().filter_map(|it| it.1).collect::<HashSet<_>>();

    let definitions = generator::generate_code(
        "dump",
        &excluded_classes,
        Some(Regex::new(r#"core_u_object|engine|x21|py_enemy_base"#).unwrap()),
    )
    .expect("Failed to generate code");

    let modules: Vec<_> = definitions
        .iter()
        .filter_map(|(package, _)| package.clone())
        .collect();

    for (package, mut def) in definitions {
        match package.clone() {
            None => {
                let mut imports = String::new();
                for module in modules.iter() {
                    imports.push_str(format!("pub mod {};", module).as_str());
                }

                def.insert_str(0, imports.as_str());
            }
            Some(package) if modified_packages.contains(package.as_str()) => {
                def.insert_str(0, format!("pub use crate::overrides::{}::*;", package).as_str());
            }
            Some(_) => {
                ()
            }
        }

        let module = package.unwrap_or("lib".to_string());
        let path = output_path.join(format!("{}.rs", module));

        let result = PrettyPlease::default()
            .format_str(def)
            .expect(format!("Failed to format code: {}", module).as_str());
        let mut file = File::create(path).expect("Failed to create output file");
        write!(file, "{}", result).unwrap();
    }

    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=dump");
}
