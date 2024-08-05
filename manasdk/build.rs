use serde::Deserialize;
use std::collections::HashMap;
use std::io::Write;

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum FieldDefinition {
    InheritInfo(Vec<String>),
    MDKClassSize(u32),
    Field((FieldSignature, u32, u32, u32)),
    FieldWithDefault((FieldSignature, u32, u32, u32, u32)),
}

#[derive(Deserialize, Debug)]
struct FieldSignature(String, String, String, Vec<FieldSignature>);

#[derive(Deserialize, Debug)]
struct FieldType(FieldSignature, usize, usize, usize);

#[derive(Deserialize, Debug)]
struct InheritInfo {
    #[serde(rename = "__InheritInfo")]
    inherit_info: Vec<String>,
}

#[derive(Deserialize, Debug)]
struct MDKClassSize {
    #[serde(rename = "__MDKClassSize")]
    inherit_info: usize,
}


#[derive(Deserialize, Debug)]
struct JsonData {
    data: Vec<HashMap<String, Vec<HashMap<String, FieldDefinition>>>>,
}

pub fn generate_code(json_path: &str, output_path: &str) -> std::io::Result<()> {
    let file = std::fs::File::open(json_path)?;
    let json_data: JsonData = serde_json::from_reader(file)?;

    let mut output = String::new();

    for struct_data in json_data.data {
        for (struct_name, struct_defs) in struct_data {
            output.push_str(&format!("#[repr(C)]\npub struct {} {{\n", struct_name));
            for field in struct_defs {
                for (field_name, struct_def) in field {
                    if let FieldDefinition::Field(struct_def) = struct_def {
                        let field_type_name = match struct_def.0.0.as_str() {
                            "float" => "f32".to_string(),
                            "double" => "f64".to_string(),
                            "int64" => "i64".to_string(),
                            "int32" => "i32".to_string(),
                            "int16" => "i16".to_string(),
                            "int8" => "i8".to_string(),
                            "uint64" => "u64".to_string(),
                            "uint32" => "u32".to_string(),
                            "uint16" => "u16".to_string(),
                            "uint8" => "u8".to_string(),
                            "bool" => "bool".to_string(),
                            _ => format!("[u8;{}]", struct_def.2), // handle other types as needed
                        };
                        
                        let field_name = match field_name.as_str() {
                            "type" => "_type",
                            "enum" => "_enum",
                            &_ => { field_name.as_str() }
                        };
                        
                        output.push_str(&format!("    pub {}: {},\n", field_name, field_type_name));
                    }
                }
            }
            output.push_str("}\n\n");
            
        }
    }

    let mut file = std::fs::File::create(output_path)?;
    file.write_all(output.as_bytes())?;
    Ok(())
}


fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let json_path = "dump/StructsInfo.json"; // update this path
    let output_path = format!("{}/generated_code.rs", out_dir);

    generate_code(json_path, &output_path).expect("Failed to generate code");

    println!("cargo:rerun-if-changed={}", json_path);
}