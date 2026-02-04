use crate::components::ComponentsBehaviour;
use heck::{ToKebabCase as _, ToPascalCase as _};

inventory::collect!(GpuiFormShape);

#[derive(Debug)]
pub struct GpuiFormShape {
    pub struct_name: &'static str,
    pub components: &'static [FieldVariant],
    /// The source file path where the struct with #[derive(GpuiForm)] is declared.
    /// This is the full path from file!() macro, useful for generating imports.
    pub source_path: &'static str,
    /// Whether the struct has koruma validation enabled at the struct level.
    pub koruma_enabled: bool,
}

impl GpuiFormShape {
    pub const fn new(
        struct_name: &'static str,
        components: &'static [FieldVariant],
        source_path: &'static str,
        koruma_enabled: bool,
    ) -> Self {
        Self {
            struct_name,
            components,
            source_path,
            koruma_enabled,
        }
    }

    pub fn has_validations(&self) -> bool {
        self.koruma_enabled
            && self
                .components
                .iter()
                .any(|field| !field.validations.is_empty())
    }

    /// Returns true if the struct has koruma validation enabled at the struct level.
    pub const fn has_koruma(&self) -> bool {
        self.koruma_enabled
    }
}

#[derive(Debug)]
pub struct FieldVariant {
    pub field_name: &'static str,
    pub field_type: &'static str,
    pub optional: bool,
    pub behaviour: ComponentsBehaviour,
    /// List of validation rule identifiers applied to this field (for diagnostics/rendering).
    pub validations: &'static [&'static str],
    /// Default value expression as a string, if one was specified.
    pub default_expr: Option<&'static str>,
}

impl FieldVariant {
    pub const fn new(
        field_name: &'static str,
        field_type: &'static str,
        optional: bool,
        behaviour: ComponentsBehaviour,
    ) -> Self {
        Self {
            field_name,
            field_type,
            optional,
            behaviour,
            validations: &[],
            default_expr: None,
        }
    }

    /// Attach a default value expression to this field metadata.
    pub const fn with_default(mut self, default_expr: &'static str) -> Self {
        self.default_expr = Some(default_expr);
        self
    }

    pub fn full_type(&self) -> syn::Type {
        let mut ty = syn::parse_str(self.field_type).unwrap();
        if self.optional {
            ty = syn::Type::Path(syn::TypePath {
                qself: None,
                path: syn::parse_str("Option").unwrap(),
            });
        }
        ty
    }

    pub fn behaviour_suffix(&self) -> String {
        self.behaviour.to_string()
    }

    pub fn field_ident(&self) -> syn::Ident {
        syn::parse_str(self.field_name).unwrap()
    }

    pub fn field_ident_pascal(&self) -> syn::Ident {
        syn::parse_str::<syn::Ident>(&self.field_name.to_pascal_case()).unwrap()
    }

    pub fn field_name_with_behaviour(&self) -> String {
        format!("{}_{}", self.field_name, self.behaviour_suffix())
    }

    pub fn field_ident_with_behaviour(&self) -> syn::Ident {
        syn::parse_str(&self.field_name_with_behaviour()).unwrap()
    }

    pub fn kebab_id(&self) -> String {
        self.field_name_with_behaviour().to_kebab_case()
    }

    /// Returns the validation rule identifiers attached to this field.
    pub fn validation_rules(&self) -> &'static [&'static str] {
        self.validations
    }

    /// Returns parsed validation rule idents as syn::Path values.
    pub fn validation_paths(&self) -> Vec<syn::Path> {
        self.validations
            .iter()
            .filter_map(|v| syn::parse_str::<syn::Path>(v).ok())
            .collect()
    }

    /// Returns the first validation rule as a syn::Path, if any.
    pub fn first_validation_path(&self) -> Option<syn::Path> {
        self.validations
            .iter()
            .find_map(|v| syn::parse_str::<syn::Path>(v).ok())
    }

    /// Attach validation rule identifiers to this field metadata.
    pub const fn with_validations(mut self, validations: &'static [&'static str]) -> Self {
        self.validations = validations;
        self
    }
}

pub use inventory;
