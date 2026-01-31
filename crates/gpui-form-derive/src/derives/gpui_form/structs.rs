use darling::{FromField, FromMeta};
use gpui_form_core::components::Components;
use koruma_derive_core::ValidationInfo;
use proc_macro2::TokenStream;
use syn::{Expr, Ident, Type};

/// Information about a field for value holder generation.
pub struct FieldOptionality {
    pub field_name: Ident,
    pub original_type: Type,
    pub inner_type: Type,
    pub was_optional: bool,
    pub wrap_in_option: bool,
    pub validation: ValidationInfo,
    pub default_expr: Option<TokenStream>,
}

#[derive(Clone, Debug, Default, FromMeta)]
pub struct KorumaOptions {
    #[darling(default)]
    pub fluent: bool,
}

#[derive(Clone, Debug)]
pub struct KorumaField(pub KorumaOptions);

impl FromMeta for KorumaField {
    fn from_word() -> darling::Result<Self> {
        Ok(KorumaField(KorumaOptions::default()))
    }

    fn from_list(items: &[darling::ast::NestedMeta]) -> darling::Result<Self> {
        KorumaOptions::from_list(items).map(KorumaField)
    }
}

#[derive(Debug, FromField)]
#[darling(attributes(gpui_form))]
pub struct ComponentField {
    pub ident: Option<Ident>,
    pub ty: Type,
    #[darling(default)]
    pub component: Option<Components>,
    #[darling(default)]
    pub default: Option<Expr>,
    #[darling(default)]
    pub skip: bool,
}

impl ComponentField {
    pub fn skip(&self) -> bool {
        self.skip && self.component.is_none()
    }
}

#[derive(Debug, darling::FromDeriveInput)]
#[darling(attributes(gpui_form), supports(struct_named, struct_unit))]
pub struct ComponentStruct {
    pub ident: Ident,
    pub data: darling::ast::Data<(), ComponentField>,
    #[darling(default)]
    pub empty: bool,
    #[darling(default)]
    pub koruma: Option<KorumaField>,
}

pub struct ComponentFieldContent {
    pub field_structure_tokens: TokenStream,
    pub field_base_declarations_tokens: TokenStream,
    pub wrap_in_option: (String, bool),
}

pub struct GpuiFormOptions {
    pub generate_shape: bool,
}
