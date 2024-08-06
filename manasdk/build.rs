use std::arch::x86_64::_mm_cmpestrc;
use std::collections::HashSet;
use std::fs;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::ptr::write;
use heck::ToSnakeCase;
use regex::Regex;
use crate::io::{ClassLookup, EnumDefinition, EnumDump, Manifest, Reference, StructDefinition, StructDump};

mod io {
    use std::collections::{HashMap, HashSet};
    use std::fmt::{Display, Formatter};
    use std::io::Read;
    use std::str::FromStr;
    use regex::Regex;

    mod raw {
        use std::collections::HashMap;

        use serde::Deserialize;

        #[derive(Debug, Deserialize)]
        pub struct RawEnumDump {
            pub data: Vec<HashMap<String, (Vec<HashMap<String, i64>>, String)>>,
        }

        #[derive(Debug, Deserialize)]
        #[serde(untagged)]
        pub enum FieldDefinition {
            InheritInfo(Vec<String>),
            MDKClassSize(usize),
            Field((FieldSignature, usize, usize, usize)),
            FieldWithBitOffset((FieldSignature, usize, usize, usize, i64)),
        }

        #[derive(Deserialize, Debug)]
        pub struct FieldSignature(pub String, pub String, pub String, pub Vec<FieldSignature>);

        #[derive(Deserialize, Debug)]
        struct FieldType(FieldSignature, usize, usize, usize);

        #[derive(Deserialize, Debug)]
        struct InheritInfo {
            #[serde(rename = "__InheritInfo")]
            pub inherit_info: Vec<String>,
        }

        #[derive(Deserialize, Debug)]
        struct MDKClassSize {
            #[serde(rename = "__MDKClassSize")]
            pub size: usize,
        }


        #[derive(Deserialize, Debug)]
        pub struct JsonData {
            pub data: Vec<HashMap<String, Vec<HashMap<String, FieldDefinition>>>>,
        }
    }

    #[derive(Debug)]
    pub struct Manifest {
        pub packages: HashSet<String>,
        // Name to package
        pub structs: HashMap<String, String>,
    }

    impl FromStr for Manifest {
        type Err = ();

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let regex = Regex::new(r"^\[[0-9A-F]+\] \{0x[0-9a-f]+\} (?<type>.+?) (?<package>.+?)(?:\.(?<name>.+))?$").unwrap();
            let mut result = Self {
                structs: HashMap::new(),
                packages: HashSet::new(),
            };

            s.lines()
                .filter_map(|it| regex.captures(it))
                .for_each(|item| {
                    let is_first = match &item["type"] {
                        "Package" => { result.packages.insert(item["package"].to_string()) }
                        _ => { result.structs.insert(item["name"].to_string(), item["package"].to_string()).is_none() }
                    };

                    if !is_first {
                        eprintln!("Duplicate entry: type={} package={} name={:?}", &item["type"], &item["package"], item.name("name"));
                    }
                });

            Ok(result)
        }
    }

    #[derive(Debug)]
    pub struct EnumDump {
        pub data: Vec<EnumDefinition>,
    }

    #[derive(Debug)]
    pub enum EnumKind {
        U8,
        U16,
        U32,
        U64,
    }

    #[derive(Debug, Clone, PartialEq, Eq, Hash)]
    pub enum Reference {
        Struct(String),
        Enum(String),
    }

    impl Display for EnumKind {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", match self {
                EnumKind::U8 => "u8",
                EnumKind::U16 => "u16",
                EnumKind::U32 => "u32",
                EnumKind::U64 => "u64",
            })
        }
    }

    impl EnumKind {
        pub fn max_val(&self) -> u64 {
            match self {
                EnumKind::U8 => u8::MAX as u64,
                EnumKind::U16 => u16::MAX as u64,
                EnumKind::U32 => u32::MAX as u64,
                EnumKind::U64 => u64::MAX,
            }
        }
    }

    #[derive(Debug)]
    pub struct EnumDefinition {
        pub name: String,
        pub kind: EnumKind,
        pub options: Vec<(String, u64)>,
        pub package: Option<String>
    }

    impl EnumDump {
        pub fn from_raw_json<R>(source: R) -> serde_json::Result<Self>
        where
            R: Read,
        {
            let raw: raw::RawEnumDump = serde_json::from_reader(source)?;

            Ok(Self {
                data: raw.data.into_iter().map(|enum_type| {
                    let (enum_name, enum_def) = enum_type.into_iter().nth(0).unwrap();
                    let kind = match enum_def.1.as_str() {
                        "uint8" => EnumKind::U8,
                        "uint16" => EnumKind::U16,
                        "uint32" => EnumKind::U32,
                        "uint64" => EnumKind::U64,
                        &_ => panic!("Invalid enum kind: {}", enum_def.1)
                    };

                    let max_val = kind.max_val();

                    EnumDefinition {
                        name: enum_name.replace(":", "_"),
                        kind: kind,
                        options: enum_def.0.into_iter().map(|it| {
                            let (option_name, option_value) = it.into_iter().nth(0).unwrap();
                            if option_value < 0 {
                                (option_name, max_val)
                            } else {
                                (option_name, option_value as u64)
                            }
                        }).collect(),
                        package: None
                    }
                }).collect()
            })
        }
    }


    pub struct ClassLookup {
        classes: HashMap<String, StructDefinition>,
        enums: HashMap<String, EnumDefinition>,
        manifest: Manifest,
        filter: Option<Regex>,
    }

    impl ClassLookup {
        pub fn new(manifest: Manifest, filter: Option<Regex>) -> Self {
            Self {
                manifest,
                filter,
                classes: HashMap::new(),
                enums: HashMap::new(),
            }
        }

        pub fn add_struct_dump(&mut self, dump: StructDump) {
            self.classes.reserve(dump.data.len());
            for mut item in dump.data {
                let package = self.manifest.structs.get(&item.name[1..])
                    .or_else(|| self.manifest.structs.get(&item.name))
                    .cloned();
                
                item.package = package;
                
                if let Some(old_value) = self.classes.insert(item.name.clone(), item) {
                    panic!("Value clash {:?}", old_value);
                }
            }
        }

        pub fn add_enum_dump(&mut self, dump: EnumDump) {
            self.enums.reserve(dump.data.len());
            for mut item in dump.data {
                let package = self.manifest.structs.get(&item.name[1..])
                    .or_else(|| self.manifest.structs.get(&item.name))
                    .cloned();

                item.package = package;
                
                if let Some(old_value) = self.enums.insert(item.name.clone(), item) {
                    panic!("Value clash {:?}", old_value);
                }
            }
        }

        pub fn get_struct(&self, name: &str) -> Option<&StructDefinition> {
            self.classes.get(name)
        }

        pub fn get_enum(&self, name: &str) -> Option<&EnumDefinition> {
            self.enums.get(name)
        }

        pub fn iter_structs(&self) -> impl Iterator<Item=&StructDefinition> {
            self.classes.values().filter(|&class| {
                let package = class.package.as_ref();
                if let Some(filter) = &self.filter {
                    package.is_none() || filter.is_match(package.unwrap().as_str())
                } else { true }
            })
        }

        pub fn iter_enums(&self) -> impl Iterator<Item=&EnumDefinition> {
            self.enums.values().filter(|&class| {
                let package = class.package.as_ref();
                if let Some(filter) = &self.filter {
                    package.is_none() || filter.is_match(package.unwrap().as_str())
                } else { true }
            })
        }
    }

    #[derive(Debug)]
    pub struct StructDump
    {
        pub data: Vec<StructDefinition>,
    }

    impl StructDump {
        pub fn from_raw_json<R>(source: R) -> serde_json::Result<Self>
        where
            R: Read,
        {
            let raw: raw::JsonData = serde_json::from_reader(source)?;

            Ok(StructDump {
                data: raw.data.into_iter().map(|map| {
                    let (name, description) = map.into_iter().nth(0).unwrap();
                    let mut result = StructDefinition {
                        name: name.replace(":", "_"),
                        parents: vec![],
                        struct_size: 0,
                        fields: vec![],
                        package: None
                    };

                    for (field_name, definition) in description.into_iter().flatten() {
                        match definition {
                            raw::FieldDefinition::InheritInfo(classes) => {
                                result.parents = classes;
                            }
                            raw::FieldDefinition::MDKClassSize(size) => {
                                result.struct_size = size;
                            }
                            raw::FieldDefinition::Field(def) => {
                                result.fields.push(FieldDefinition::new(
                                    field_name,
                                    def.1,
                                    def.2,
                                    def.3,
                                    None,
                                    def.0.into(),
                                ));
                            }
                            raw::FieldDefinition::FieldWithBitOffset(def) => {
                                result.fields.push(FieldDefinition::new(
                                    field_name,
                                    def.1,
                                    def.2,
                                    def.3,
                                    Some(def.4),
                                    def.0.into(),
                                ));
                            }
                        }
                    }

                    result
                }).collect()
            })
        }
    }

    #[derive(Debug)]
    pub struct StructDefinition {
        pub name: String,
        pub parents: Vec<String>,
        pub struct_size: usize,
        pub fields: Vec<FieldDefinition>,
        pub package: Option<String>
    }

    #[derive(Debug)]
    pub struct FieldDefinition {
        pub name: String,
        pub offset: usize,
        pub size: usize,
        pub bit_offset: Option<i64>,
        pub unknown: usize,
        pub signature: TypeSignature,
    }

    impl FieldDefinition {
        pub fn new(name: String, offset: usize, size: usize, unknown: usize, bit_offset: Option<i64>, signature: TypeSignature) -> Self {
            Self { name, offset, size, bit_offset, unknown, signature }
        }
    }

    #[derive(Debug)]
    pub struct TypeSignature {
        pub name: String,
        pub kind: FieldKind,
        pub keyword: String,
        pub generics: Vec<TypeSignature>,
    }

    impl TypeSignature {
        pub(crate) fn fill_types(&self, target: &mut HashSet<Reference>) {
            match self.kind {
                FieldKind::Struct | FieldKind::Class => {
                    target.insert(Reference::Struct(self.name.clone()));
                }
                FieldKind::Enum => {
                    target.insert(Reference::Enum(self.name.clone()));
                }
                _ => {}
            }

            for x in self.generics.iter() {
                x.fill_types(target);
            }
        }
    }

    impl TypeSignature {
        pub fn to_rust_signature(&self, property_name: Option<&str>) -> String {
            let field_type_name = match self.name.as_str() {
                "float" => "f32".to_string(),
                "double" => "f64".to_string(),
                "int64" => "i64".to_string(),
                "int32" => "i32".to_string(),
                "int16" => "i16".to_string(),
                "int8" => "i8".to_string(),
                "uint64" => "u64".to_string(),
                "uint32" => "u32".to_string(),
                "uint16" => "u16".to_string(),
                "uint8" if { property_name.is_some_and(|it| it.starts_with("b_")) } => "bool".to_string(),
                "uint8" => "u8".to_string(),
                "bool" => "bool".to_string(),
                _ => self.name.clone().replace(":", "_"), // handle other types as needed
            };

            if self.generics.is_empty() {
                format!("{}{}", self.keyword, field_type_name)
            } else {
                format!("{}{}<{}>", self.keyword, field_type_name, self.generics.iter()
                    .map(|it| it.to_rust_signature(None))
                    .collect::<Vec<_>>()
                    .join(",")
                )
            }
        }
    }

    impl From<raw::FieldSignature> for TypeSignature {
        fn from(value: raw::FieldSignature) -> Self {
            Self {
                name: value.0,
                kind: match value.1.as_str() {
                    "S" => FieldKind::Struct,
                    "C" => FieldKind::Class,
                    "E" => FieldKind::Enum,
                    "D" => FieldKind::Primitive,
                    _ => panic!("Unknown data type: {}", value.1)
                },
                keyword: match value.2.as_str() {
                    "*" => "*mut ".to_string(),
                    "" => "".to_string(),
                    _ => panic!("Unknown keyword: {}", value.2)
                },
                generics: value.3.into_iter().map(|it| it.into()).collect(),
            }
        }
    }

    #[derive(Debug, Eq, PartialEq)]
    pub enum FieldKind {
        Struct,
        Class,
        Primitive,
        Enum,
    }
}

fn print_fields(struct_def: &StructDefinition, lut: &ClassLookup, output: &mut impl Write, dependencies: &mut HashSet<Reference>) -> std::io::Result<()> {
    let mut offset = 0;

    let parent = struct_def.parents.first();
    if let Some(parent) = parent {
        // let parent_obj = lut.get_struct(parent);
        // if let Some((parent_obj, _)) = parent_obj {
        //     print_fields(parent_obj, lut, output, dependencies)?;
        //     offset = parent_obj.struct_size;
        // } else {
        write!(output, "    pub {}: {},\n", parent.to_snake_case(), parent)?;
        dependencies.insert(Reference::Struct(parent.clone()));
        if let Some(parent_obj) = lut.get_struct(parent) {
            offset = parent_obj.struct_size;
        } else {
            offset = struct_def.fields.first().map(|it| it.offset).unwrap_or_default();
        }

        // }
    }

    let mut counter = struct_def.parents.len() * 100;

    for field in struct_def.fields.iter() {
        if offset > field.offset {
            // Padding issue
            continue;
        }
        
        if offset < field.offset {
            write!(output, "    pub _padding_{}: [u8;{}],\n", counter, field.offset - offset)?;
            counter += 1;

            offset = field.offset;
        }

        if field.bit_offset.is_some() {
            // Not implemented
            continue;
        }

        let field_name = field.name.to_snake_case();
        let field_name = match field_name.as_str() {
            "type" => "_type".to_string(),
            "enum" => "_enum".to_string(),
            "continue" => "_continue".to_string(),
            "in" => "_in".to_string(),
            "loop" => "_loop".to_string(),
            "struct" => "_struct".to_string(),
            "box" => "_box".to_string(),
            "move" => "_move".to_string(),
            "" => "_unknown_".to_string(),
            _ => {
                if field_name.starts_with(char::is_numeric) {
                    format!("_{}", field_name)
                } else {
                    field_name
                }
            }
        };

        offset = offset + field.size;
        field.signature.fill_types(dependencies);
        write!(output, "    pub {}: {},\n", field_name, field.signature.to_rust_signature(Some(field_name.as_str())))?;
    }

    if offset < struct_def.struct_size {
        write!(output, "    pub _padding_{}: [u8;{}],\n", counter, struct_def.struct_size - offset)?;
    }

    Ok(())
}

pub fn generate_code(structs_path: &str, classes_path: &str, enums_path: &str, gobjects: &str, output_path: &str) -> std::io::Result<()> {
    let manifest: Manifest = std::io::read_to_string(File::open(gobjects)?)?.parse().unwrap();
    let structs_dump: StructDump = StructDump::from_raw_json(File::open(structs_path)?)?;
    let classes_dump: StructDump = StructDump::from_raw_json(File::open(classes_path)?)?;
    let enums_dump: EnumDump = EnumDump::from_raw_json(File::open(enums_path)?)?;

    let mut lut = ClassLookup::new(manifest, Some(Regex::new(r"^X21$").unwrap()));
    lut.add_struct_dump(classes_dump);
    lut.add_struct_dump(structs_dump);
    lut.add_enum_dump(enums_dump);

    let mut tests = BufWriter::new(Vec::new());

    writeln!(tests, "
#[cfg(test)]
mod tests {{
    #![allow(non_snake_case)]
   use std::mem::size_of;
   use super::*;
")?;

    let mut file = std::fs::File::create(output_path)?;
    let mut deps = HashSet::new();
    let mut defined = HashSet::new();

    for struct_data in lut.iter_structs() {
        print_struct(struct_data, &mut defined, &mut deps, &lut, &mut file, &mut tests)?;
    }

    for enum_def in lut.iter_enums() {
        print_enum(enum_def, &mut defined, &mut file)?;
    }

    let mut to_be_defined: HashSet<_> = deps.difference(&defined).cloned().collect();
    while !to_be_defined.is_empty() {
        let mut deps = HashSet::new();

        for reference in to_be_defined {
            match reference {
                Reference::Struct(name) => {
                    if let Some(obj) = lut.get_struct(name.as_str()) {
                        print_struct(obj, &mut defined, &mut deps, &lut, &mut file, &mut tests)?
                    }
                }
                Reference::Enum(name) => {
                    if let Some(obj) = lut.get_enum(name.as_str()) {
                        print_enum(obj, &mut defined, &mut file)?
                    }
                }
            }
        }

        to_be_defined = deps.difference(&defined).cloned().collect();
    }


    writeln!(tests, "\n}}")?;
    tests.flush()?;

    file.write_all(tests.get_ref())?;

    Ok(())
}

fn print_enum(enum_def: &EnumDefinition, defined: &mut HashSet<Reference>, file: &mut impl Write) -> std::io::Result<()> {
    if !defined.insert(Reference::Enum(enum_def.name.clone())) {
        return Ok(());
    }

    write!(file, "#[repr({})]\n#[derive(Debug, Clone)]\npub enum {} {{\n", enum_def.kind, enum_def.name)?;

    let max_value = enum_def.kind.max_val();
    let mut values = HashSet::new();
    for (option_name, option_value) in enum_def.options.iter() {
        if !values.insert(*option_value) || *option_value > max_value {
            continue;
        }

        let option_name = match option_name.as_str() {
            "Self" => "_Self",
            &_ => option_name.as_str()
        };
        write!(file, "    {} = {},\n", option_name, option_value)?;
    }

    write!(file, "}}\n\n")?;

    Ok(())
}

fn print_struct(struct_data: &StructDefinition, defined: &mut HashSet<Reference>, mut deps: &mut HashSet<Reference>, lut: &ClassLookup, mut output: &mut impl Write, mut tests: &mut impl Write) -> std::io::Result<()> {
    if !defined.insert(Reference::Struct(struct_data.name.clone())) {
        return Ok(());
    }

    write!(output, "#[repr(C)]\n#[derive(Debug, Clone)]\npub struct {} {{\n", struct_data.name)?;
    print_fields(&struct_data, &lut, &mut output, &mut deps)?;
    write!(output, "}}\n\n")?;


    writeln!(tests, "
        #[test]
        fn test_{}() {{
            assert_eq!(size_of::<{}>(), {});
        }}
        ", struct_data.name, struct_data.name, struct_data.struct_size
    )?;

    Ok(())
}

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let classes = "dump/ClassesInfo.json";
    let structs = "dump/StructsInfo.json";
    let enums = "dump/EnumsInfo.json";
    let offsets = "dump/OffsetsInfo.json";
    let functions = "dump/FunctionsInfo.json";
    let gobjects = "dump/GObjects-Dump.txt";

    let output_path = format!("{}/generated_code.rs", out_dir);

    generate_code(classes, structs, enums, gobjects, &output_path).expect("Failed to generate code");

    println!("cargo:rerun-if-changed=build.rs");
}