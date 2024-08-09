use std::collections::{HashSet, VecDeque};
use std::fs::File;
use std::io::{Write};

use heck::ToSnakeCase;
use proc_macro2::{TokenStream};
use quote::{format_ident, quote, TokenStreamExt, ToTokens};
use regex::Regex;
use syn::{Ident, parse_str};

use crate::{ClassLookup, EnumDefinition, EnumDump, EnumKind, FieldDefinition, FieldKind, Manifest, StructDefinition, StructDump, TypeSignature};
use crate::serialization::OffsetData;

trait ToRustCode {
    fn name(&self) -> &str;
    fn package(&self) -> Option<&str>;
    fn generate_code(&self, context: &ClassLookup) -> TokenStream;
    fn generate_test(&self, context: &ClassLookup) -> Option<TokenStream>;
}

struct TypeIterator<'a> {
    lookup: &'a ClassLookup,
    tracked_types: HashSet<&'a str>,
    struct_queue: VecDeque<&'a StructDefinition>,
    enum_queue: VecDeque<&'a EnumDefinition>,
}
impl<'a> TypeIterator<'a> {
    fn new(lookup: &'a ClassLookup) -> Self {
        let mut result = Self {
            lookup,
            tracked_types: HashSet::new(),
            struct_queue: VecDeque::new(),
            enum_queue: VecDeque::new(),
        };

        for def in lookup.iter_structs() {
            result.track_struct(def);
        }

        for def in lookup.iter_enums() {
            result.track_enum(def);
        }

        result
    }

    fn track_struct(&mut self, def: &'a StructDefinition) {
        if self.tracked_types.insert(&def.name) {
            self.struct_queue.push_back(def);
        }
    }

    fn track_enum(&mut self, def: &'a EnumDefinition) {
        if self.tracked_types.insert(&def.name) {
            self.enum_queue.push_back(def);
        }
    }

    fn track_by_name(&mut self, name: &str) {
        if let Some(def) = self.lookup.get_struct(name) {
            self.track_struct(def);
        } else if let Some(def) = self.lookup.get_enum(name) {
            self.track_enum(def);
        }
    }
}

impl<'a> Iterator for TypeIterator<'a> {
    type Item = &'a dyn ToRustCode;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(latest_struct) = self.struct_queue.pop_front() {
            if let Some(parent) = latest_struct.parents.first() {
                self.track_by_name(parent.as_str());
            }

            let mut types_queue = latest_struct.fields.iter().map(|it| &it.signature).collect::<VecDeque<_>>();
            while let Some(type_) = types_queue.pop_front() {
                if type_.kind != FieldKind::Primitive {
                    self.track_by_name(type_.name.as_str());
                }

                for generic in type_.generics.iter() {
                    types_queue.push_back(generic);
                }
            }

            Some(latest_struct)
        } else if let Some(latest_enum) = self.enum_queue.pop_front() {
            Some(latest_enum)
        } else {
            None
        }
    }
}

impl ClassLookup {
    fn iter_compilation_units(&self) -> impl Iterator<Item=&dyn ToRustCode> {
        TypeIterator::new(&self)
    }
}

impl ToTokens for TypeSignature {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name_ident = format_ident!("{}", self.name);
        let name = if self.kind == FieldKind::Enum && self.name.ends_with("Flags") {
            quote!(::flagset::FlagSet<#name_ident>)
        } else {
            quote!(#name_ident)
        };

        let typed_stream = if self.generics.is_empty() {
            quote!(#name)
        } else {
            let generics = &self.generics;
            quote!(#name <#(#generics),*>)
        };

        let result = match self.is_pointer {
            true if self.name.starts_with("U") => { quote! { UObjectPointer<#typed_stream> } }
            true => { quote! { *mut #typed_stream } }
            false => { typed_stream }
        };

        tokens.append_all(result);
    }
}

impl ToTokens for FieldDefinition {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let name = format_ident!("{}", self.name);

        tokens.append_all(quote! {
            pub #name: #(self.signature)
        })
    }
}

impl ToRustCode for EnumDefinition {
    fn name(&self) -> &str {
        self.name.as_str()
    }

    fn package(&self) -> Option<&str> {
        self.package.as_ref().map(|it| it.as_str())
    }

    fn generate_code(&self, _context: &ClassLookup) -> TokenStream {
        let data_type = format_ident!("{}", self.kind.to_string());
        let options = self.options.iter().map(|(name, value)| {
            let name = format_ident!("{}", name);
            let value = match self.kind {
                EnumKind::U8 => (*value as u8).to_token_stream(),
                EnumKind::U16 => (*value as u16).to_token_stream(),
                EnumKind::U32 => (*value as u32).to_token_stream(),
                EnumKind::U64 => (*value).to_token_stream(),
            };

            quote! {
                #name = #value
            }
        });

        let name = format_ident!("{}", self.name);
        if self.name.ends_with("Flags") {
            quote! {
                ::flagset::flags! {
                    pub enum #name: #data_type {
                        #(#options),*
                    }
                }
            }
        } else {
            quote! {
                #[repr(#data_type)]
                #[derive(Debug, Clone, Copy)]
                pub enum #name {
                    #(#options),*
                }
            }
        }
    }

    fn generate_test(&self, _context: &ClassLookup) -> Option<TokenStream> {
        None
    }
}

impl StructDefinition {
    fn resolve_fields(&self, context: &ClassLookup) -> Vec<TokenStream> {
        let mut target = vec![];
        let mut offset = 0;

        let parent = self.parents.first();
        if let Some(parent) = parent {
            if let Some(parent_obj) = context.get_struct(parent) {
                offset = parent_obj.struct_size;
            } else {
                offset = self.fields.first().map(|it| it.offset).unwrap_or_default();
            }
            // Parent is added through #extend attribute
        }

        let mut counter = self.parents.len() * 100;

        for field in self.fields.iter() {
            if offset > field.offset {
                // Padding issue
                continue;
            }

            if offset < field.offset {
                let name = format_ident!("_padding_{}", counter);
                let size = field.offset - offset;
                target.push(quote!(pub #name: [u8; #size]));
                counter += 1;

                offset = field.offset;
            }

            if field.bit_offset.is_some() {
                // Not implemented
                continue;
            }

            let field_name = field.name.to_snake_case();
            let field_name = match field_name.as_str() {
                "" => format_ident!("{}", "_unknown_"),
                _ => {
                    if field_name.starts_with(char::is_numeric) {
                        format_ident!("_{}", field_name)
                    } else if parse_str::<Ident>(field_name.as_str()).is_err() {
                        format_ident!("{}_", field_name)
                    } else {
                        format_ident!("{}",field_name)
                    }
                }
            };

            offset = offset + field.size;
            let field_type = &field.signature;

            target.push(quote!(pub #field_name: #field_type));
        }


        if offset < self.struct_size {
            let name = format_ident!("_padding_{}", counter);
            let size = self.struct_size - offset;
            target.push(quote!(pub #name: [u8; #size]));
        }

        target
    }
}

impl ToRustCode for StructDefinition {
    fn name(&self) -> &str {
        self.name.as_str()
    }

    fn package(&self) -> Option<&str> {
        self.package.as_ref().map(|it| it.as_str())
    }

    fn generate_code(&self, context: &ClassLookup) -> TokenStream {
        let name = format_ident!("{}", self.name);
        let fields = self.resolve_fields(context);

        let extend_statement = self.parents.first().map(|parent| {
            let parent_ident = format_ident!("{}",  parent);
            quote! {
                #[extend(#parent_ident)]
            }
        });

        quote! {
            #[repr(C)]
            #extend_statement
            #[derive(Debug, Clone)]
            pub struct #name {
                #(#fields),*
            }
        }
    }

    fn generate_test(&self, _context: &ClassLookup) -> Option<TokenStream> {
        let test_name = format_ident!("test_{}", self.name);
        let name = format_ident!("{}", self.name);
        let size = self.struct_size;

        Some(quote! {
            #[test]
            fn #test_name() {
                assert_eq!(size_of::<#name>(), #size);
            }
        })
    }
}


pub fn generate_code(structs_path: &str, classes_path: &str, enums_path: &str, gobjects: &str, offsets_path: &str, exclusions: &[&str]) -> std::io::Result<TokenStream> {
    let manifest: Manifest = std::io::read_to_string(File::open(gobjects)?)?.parse().unwrap();
    let structs_dump: StructDump = StructDump::from_raw_json(File::open(structs_path)?)?;
    let classes_dump: StructDump = StructDump::from_raw_json(File::open(classes_path)?)?;
    let enums_dump: EnumDump = EnumDump::from_raw_json(File::open(enums_path)?)?;
    let offsets: OffsetData = serde_json::from_reader(File::open(offsets_path)?)?;

    let mut lut = ClassLookup::new(manifest, Some(Regex::new(r"^X21$").unwrap()));
    lut.add_struct_dump(classes_dump);
    lut.add_struct_dump(structs_dump);
    lut.add_enum_dump(enums_dump);


    let units = lut.iter_compilation_units()
        .filter(|it| !exclusions.contains(&it.name()))
        .collect::<Vec<_>>();

    let code = units.iter().map(|it| it.generate_code(&lut));
    let tests = units.iter().filter_map(|it| it.generate_test(&lut));
    let offset_constants = offsets.data.iter().map(|offset| {
        let ident = format_ident!("{}", offset.0);
        let value = offset.1;
        quote! {pub const #ident: usize = #value;}
    });

    let definitions = quote! {
        #(#code)*

        #[cfg(test)]
        mod tests {
            #![allow(non_snake_case)]
            use std::mem::size_of;
            use super::*;

            #(#tests)*
        }

        mod Offsets {
            #(#offset_constants)*
        }
    };

    Ok(definitions)
}


#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use rust_format::{Formatter, PrettyPlease};

    use crate::{EnumKind, FieldKind};

    use super::*;

    #[test]
    fn test_enum() {
        let def = EnumDefinition {
            kind: EnumKind::U16,
            name: "MyTestFlags".into(),
            package: None,
            options: vec![
                ("Option1".into(), 0),
                ("Option2".into(), 1),
            ],
        };

        let lookup = ClassLookup::new(Manifest { packages: HashSet::new(), structs: HashMap::new() }, None);

        let tokens = def.generate_code(&lookup);
        let actual = PrettyPlease::default().format_tokens(tokens).unwrap();
        let expected = PrettyPlease::default().format_tokens(quote! {
            ::flagset::flags! {
                pub enum MyTestFlags: u16 {
                    Option1 = 0u16,
                    Option2 = 1u16
                }
            }
        }).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_struct() {
        let def = StructDefinition {
            struct_size: 20,
            package: None,
            name: "MyTest".into(),
            parents: vec![],
            fields: vec![
                FieldDefinition::new("field_1".into(), 0, 1, 1, None, TypeSignature::new_simple("u8".into(), FieldKind::Primitive)),
                FieldDefinition::new("field_2".into(), 8, 8, 1, None, TypeSignature::new_pointer("u8".into(), FieldKind::Primitive)),
            ],
        };


        let lookup = ClassLookup::new(Manifest { packages: HashSet::new(), structs: HashMap::new() }, None);
        let tokens = def.generate_code(&lookup);

        let actual = PrettyPlease::default().format_tokens(tokens).unwrap();
        let expected = PrettyPlease::default().format_tokens(quote! {
            #[repr(C)]
            #[derive(Debug, Clone)]
            pub struct MyTest {
                pub field_1: u8,
                pub _padding_0: [u8; 7usize],
                pub field_2: *mut u8,
                pub _padding_1: [u8; 4usize]
            }
        }).unwrap();

        assert_eq!(actual, expected);
    }
}