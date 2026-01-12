use darling::FromDeriveInput;
use proc_macro::TokenStream;
use quote::quote;
use syn::DeriveInput;

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

    let title_token = if args.fluent {
        quote! {
        use es_fluent::ToFluentString as _;
        self.to_fluent_string().into()
        }
    } else {
        quote! { self.to_string().into() }
    };

    let expanded = quote! {
        impl gpui_component::select::SelectItem for #item_ident {
            type Value = Self;

            fn title(&self) -> gpui::SharedString {
                #title_token
            }

            fn value(&self) -> &Self::Value {
                self
            }
        }
    };

    expanded.into()
}
