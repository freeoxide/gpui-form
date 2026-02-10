use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Expr, Path, parse_macro_input};

fn parse_new_path(attrs: &[syn::Attribute]) -> syn::Result<Path> {
    let mut new_path: Option<Path> = None;

    for attr in attrs {
        if !attr.path().is_ident("gpui_form_custom") {
            continue;
        }

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("new") {
                let value = meta.value()?;
                let expr: Expr = value.parse()?;
                let path = match expr {
                    Expr::Path(expr_path) => expr_path.path,
                    Expr::Group(group) => match *group.expr {
                        Expr::Path(expr_path) => expr_path.path,
                        _ => {
                            return Err(meta.error(
                                "expected path expression for `new`, e.g. `new = Self::new`",
                            ));
                        },
                    },
                    _ => {
                        return Err(meta
                            .error("expected path expression for `new`, e.g. `new = Self::new`"));
                    },
                };

                if new_path.is_some() {
                    return Err(meta.error("duplicate `new` option"));
                }
                new_path = Some(path);
                Ok(())
            } else {
                Err(meta.error("unsupported option, expected `new = <path>`"))
            }
        })?;
    }

    Ok(new_path.unwrap_or_else(|| syn::parse_quote!(Self::new)))
}

fn expand(input: DeriveInput) -> syn::Result<TokenStream> {
    let ident = &input.ident;
    let new_path = parse_new_path(&input.attrs)?;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    Ok(quote! {
        impl #impl_generics gpui_form_component::custom::CustomComponentShape for #ident #ty_generics #where_clause {
            type State = Self;

            fn new(
                window: &mut ::gpui::Window,
                cx: &mut ::gpui::Context<'_, Self::State>,
            ) -> Self::State {
                #new_path(window, cx)
            }
        }
    })
}

pub fn from(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match expand(input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[cfg(test)]
mod tests {
    use super::expand;
    use quote::quote;
    use syn::DeriveInput;

    fn compact_tokens(tokens: &str) -> String {
        tokens.chars().filter(|c| !c.is_whitespace()).collect()
    }

    #[test]
    fn test_custom_component_state_default_new_path() {
        let input: DeriveInput = syn::parse2(quote! {
            #[derive(CustomComponentState)]
            struct TagsState;
        })
        .unwrap();

        let expanded = expand(input).unwrap();
        let compact = compact_tokens(&expanded.to_string());

        assert!(
            compact.contains("implgpui_form_component::custom::CustomComponentShapeforTagsState"),
            "should implement CustomComponentShape for derived type"
        );
        assert!(
            compact.contains("Self::new(window,cx)"),
            "should default to Self::new constructor"
        );
    }

    #[test]
    fn test_custom_component_state_explicit_new_path() {
        let input: DeriveInput = syn::parse2(quote! {
            #[derive(CustomComponentState)]
            #[gpui_form_custom(new = crate::state::build)]
            struct TagsState;
        })
        .unwrap();

        let expanded = expand(input).unwrap();
        let compact = compact_tokens(&expanded.to_string());

        assert!(
            compact.contains("crate::state::build(window,cx)"),
            "should use explicit new path from attribute"
        );
    }
}
