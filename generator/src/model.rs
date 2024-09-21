use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use heck::ToSnakeCase;
use regex::Regex;

#[derive(Clone, Debug)]
pub struct Manifest {
    pub packages: HashSet<String>,
    // Name to package
    pub structs: HashMap<String, String>,
}

impl FromStr for Manifest {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let regex = Regex::new(r"^\[[0-9A-F]+\] \{0x[0-9a-f]+\} (?<type>.+?) (?<package>.+?)(?:\.(?<name>.+))?$")?;
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

#[derive(Clone, Debug)]
pub struct EnumDump {
    pub data: Vec<EnumDefinition>,
}

#[derive(Clone, Debug)]
pub struct FunctionDump {
    pub data: HashMap<String, Vec<FunctionDefinition>>,
}

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug)]
pub struct EnumDefinition {
    pub name: String,
    pub kind: EnumKind,
    pub options: Vec<(String, u64)>,
    pub package: Option<String>,
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
                .map(|it| it.to_snake_case());

            item.package = package;

            if let Some(old_value) = self.classes.insert(item.name.clone(), item) {
                panic!("Value clash {:?}", old_value);
            }
        }
    }

    pub fn add_function_dump(&mut self, dump: FunctionDump) {
        for (class, functions) in dump.data {
            self.classes.entry(class).and_modify(|class| {
                class.functions = functions;
            });
        }
    }

    pub fn add_enum_dump(&mut self, dump: EnumDump) {
        self.enums.reserve(dump.data.len());
        for mut item in dump.data {
            let package = self.manifest.structs.get(&item.name[1..])
                .or_else(|| self.manifest.structs.get(&item.name))
                .map(|it| it.to_snake_case());

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

#[derive(Clone, Debug)]
pub struct StructDump
{
    pub data: Vec<StructDefinition>,
}

#[derive(Clone, Debug)]
pub struct StructDefinition {
    pub name: String,
    pub parents: Vec<String>,
    pub struct_size: usize,
    pub fields: Vec<FieldDefinition>,
    pub package: Option<String>,
    pub functions: Vec<FunctionDefinition>,
}

#[derive(Clone, Debug)]
pub struct FieldDefinition {
    pub name: String,
    pub offset: usize,
    pub size: usize,
    pub bit_offset: Option<u8>,
    pub unknown: usize,
    pub signature: TypeSignature,
}

#[derive(Clone, Debug)]
pub struct FunctionDefinition {
    pub name: String,
    pub return_value: TypeSignature,
    pub arguments: Vec<ArgumentDefinition>,
    pub flags: String,
    pub offset: usize,
}

#[derive(Clone, Debug)]
pub struct ArgumentDefinition {
    pub name: String,
    pub is_out_param: bool,
    pub type_: TypeSignature,
}

impl FieldDefinition {
    pub fn new(name: String, offset: usize, size: usize, unknown: usize, bit_offset: Option<u8>, signature: TypeSignature) -> Self {
        Self { name, offset, size, bit_offset, unknown, signature }
    }
}

#[derive(Clone, Debug)]
pub struct TypeSignature {
    pub name: String,
    pub kind: FieldKind,
    pub is_pointer: bool,
    pub generics: Vec<TypeSignature>,
}

impl TypeSignature {
    pub fn has_pointers(&self) -> bool {
        if self.is_pointer {
            true
        } else {
            self.generics.iter().any(|it| it.has_pointers())
        }
    }

    pub fn new_struct(name: String) -> Self {
        Self {
            name,
            kind: FieldKind::Struct,
            is_pointer: false,
            generics: vec![],
        }
    }

    pub fn new_simple(name: String, kind: FieldKind) -> TypeSignature {
        Self {
            name,
            kind,
            is_pointer: false,
            generics: vec![],
        }
    }

    pub fn new_pointer(name: String, kind: FieldKind) -> TypeSignature {
        Self {
            name,
            kind,
            is_pointer: true,
            generics: vec![],
        }
    }

    pub fn fill_types(&self, target: &mut HashSet<Reference>) {
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FieldKind {
    Struct,
    Class,
    Primitive,
    Enum,
}