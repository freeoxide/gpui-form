use darling::FromAttributes;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Path, parse_macro_input};

#[derive(Debug, Default, FromAttributes)]
#[darling(attributes(gpui_form_custom))]
struct CustomComponentStateMeta {
    #[darling(default)]
    new: Option<Path>,
    #[darling(default)]
    component: Option<Path>,
}

fn parse_meta(attrs: &[syn::Attribute]) -> darling::Result<(Path, Path)> {
    let meta = CustomComponentStateMeta::from_attributes(attrs)?;
    let new_path = meta.new.unwrap_or_else(|| syn::parse_quote!(Self::new));
    let component = meta.component.ok_or_else(|| {
        darling::Error::custom(
            "CustomComponentState requires `#[gpui_form_custom(component = YourUiComponent)]`",
        )
    })?;
    Ok((new_path, component))
}

fn expand(input: DeriveInput) -> darling::Result<TokenStream> {
    let ident = &input.ident;
    let (new_path, component_path) = parse_meta(&input.attrs)?;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    Ok(quote! {
        impl #impl_generics gpui_form_component::custom::CustomComponentShape for #ident #ty_generics #where_clause {
            type State = Self;
            type Component = #component_path;

            fn new(
                window: &mut ::gpui::Window,
                cx: &mut ::gpui::Context<'_, Self::State>,
            ) -> Self::State {
                #new_path(window, cx)
            }

            fn component(state: &::gpui::Entity<Self::State>) -> Self::Component {
                <#component_path>::new(state)
            }
        }
    })
}

pub fn from(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match expand(input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.write_errors().into(),
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
            #[gpui_form_custom(component = TagsInput)]
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
        assert!(
            compact.contains("typeComponent=TagsInput"),
            "should set Component associated type"
        );
    }

    #[test]
    fn test_custom_component_state_explicit_new_path() {
        let input: DeriveInput = syn::parse2(quote! {
            #[derive(CustomComponentState)]
            #[gpui_form_custom(new = crate::state::build, component = crate::ui::TagsInput)]
            struct TagsState;
        })
        .unwrap();

        let expanded = expand(input).unwrap();
        let compact = compact_tokens(&expanded.to_string());

        assert!(
            compact.contains("crate::state::build(window,cx)"),
            "should use explicit new path from attribute"
        );
        assert!(
            compact.contains("typeComponent=crate::ui::TagsInput"),
            "should use explicit component type from attribute"
        );
    }

    #[test]
    fn test_custom_component_state_missing_component_errors() {
        let input: DeriveInput = syn::parse2(quote! {
            #[derive(CustomComponentState)]
            struct TagsState;
        })
        .unwrap();

        let result = expand(input);
        assert!(
            result.is_err(),
            "should error when component is not specified"
        );
    }
}
