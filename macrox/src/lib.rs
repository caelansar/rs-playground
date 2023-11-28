mod builder;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DataEnum, DeriveInput};

#[proc_macro_derive(Builder)]
pub fn derive_builder(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    builder::BuilderContext::from(input).render().into()
}

#[proc_macro_derive(EnumString)]
pub fn enum_string(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    let variants = match input.data {
        Data::Enum(DataEnum { variants, .. }) => variants,
        _ => panic!("EnumString only works on enums"),
    };

    let from_str_arms = variants.iter().map(|variant| {
        let variant_name = &variant.ident;
        let variant_str = variant_name.to_string();
        quote! { #variant_str => Ok(#name::#variant_name), }
    });

    let to_string_arms = variants.iter().map(|variant| {
        let variant_name = &variant.ident;
        quote! { #name::#variant_name => stringify!(#variant_name), }
    });

    let expanded = quote! {
        impl std::str::FromStr for #name {
            type Err = String;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    #( #from_str_arms )*
                    _ => Err(format!("Invalid value for {}: {}", stringify!(#name), s)),
                }
            }
        }

        impl std::string::ToString for #name {
            fn to_string(&self) -> String {
                match self {
                    #( #to_string_arms )*
                }.to_string()
            }
        }
    };

    TokenStream::from(expanded)
}
