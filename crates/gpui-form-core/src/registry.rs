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
}

impl GpuiFormShape {
    pub const fn new(
        struct_name: &'static str,
        components: &'static [FieldVariant],
        source_path: &'static str,
    ) -> Self {
        Self {
            struct_name,
            components,
            source_path,
        }
    }
}

#[derive(Debug)]
pub struct FieldVariant {
    pub field_name: &'static str,
    pub field_type: &'static str,
    /// The intermediate type used for parsing during input (e.g., u32 for a nutype Age).
    /// If None, field_type is used for both parsing and final validation.
    pub item_type: Option<&'static str>,
    pub optional: bool,
    pub behaviour: ComponentsBehaviour,
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
            item_type: None,
            optional,
            behaviour,
        }
    }

    /// Create a new FieldVariant with an item type for two-phase validation.
    /// The item_type is used for intermediate parsing (e.g., u32),
    /// while field_type is used for final validation (e.g., Age nutype).
    pub const fn new_with_item(
        field_name: &'static str,
        field_type: &'static str,
        item_type: &'static str,
        optional: bool,
        behaviour: ComponentsBehaviour,
    ) -> Self {
        Self {
            field_name,
            field_type,
            item_type: Some(item_type),
            optional,
            behaviour,
        }
    }

    /// Returns true if this field uses two-phase validation (has a separate item type)
    pub const fn has_item_type(&self) -> bool {
        self.item_type.is_some()
    }

    /// Returns the type to use for parsing input values.
    /// If item_type is set, returns that; otherwise returns field_type.
    pub fn parse_type(&self) -> &'static str {
        self.item_type.unwrap_or(self.field_type)
    }

    /// Returns the type to use for final validation.
    /// This is always the field_type.
    pub fn validation_type(&self) -> &'static str {
        self.field_type
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
}

pub use inventory;
