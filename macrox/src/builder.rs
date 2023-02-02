use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{
    Data, DataStruct, DeriveInput, Fields, FieldsNamed, GenericArgument, Path, Type, TypePath,
};

struct Field {
    name: Ident,
    ty: Type,
    optional: bool,
}

pub struct BuilderContext {
    name: Ident,
    fields: Vec<Field>,
}

impl From<syn::Field> for Field {
    fn from(f: syn::Field) -> Self {
        let (optional, ty) = get_option_inner(&f.ty);
        Self {
            name: f.ident.unwrap(),
            optional,
            ty: ty.to_owned(),
        }
    }
}

impl From<DeriveInput> for BuilderContext {
    fn from(input: DeriveInput) -> Self {
        let name = input.ident;

        let fields = if let Data::Struct(DataStruct {
            fields: Fields::Named(FieldsNamed { named, .. }),
            ..
        }) = input.data
        {
            named
        } else {
            panic!("Unsupported data type");
        };

        let fds = fields.into_iter().map(Field::from).collect();
        Self { name, fields: fds }
    }
}

impl BuilderContext {
    pub fn render(&self) -> TokenStream {
        let name = &self.name;
        let builder_name = Ident::new(&format!("{}Builder", name), name.span());

        let optionized_fields = self.gen_optionized_fields();
        let methods = self.gen_methods();
        let assigns = self.gen_assigns();

        quote! {
            #[derive(Debug, Default)]
            struct #builder_name {
                #(#optionized_fields,)*
            }

            impl #builder_name {
                #(#methods)*

                pub fn build(mut self) -> Result<#name, &'static str> {
                    Ok(#name {
                        #(#assigns,)*
                    })
                }
            }

            impl #name {
                fn builder() -> #builder_name {
                    Default::default()
                }
            }
        }
    }

    fn gen_optionized_fields(&self) -> Vec<TokenStream> {
        self.fields
            .iter()
            .map(|Field { name, ty, .. }| quote! { #name: std::option::Option<#ty> })
            .collect()
    }

    fn gen_methods(&self) -> Vec<TokenStream> {
        self.fields
            .iter()
            .map(|Field { name, ty, .. }| {
                let method_name = Ident::new(&format!("with_{}", name), name.span());
                quote! {
                    pub fn #method_name(mut self, v: impl Into<#ty>) -> Self {
                        self.#name = Some(v.into());
                        self
                    }
                }
            })
            .collect()
    }

    fn gen_assigns(&self) -> Vec<TokenStream> {
        self.fields
            .iter()
            .map(|Field { name, optional, .. }| {
                if *optional {
                    quote! {
                        #name: self.#name.take()
                    }
                } else {
                    quote! {
                        #name: self.#name.take().ok_or(concat!(stringify!(#name), " needs to be set!"))?
                    }
                }
            })
            .collect()
    }
}

fn get_option_inner(ty: &Type) -> (bool, &Type) {
    if let Type::Path(TypePath {
        path: Path { segments, .. },
        ..
    }) = ty
    {
        if let Some(v) = segments.iter().next() {
            if v.ident == "Option" {
                let t = match &v.arguments {
                    syn::PathArguments::AngleBracketed(a) => match a.args.iter().next() {
                        Some(GenericArgument::Type(t)) => t,
                        _ => panic!("Unsupported data type"),
                    },
                    _ => panic!("Unsupported data type"),
                };
                return (true, t);
            }
        }
    }
    (false, ty)
}
