use anyhow::Context;
use heck::ToSnakeCase;
use itertools::Itertools;
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, ToTokens, TokenStreamExt};
use rayon::prelude::*;
use regex::Regex;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fs::File;
use std::io::Write;
use std::iter::once;
use std::path::Path;
use syn::{parse_str, Ident};

use crate::serialization::OffsetData;
use crate::{
    ClassLookup, EnumDefinition, EnumDump, EnumKind, FieldDefinition, FieldKind,
    FunctionDefinition, FunctionDump, Manifest, StructDefinition, StructDump, TypeSignature,
};

trait ToRustCode: Send + Sync {
    fn name(&self) -> &str;
    fn package(&self) -> Option<&str>;
    fn generate_code(&self, context: &ClassLookup) -> TokenStream;
    fn generate_test(&self, context: &ClassLookup) -> Option<TokenStream>;
    fn generate_impl(&self, context: &ClassLookup) -> Option<TokenStream>;
}

trait ToTokensWithContext {
    fn to_tokens(&self, context: &ClassLookup) -> TokenStream;
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

            let mut types_queue = latest_struct
                .fields
                .iter()
                .map(|it| &it.signature)
                .chain(latest_struct.functions.iter().flat_map(|fun| {
                    once(&fun.return_value).chain(fun.arguments.iter().map(|it| &it.type_))
                }))
                .collect::<VecDeque<_>>();
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
    fn iter_compilation_units(&self) -> impl Iterator<Item = &dyn ToRustCode> {
        TypeIterator::new(&self)
    }
}

#[derive(Clone, Debug)]
enum PointerHandling {
    Raw {
        mutable: bool,
    },
    Borrow {
        mutable: bool,
        lifetime: Option<String>,
    },
    Wrapper,
}

impl TypeSignature {
    fn is_uobject(&self) -> bool {
        self.name.starts_with(&['U', 'A'])
            && self.name[1..]
                .chars()
                .nth(0)
                .filter(char::is_ascii_uppercase)
                .is_some()
    }

    fn to_tokens_with(&self, pointer_handling: PointerHandling, context: &ClassLookup) -> TokenStream {
        let name_ident = format_ident!("{}", self.name);
        let package = match self.kind {
            FieldKind::Struct => { context.get_struct(self.name.as_str()).and_then(|it| it.package.clone())  }
            FieldKind::Class => { context.get_struct(self.name.as_str()).and_then(|it| it.package.clone()) }
            FieldKind::Primitive => { None }
            FieldKind::Enum => { context.get_enum(self.name.as_str()).and_then(|it| it.package.clone()) }
        }.map(|it | {
            let ident = format_ident!("{}", it);
            quote! { crate:: #ident:: }
        });

        let name = if self.kind == FieldKind::Enum && self.name.ends_with("Flags") {
            quote!(::flagset::FlagSet<#package #name_ident>)
        } else {
            quote!(#package #name_ident)
        };
        let typed_stream = if self.generics.is_empty() {
            quote!(#name)
        } else {
            let generics = self
                .generics
                .iter()
                .map(|it| it.to_tokens_with(pointer_handling.clone(), context));
            quote!(#name <#(#generics),*>)
        };

        match (self.is_pointer, pointer_handling) {
            (true, PointerHandling::Borrow { mutable, lifetime }) => {
                let lifetime_token =
                    lifetime.map(|it| syn::Lifetime::new(it.as_str(), Span::call_site()));

                if mutable {
                    quote! { &#lifetime_token mut #typed_stream }
                } else {
                    quote! { &#lifetime_token #typed_stream }
                }
            }
            (true, PointerHandling::Wrapper) => {
                if self.is_uobject() {
                    quote! { UObjectPointer<#typed_stream> }
                } else {
                    quote! { *mut #typed_stream }
                }
            }
            (true, PointerHandling::Raw { mutable }) => {
                if mutable {
                    quote! { *mut #typed_stream }
                } else {
                    quote! { *const #typed_stream }
                }
            }
            (false, _) => typed_stream,
        }
    }
}

impl ToTokensWithContext for TypeSignature {
    fn to_tokens(&self, context: &ClassLookup) -> TokenStream {
        self.to_tokens_with(PointerHandling::Wrapper, context)
    }
}

impl ToTokensWithContext for FieldDefinition {
    fn to_tokens(&self, context: &ClassLookup) -> TokenStream {
        let name = format_ident!("{}", self.name);
        let signature = self.signature.to_tokens(context);

        quote! {
            pub #name: #signature
        }
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

    fn generate_impl(&self, context: &ClassLookup) -> Option<TokenStream> {
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

            let field_name = as_identifier(field.name.as_str());

            offset = offset + field.size;
            let field_type = &field.signature.to_tokens(context);

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
            let parent_ident = format_ident!("{}", parent);
            let package = context.get_struct(parent.as_str())
                .and_then(|it| it.package.clone())
                .map(|it| format_ident!("{}", it))
                .map(|it| quote!(#it ::));

            quote! {
                #[extend(crate:: #package #parent_ident)]
            }
        });

        let derives = if self.parents.contains(&"UObject".to_string()) {
            vec!["Debug", "Clone", "HasClassObject"]
        } else {
            vec!["Debug", "Clone"]
        }
        .into_iter()
        .map(|it| format_ident!("{}", it));

        quote! {
            #[repr(C)]
            #extend_statement
            #[derive(#(#derives),*)]
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

    fn generate_impl(&self, context: &ClassLookup) -> Option<TokenStream> {
        let name = format_ident!("{}", self.name);
        let functions = self.functions.iter().map(|it| it.to_tokens(self, context));
        let bit_functions = self.fields.iter().filter(|it| it.bit_offset.is_some()).map(|field| {
            let identifier = as_identifier(field.name.as_str());
            let getter = format_ident!("bit_get_{}", identifier);
            let setter = format_ident!("bit_set_{}", identifier);
            let offset = field.offset;
            let bit_offset = field.bit_offset.unwrap();
   
            quote! {
                pub fn #getter (&self) -> bool {
                    let base_address = unsafe {
                        (std::ptr::addr_of!(*self) as *const u8).add(#offset)
                    };
                    
                    unsafe {
                        let mask = 0b00000001u8 << #bit_offset;
                        ((*base_address) & mask) != 0
                    }
                }

                pub fn #setter (&mut self, value: bool) {
                    let base_address = unsafe {
                        (std::ptr::addr_of!(*self) as *const u8).add(#offset) as *mut u8
                    };
        
                    unsafe {
                        let mask = 0b00000001u8 << #bit_offset;
                        if value {
                            *base_address |= mask;
                        } else {
                            *base_address &= !mask;
                        }
                    };
                }
            }
        });

        Some(quote! {
            impl #name {
                #(#functions)*
                
                #(#bit_functions)*
            }
        })
    }
}

impl FunctionDefinition {
    fn to_tokens(&self, owner: &StructDefinition, context: &ClassLookup) -> TokenStream {
        let fn_id = as_identifier(self.name.as_str());

        let return_value = if self.return_value.is_pointer || self.return_value.name != "void" {
            Some(&self.return_value)
        } else {
            None
        };

        let is_static = self.flags.contains("Static");
        let out_params = self
            .arguments
            .iter()
            .enumerate()
            .filter(|(_index, it)| it.is_out_param)
            .map(|(index, it)| (index, it))
            .collect::<Vec<_>>();

        let this_arg = if !is_static {
            Some(quote! { &self })
        } else {
            None
        };

        // Args in the function signature
        let signature_args = this_arg
            .into_iter()
            .chain(self.arguments.iter().enumerate().map(|(index, it)| {
                let id = as_identifier(it.name.as_str());
                let type_ = &it.type_;
                let type_stream = type_.to_tokens_with(PointerHandling::Borrow {
                    mutable: false,
                    lifetime: None,
                }, context);

                if it.is_out_param {
                    quote! {
                        #id: &mut #type_stream
                    }
                } else {
                    quote! {
                        #id: #type_stream
                    }
                }
            }));

        // The arguments within the struct
        let struct_args = self
            .arguments
            .iter()
            .enumerate()
            .map(|(index, it)| {
                if it.is_out_param {
                    let generic_name = format!("'b{}", index);
                    let type_stream = it.type_.to_tokens_with(PointerHandling::Borrow {
                        mutable: false,
                        lifetime: Some(generic_name),
                    }, context);
                    quote!(#type_stream)
                } else {
                    let type_stream = it.type_.to_tokens_with(PointerHandling::Borrow {
                        mutable: false,
                        lifetime: Some("'a".into()),
                    }, context);
                    quote!(#type_stream)
                }
            })
            .chain(return_value.cloned().map(|it| {
                let tokens = it.to_tokens(context);
                quote!(#tokens)
            }))
            .chain(once(quote! { std::marker::PhantomData<&'a u8> }));

        // Only the names for filling the struct
        let signature_arg_names = self
            .arguments
            .iter()
            .enumerate()
            .map(|(index, it)| {
                let id = as_identifier(it.name.as_str());
                if it.is_out_param {
                    quote!(unsafe { ::core::mem::zeroed() })
                } else {
                    quote!(#id)
                }
            })
            .chain(return_value.map(|it| quote!(unsafe { ::std::mem::zeroed() })))
            .chain(once(quote! { std::default::Default::default() }));

        let class_name = &owner.name[1..];
        let fn_name = &self.name;
        let unable_to_find_class = format!("Unable to find {}", &owner.name[1..]);
        let unable_to_find_function = format!("Unable to find {}::{}", &owner.name[1..], self.name);

        let return_type = return_value.map(|it| {
            let tokens = it.to_tokens(context);
            quote!(-> #tokens)
        });

        let return_statement = return_value.map(|it| {
            let size: syn::Index = self.arguments.len().into();
            quote!(parms.#size)
        });

        let class = if !is_static {
            quote!(&self.class.as_ref().expect(#unable_to_find_class))
        } else {
            quote!(UClass::find(#class_name).expect(#unable_to_find_class))
        };

        let this = if !is_static {
            quote!(self)
        } else {
            quote!(class.default_object.as_ref().expect("No default object"))
        };

        let call_statement = if self.flags.contains("Native") {
            quote! {
                let flags = func.function_flags;
                func.function_flags |= EFunctionFlags::Native;
                #this.process_event(func, &mut parms);
                func.function_flags = flags;
            }
        } else {
            quote! {
                #this.process_event(func, &mut parms);
            }
        };

        let swap_out_into = out_params.iter().map(|(index, arg)| {
            let accessor: syn::Index = (*index).into();
            let arg: syn::Ident = as_identifier(arg.name.as_str());

            quote! {
                std::mem::swap(&mut parms.#accessor, #arg);
            }
        });

        let swap_out_back = out_params.iter().map(|(index, arg)| {
            let accessor: syn::Index = (*index).into();
            let arg: syn::Ident = as_identifier(arg.name.as_str());

            quote! {
                std::mem::swap(#arg, &mut parms.#accessor);
            }
        });

        let generics = once("'a".to_string())
            .chain(
                out_params
                    .iter()
                    .filter(|(_, arg)| arg.type_.has_pointers())
                    .map(|(index, _)| format!("'b{}", index)),
            )
            .map(|name| syn::Lifetime::new(name.as_str(), Span::call_site()));

        quote! {
            pub fn #fn_id(#(#signature_args),*) #return_type {
                #[repr(C)]
                #[derive(Debug)]
                struct Args<#(#generics),*>(#(#struct_args),*);

                let class = #class;

                let func = class
                    .find_function_mut(#fn_name)
                    .expect(#unable_to_find_function);

                let mut parms = Args(
                    #(#signature_arg_names),*
                );

                #(#swap_out_into)*

                #call_statement

                #(#swap_out_back)*

                #return_statement
            }
        }
    }
}

pub fn generate_code<P: AsRef<Path>>(
    base_path: P,
    excluded_types: &[&str],
    package_filter: Option<Regex>,
) -> anyhow::Result<HashMap<Option<String>, String>> {
    let manifest: Manifest = std::fs::read_to_string(base_path.as_ref().join("GObjects-Dump.txt"))
        .context("ObjectsDump")?
        .parse()
        .context("Unable to parse manifest")?;
    let structs_dump: StructDump = StructDump::from_raw_json(
        File::open(base_path.as_ref().join("StructsInfo.json")).context("StructsInfo")?,
    )?;
    let classes_dump: StructDump = StructDump::from_raw_json(
        File::open(base_path.as_ref().join("ClassesInfo.json")).context("ClassesInfo")?,
    )?;
    let enums_dump: EnumDump = EnumDump::from_raw_json(
        File::open(base_path.as_ref().join("EnumsInfo.json")).context("EnumsInfo")?,
    )?;
    let offsets: OffsetData = serde_json::from_reader(
        File::open(base_path.as_ref().join("OffsetsInfo.json")).context("Offsets")?,
    )?;

    let mut lut = ClassLookup::new(manifest, package_filter);
    lut.add_struct_dump(classes_dump);
    lut.add_struct_dump(structs_dump);
    lut.add_enum_dump(enums_dump);

    lut.add_function_dump(FunctionDump::from_raw_json(
        File::open(base_path.as_ref().join("FunctionsInfo.json")).context("Functions")?,
    )?);

    let mut grouped: HashMap<_, Vec<&dyn ToRustCode>> = lut
        .iter_compilation_units()
        .filter(|it| !excluded_types.contains(&it.name()))
        .into_grouping_map_by(|it| it.package())
        .collect();

    grouped.entry(None).or_insert(Vec::new());

    let definitions: HashMap<_, _> = grouped.par_iter().map(|(package, structs)| {
        let code = structs.iter().map(|it| {
            let code = it.generate_code(&lut);
            let implem = it.generate_impl(&lut);

            quote! {
                #code

                #implem
            }
        });

        let tests = structs.iter().filter_map(|it| it.generate_test(&lut));

        let offsets = if package.is_none() {
            let offset_constants = offsets.data.iter().map(|offset| {
                let ident = format_ident!("{}", offset.0);
                let value = offset.1;
                quote! {pub const #ident: usize = #value;}
            });

            Some(
                quote! {
                    mod offsets {
                        #(#offset_constants)*
                    }
                }
            )
        } else {
            Some( quote! { use super::*; } )
        };


        let code = quote! {
            use manasdk_macros::{extend, HasClassObject};

            #offsets

            #(#code)*

            #[cfg(test)]
            mod tests {
                #![allow(non_snake_case)]
                use std::mem::size_of;
                use super::*;

                #(#tests)*
            }
        };

        (package.map(|it| it.to_string()), code.to_string())
    }).collect();

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
            options: vec![("Option1".into(), 0), ("Option2".into(), 1)],
        };

        let lookup = ClassLookup::new(
            Manifest {
                packages: HashSet::new(),
                structs: HashMap::new(),
            },
            None,
        );

        let tokens = def.generate_code(&lookup);
        let actual = PrettyPlease::default().format_tokens(tokens).unwrap();
        let expected = PrettyPlease::default()
            .format_tokens(quote! {
                ::flagset::flags! {
                    pub enum MyTestFlags: u16 {
                        Option1 = 0u16,
                        Option2 = 1u16
                    }
                }
            })
            .unwrap();

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
                FieldDefinition::new(
                    "field_1".into(),
                    0,
                    1,
                    1,
                    None,
                    TypeSignature::new_simple("u8".into(), FieldKind::Primitive),
                ),
                FieldDefinition::new(
                    "field_2".into(),
                    8,
                    8,
                    1,
                    None,
                    TypeSignature::new_pointer("u8".into(), FieldKind::Primitive),
                ),
            ],
            functions: vec![],
        };

        let lookup = ClassLookup::new(
            Manifest {
                packages: HashSet::new(),
                structs: HashMap::new(),
            },
            None,
        );
        let tokens = def.generate_code(&lookup);

        let actual = PrettyPlease::default().format_tokens(tokens).unwrap();
        let expected = PrettyPlease::default()
            .format_tokens(quote! {
                #[repr(C)]
                #[derive(Debug, Clone)]
                pub struct MyTest {
                    pub field_1: u8,
                    pub _padding_0: [u8; 7usize],
                    pub field_2: *mut u8,
                    pub _padding_1: [u8; 4usize]
                }
            })
            .unwrap();

        assert_eq!(actual, expected);
    }
}

fn as_identifier(name: &str) -> Ident {
    let field_name = name.to_snake_case();
    match field_name.as_str() {
        "" => format_ident!("{}", "_unknown_"),
        _ => {
            if field_name.starts_with(char::is_numeric) {
                format_ident!("_{}", field_name)
            } else if parse_str::<Ident>(field_name.as_str()).is_err() {
                format_ident!("{}_", field_name)
            } else {
                format_ident!("{}", field_name)
            }
        }
    }
}
