use crate::components::ComponentsBehaviour;
use heck::{ToKebabCase as _, ToPascalCase as _};

inventory::collect!(GpuiFormShape);

#[derive(Debug)]
pub struct GpuiFormShape {
    pub struct_name: &'static str,
    pub components: &'static [FieldVariant],
}

impl GpuiFormShape {
    pub const fn new(struct_name: &'static str, components: &'static [FieldVariant]) -> Self {
        Self {
            struct_name,
            components,
        }
    }
}

#[derive(Debug)]
pub struct FieldVariant {
    pub field_name: &'static str,
    pub field_type: &'static str,
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
            optional,
            behaviour,
        }
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
