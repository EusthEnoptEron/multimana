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

    generator::generate_code(classes, structs, enums, gobjects, &output_path, &exclusions).expect("Failed to generate code");

    println!("cargo::rerun-if-changed=build.rs");
}