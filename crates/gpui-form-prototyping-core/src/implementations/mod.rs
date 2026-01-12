pub mod checkbox;
pub mod date_picker;
pub mod input;
pub mod number_input;
pub mod select;
pub mod switch;
pub mod tuple_select;

use gpui_form_core::registry::{FieldVariant, GpuiFormShape};
use heck::ToSnakeCase as _;
use proc_macro2::TokenStream;

pub enum FieldGenerator {
    Input(input::InputCodeGenerator),
    NumberInput(number_input::NumberInputCodeGenerator),
    Checkbox(checkbox::CheckboxCodeGenerator),
    Switch(switch::SwitchCodeGenerator),
    Select(select::SelectCodeGenerator),
    TupleSelect(tuple_select::TupleSelectCodeGenerator),
    DatePicker(date_picker::DatePickerCodeGenerator),
}

impl FieldGenerator {
    pub fn as_generator(&self) -> &dyn FieldCodeGenerator {
        match self {
            FieldGenerator::Input(generator) => generator,
            FieldGenerator::NumberInput(generator) => generator,
            FieldGenerator::Checkbox(generator) => generator,
            FieldGenerator::Switch(generator) => generator,
            FieldGenerator::Select(generator) => generator,
            FieldGenerator::TupleSelect(generator) => generator,
            FieldGenerator::DatePicker(generator) => generator,
        }
    }
}

#[derive(Default)]
pub struct GeneratedSubscription {
    pub calls: Vec<TokenStream>,
    pub handlers: Vec<TokenStream>,
}

impl GeneratedSubscription {
    pub fn is_empty(&self) -> bool {
        self.calls.is_empty() && self.handlers.is_empty()
    }
}

pub trait FieldCodeGenerator {
    fn generate_cx_new_call(
        &self,
        field: &FieldVariant,
        component: &GpuiFormShape,
    ) -> Option<TokenStream>;

    fn generate_field_initializers(
        &self,
        field: &FieldVariant,
        component: &GpuiFormShape,
    ) -> Option<TokenStream>;

    fn generate_render_child(&self, field: &FieldVariant, component: &GpuiFormShape)
    -> TokenStream;

    fn generate_focusable_cycle(
        &self,
        field: &FieldVariant,
        component: &GpuiFormShape,
    ) -> Option<TokenStream>;

    fn generate_subscription(
        &self,
        field: &FieldVariant,
        component: &GpuiFormShape,
    ) -> Option<GeneratedSubscription>;

    fn generate_post_subscription_initialization(
        &self,
        _field: &FieldVariant,
        _component: &GpuiFormShape,
    ) -> Option<TokenStream> {
        None
    }
}

pub trait ComponentShape {
    fn cx_new_calls(&self) -> Option<TokenStream>;

    fn field_initializers(&self) -> Option<TokenStream>;

    fn child_elements(&self) -> TokenStream;

    fn focusable_cycle(&self) -> Option<TokenStream>;

    fn subscription_calls(&self) -> Option<TokenStream>;

    fn event_handlers(&self) -> Option<TokenStream>;

    fn post_subscription_initialization(&self) -> Option<TokenStream>;
}

pub trait ComponentIdentities {
    fn struct_name(&self) -> &'static str;
    fn struct_name_ident(&self) -> syn::Ident {
        syn::parse_str::<syn::Ident>(self.struct_name()).unwrap()
    }
    fn struct_form_ident(&self) -> syn::Ident {
        let str_repr = format!("{}Form", self.struct_name());
        syn::parse_str::<syn::Ident>(&str_repr).unwrap()
    }
    fn struct_form_components_ident(&self) -> syn::Ident {
        let str_repr = format!("{}FormComponents", self.struct_name());
        syn::parse_str::<syn::Ident>(&str_repr).unwrap()
    }
    fn struct_form_fields_ident(&self) -> syn::Ident {
        let str_repr = format!("{}FormFields", self.struct_name());
        syn::parse_str::<syn::Ident>(&str_repr).unwrap()
    }
    fn form_id_literal(&self) -> String {
        format!("{}-form", self.struct_name().to_snake_case())
    }
    fn ftl_label_ident(&self) -> syn::Ident {
        let str_repr = format!("{}LabelKvFtl", self.struct_name());
        syn::parse_str::<syn::Ident>(&str_repr).unwrap()
    }
    fn ftl_description_ident(&self) -> syn::Ident {
        let str_repr = format!("{}DescriptionKvFtl", self.struct_name());
        syn::parse_str::<syn::Ident>(&str_repr).unwrap()
    }
}

impl ComponentIdentities for FieldVariant {
    fn struct_name(&self) -> &'static str {
        self.field_type
    }
}

impl ComponentIdentities for GpuiFormShape {
    fn struct_name(&self) -> &'static str {
        self.struct_name
    }
}

use quote::quote;

/// Helper function to generate the label tokens for a field.
pub fn generate_label_tokens(
    field: &FieldVariant,
    _component: &GpuiFormShape,
) -> proc_macro2::TokenStream {
    #[cfg(feature = "fluent")]
    {
        let ftl_label_ident = _component.ftl_label_ident();
        let field_name_pascal_case_ident = field.field_ident_pascal();
        quote! { #ftl_label_ident::#field_name_pascal_case_ident.to_fluent_string() }
    }
    #[cfg(not(feature = "fluent"))]
    {
        use heck::ToTitleCase;
        let title = field.field_name.to_title_case();
        quote! { #title }
    }
}

/// Helper function to generate the description_fn tokens for a field.
/// Includes validation error display if validations are present.
pub fn generate_description_fn_tokens(
    field: &FieldVariant,
    _component: &GpuiFormShape,
) -> proc_macro2::TokenStream {
    let field_name_ident = field.field_ident();

    #[cfg(feature = "fluent")]
    let description_tokens = {
        let ftl_description_ident = _component.ftl_description_ident();
        let field_name_pascal_case_ident = field.field_ident_pascal();
        quote! { #ftl_description_ident::#field_name_pascal_case_ident.to_fluent_string() }
    };
    #[cfg(not(feature = "fluent"))]
    let description_tokens = {
        use heck::ToTitleCase;
        let title = field.field_name.to_title_case();
        quote! { #title }
    };

    let field_has_validations = !field.validations.is_empty();
    let error_tokens = if field_has_validations {
        #[cfg(feature = "fluent")]
        let conversion_tokens = quote! { v.to_fluent_string() };
        #[cfg(not(feature = "fluent"))]
        let conversion_tokens = quote! { v.to_string() };

        quote! {{
            validation_errors.as_ref().and_then(|e| {
                let errs = e.#field_name_ident().all();
                if errs.is_empty() {
                    None
                } else {
                    Some(
                        errs.iter()
                            .map(|v| #conversion_tokens)
                            .collect::<Vec<_>>()
                            .join("\n"),
                    )
                }
            })
        }}
    } else {
        quote! {{ None }}
    };
    let error_color_tokens = quote! { cx.theme().danger };

    if !field_has_validations {
        quote! {
            .description_fn({
                let description = #description_tokens;
                move |_, _| {
                    div()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(div().child(description.clone()))
                }
            })
        }
    } else {
        quote! {
            .description_fn({
                let description = #description_tokens;
                let error = #error_tokens;
                let error_color = #error_color_tokens;
                move |_, _| {
                    div()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(div().child(description.clone()))
                        .when(error.is_some(), |this| {
                            this.child(
                                div()
                                    .text_color(error_color)
                                    .child(error.clone().unwrap_or_default()),
                            )
                        })
                }
            })
        }
    }
}

