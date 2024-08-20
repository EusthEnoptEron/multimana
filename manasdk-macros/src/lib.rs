use proc_macro::TokenStream;

use quote::{quote, ToTokens};
use syn::{Data, DeriveInput, Fields, ItemStruct, parse_macro_input, parse_quote, Type, TypePath};

#[proc_macro_derive(HasClassObject)]
pub fn has_class_object_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name_as_string = input.ident.to_string();
    let name = input.ident;
    
    let simple_name = &name_as_string.as_str()[1..];

    let expanded = quote! {
        impl crate::HasClassObject for #name {
            fn static_class() -> &'static UClass {
                UClass::find(#simple_name).unwrap()
            }
        }
    };

    expanded.into()
}


#[proc_macro_attribute]
pub fn extend(args: TokenStream, input: TokenStream) -> TokenStream {
    let parent = parse_macro_input!(args as TypePath);
    let mut child = parse_macro_input!(input as ItemStruct);

    match &mut child.fields {
        Fields::Named(named) => {
            named.named.insert(0, parse_quote! {
                __base: #parent
            })
        }
        Fields::Unnamed(unnamed) => {
            unnamed.unnamed.insert(0, parse_quote! {
                #parent
            })
        }
        Fields::Unit => {
            panic!("Cannot extend a unit type!");
        }
    };

    let child_ident = &child.ident;

    let result = quote! {
        #child

        impl ::core::ops::Deref for #child_ident {
            type Target = #parent;

            fn deref(&self) -> &Self::Target {
                &self.__base
            }
        }

        impl ::core::ops::DerefMut for #child_ident {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.__base
            }
        }
        
        impl<T> ::core::convert::AsRef<T> for #child_ident
            where T: ?Sized,
            <#child_ident as ::core::ops::Deref>::Target: ::core::convert::AsRef<T>,
            {
            fn as_ref(&self) -> &T {
                 ::core::ops::Deref::deref(self).as_ref()
            }
        }
        
        impl<T> ::core::convert::AsMut<T> for #child_ident
        where
            <#child_ident as ::core::ops::Deref>::Target: ::core::convert::AsMut<T>,
        {
            fn as_mut(&mut self) -> &mut T {
                 ::core::ops::DerefMut::deref_mut(self).as_mut()
            }
        }
        
    };

    result.into()
}