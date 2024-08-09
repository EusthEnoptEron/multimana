use std::collections::{HashMap, HashSet};
use std::io::Read;
use serde::Deserialize;
use crate::{EnumDefinition, EnumDump, EnumKind, FieldKind, StructDefinition, StructDump, TypeSignature};

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
    pub updated_at: String
}

#[derive(Deserialize, Debug, Eq, PartialEq)]
pub struct OffsetData {
    pub data: Vec<Offset>,
}

#[derive(Deserialize, Debug, Eq, PartialEq)]
pub struct Offset(
    pub String,
    pub usize,
);


impl EnumDump {
    pub fn from_raw_json<R>(source: R) -> serde_json::Result<Self>
    where
        R: Read,
    {
        let raw: RawEnumDump = serde_json::from_reader(source)?;

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
                let mut taken = HashSet::new();
                let mut options = vec![];

                for it in enum_def.0 {
                    let (mut option_name, option_value) = it.into_iter().nth(0).unwrap();
                    if option_name == "Self" {
                        option_name = "Self_".to_string();
                    }

                    let option = if option_value < 0 {
                        Some((option_name, max_val))
                    } else if option_value <= max_val as i64 {
                        Some((option_name, option_value as u64))
                    } else {
                        None
                    };

                    if let Some(option) = option {
                        if taken.insert(option.1) {
                            options.push(option);
                        }
                    }
                }

                EnumDefinition {
                    name: enum_name.replace(":", "_"),
                    kind,
                    options,
                    package: None,
                }
            }).collect()
        })
    }
}


impl StructDump {
    pub fn from_raw_json<R>(source: R) -> serde_json::Result<Self>
    where
        R: Read,
    {
        let raw: JsonData = serde_json::from_reader(source)?;

        Ok(StructDump {
            data: raw.data.into_iter().map(|map| {
                let (name, description) = map.into_iter().nth(0).unwrap();
                let mut result = StructDefinition {
                    name: name.replace(":", "_"),
                    parents: vec![],
                    struct_size: 0,
                    fields: vec![],
                    package: None,
                };

                for (field_name, definition) in description.into_iter().flatten() {
                    match definition {
                        FieldDefinition::InheritInfo(classes) => {
                            result.parents = classes;
                        }
                        FieldDefinition::MDKClassSize(size) => {
                            result.struct_size = size;
                        }
                        FieldDefinition::Field(def) => {
                            result.fields.push(crate::FieldDefinition::new(
                                field_name,
                                def.1,
                                def.2,
                                def.3,
                                None,
                                def.0.into(),
                            ));
                        }
                        FieldDefinition::FieldWithBitOffset(def) => {
                            result.fields.push(crate::FieldDefinition::new(
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


impl From<FieldSignature> for TypeSignature {
    fn from(value: FieldSignature) -> Self {
        Self {
            name: match value.0.as_str() {
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
                name => name.replace(":", "_"), // handle other types as needed
            },
            kind: match value.1.as_str() {
                "S" => FieldKind::Struct,
                "C" => FieldKind::Class,
                "E" => FieldKind::Enum,
                "D" => FieldKind::Primitive,
                _ => panic!("Unknown data type: {}", value.1)
            },
            is_pointer: match value.2.as_str() {
                "*" => true,
                "" => false,
                _ => panic!("Unknown keyword: {}", value.2)
            },
            generics: value.3.into_iter().map(|it| it.into()).collect(),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_offsets() {
        let result: OffsetData = serde_json::from_str(r#"{   
            "credit": {
                "dumper_link": "https://github.com/Encryqed/Dumper-7",
                "dumper_used": "Dumper-7"
            },
            "data": [["key", 0]]}"#).unwrap();
        
        assert_eq!(result, OffsetData {
            data: vec![Offset("key".to_string(), 0)]
        });
    }
}