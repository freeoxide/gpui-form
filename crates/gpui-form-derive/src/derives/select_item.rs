use darling::FromDeriveInput;
use heck::ToSnakeCase as _;
use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields};

#[derive(FromDeriveInput)]
#[darling(supports(enum_any), attributes(select_item))]
struct SelectItemArgs {
    ident: syn::Ident,
    #[darling(default)]
    fluent: bool,
}

pub fn from(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as DeriveInput);

    let args = match SelectItemArgs::from_derive_input(&input) {
        Ok(args) => args,
        Err(err) => return err.write_errors().into(),
    };

    let item_ident = &args.ident;
    let fallback_title_token = match &input.data {
        Data::Enum(data) => {
            let arms = data.variants.iter().map(|variant| {
                let ident = &variant.ident;
                let title = ident.to_string();
                let pattern = match &variant.fields {
                    Fields::Named(_) => quote! { Self::#ident { .. } },
                    Fields::Unnamed(_) => quote! { Self::#ident(..) },
                    Fields::Unit => quote! { Self::#ident },
                };

                quote! { #pattern => #title.to_string(), }
            });

            quote! {
                match self {
                    #(#arms)*
                }
            }
        },
        _ => quote! { stringify!(#item_ident).to_string() },
    };

    let title_token = match (&input.data, args.fluent) {
        (Data::Enum(data), true) => {
            let enum_key = item_ident.to_string().to_snake_case();
            let key_arms = data.variants.iter().map(|variant| {
                let ident = &variant.ident;
                let key = format!("{enum_key}.{}", ident.to_string().to_snake_case());
                let pattern = match &variant.fields {
                    Fields::Named(_) => quote! { Self::#ident { .. } },
                    Fields::Unnamed(_) => quote! { Self::#ident(..) },
                    Fields::Unit => quote! { Self::#ident },
                };

                quote! { #pattern => #key, }
            });

            quote! {
                {
                    let key: &'static str = match self {
                        #(#key_arms)*
                    };
                    let translated = ::rust_i18n::t!(key);
                    if &*translated == key {
                        #fallback_title_token.into()
                    } else {
                        translated.into_owned().into()
                    }
                }
            }
        },
        _ => quote! { self.to_string().into() },
    };

    let expanded = quote! {
        impl ::gpui_kit::component::select::SelectItem for #item_ident {
            type Value = Self;

            fn title(&self) -> ::gpui::SharedString {
                #title_token
            }

            fn value(&self) -> &Self::Value {
                self
            }
        }
    };

    expanded.into()
}
