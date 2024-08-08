use proc_macro::TokenStream;

use quote::quote;
use syn::{Field, Fields, ItemStruct, parse_macro_input, parse_quote};

#[proc_macro_attribute]
pub fn extend(args: TokenStream, input: TokenStream) -> TokenStream {
    let parent = parse_macro_input!(args as syn::Ident);
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

        impl ::std::ops::Deref for #child_ident {
            type Target = #parent;

            fn deref(&self) -> &Self::Target {
                unsafe {
                    std::mem::transmute(self)
                }
            }
        }

       impl ::std::ops::DerefMut for #child_ident {
            fn deref_mut(&mut self) -> &mut Self::Target {
                unsafe {
                    std::mem::transmute(self)
                }
            }
        }
    };

    result.into()
}