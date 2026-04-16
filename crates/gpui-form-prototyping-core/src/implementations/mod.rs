pub mod checkbox;
pub mod custom;
pub mod date_picker;
pub mod infinite_select;
pub mod input;
pub mod number_input;
pub mod select;
pub mod switch;

use gpui_form_schema::registry::{FieldVariant, GpuiFormShape};
use heck::{ToPascalCase as _, ToSnakeCase as _};
use proc_macro2::TokenStream;
use quote::quote;

use crate::imports::ImportItem;

pub enum FieldGenerator {
    Input(input::InputCodeGenerator),
    NumberInput(number_input::NumberInputCodeGenerator),
    Checkbox(checkbox::CheckboxCodeGenerator),
    Switch(switch::SwitchCodeGenerator),
    Select(select::SelectCodeGenerator),
    InfiniteSelect(infinite_select::InfiniteSelectCodeGenerator),
    Custom(custom::CustomCodeGenerator),
    DatePicker(date_picker::DatePickerCodeGenerator),
}

impl FieldGenerator {
    pub fn as_generator(&self) -> &dyn FieldCodeGenerator {
        match self {
            Self::Input(generator) => generator,
            Self::NumberInput(generator) => generator,
            Self::Checkbox(generator) => generator,
            Self::Switch(generator) => generator,
            Self::Select(generator) => generator,
            Self::InfiniteSelect(generator) => generator,
            Self::Custom(generator) => generator,
            Self::DatePicker(generator) => generator,
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
    fn generate_imports(&self, _field: &FieldVariant) -> Vec<ImportItem> {
        vec![]
    }

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

pub trait ShapeIdentities {
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
        let str_repr = format!("{}LabelVariants", self.struct_name());
        syn::parse_str::<syn::Ident>(&str_repr).unwrap()
    }

    fn ftl_description_ident(&self) -> syn::Ident {
        let str_repr = format!("{}DescriptionVariants", self.struct_name());
        syn::parse_str::<syn::Ident>(&str_repr).unwrap()
    }
}

impl ShapeIdentities for GpuiFormShape {
    fn struct_name(&self) -> &'static str {
        self.struct_name
    }
}

pub trait FieldVariantExt {
    fn field_ident(&self) -> syn::Ident;
    fn field_ident_pascal(&self) -> syn::Ident;
    fn field_ident_with_behaviour(&self) -> syn::Ident;
    fn value_type(&self) -> syn::Type;
    fn component_ident(&self) -> TokenStream;
}

impl FieldVariantExt for FieldVariant {
    fn field_ident(&self) -> syn::Ident {
        syn::parse_str(self.field_name).unwrap()
    }

    fn field_ident_pascal(&self) -> syn::Ident {
        syn::parse_str::<syn::Ident>(&self.field_name_pascal()).unwrap()
    }

    fn field_ident_with_behaviour(&self) -> syn::Ident {
        syn::parse_str(&self.field_name_with_behaviour()).unwrap()
    }

    fn value_type(&self) -> syn::Type {
        syn::parse_str::<syn::Type>(self.value_type).expect("field value_type should be valid Rust")
    }

    fn component_ident(&self) -> TokenStream {
        let ident = syn::parse_str::<syn::Ident>(&self.behaviour.component_name().to_pascal_case())
            .unwrap();
        quote! { #ident }
    }
}

pub fn generate_entity_creation(field: &FieldVariant, component: &GpuiFormShape) -> TokenStream {
    let form_components_struct_ident = component.struct_form_components_ident();
    let var_name_ident = field.field_ident_with_behaviour();
    let fn_name_ident = var_name_ident.clone();

    quote! {
        let #var_name_ident =
            cx.new(|cx| #form_components_struct_ident::#fn_name_ident(window, cx));
    }
}

pub fn generate_entity_field_initializer(field: &FieldVariant) -> TokenStream {
    let field_var_name_ident = field.field_ident_with_behaviour();
    quote! { #field_var_name_ident, }
}

pub fn generate_entity_focus(field: &FieldVariant) -> TokenStream {
    let field_var_name_ident = field.field_ident_with_behaviour();
    quote! {
        self.fields.#field_var_name_ident.focus_handle(cx),
    }
}

pub fn generate_text_value_prefill(field: &FieldVariant) -> TokenStream {
    let field_var_name_ident = field.field_ident_with_behaviour();
    let field_name_ident = field.field_ident();

    quote! {
        if let Some(value) = current_data.#field_name_ident.as_ref() {
            #field_var_name_ident.update(cx, |state, cx| {
                state.set_value(value.to_string(), window, cx);
            });
        }
    }
}

pub fn render_standard_field(
    field: &FieldVariant,
    component: &GpuiFormShape,
    child_tokens: TokenStream,
) -> TokenStream {
    let description_fn_tokens = generate_description_fn_tokens(field, component);
    let label_tokens = generate_label_tokens(field, component);

    quote! {
        .child(
            field()
                .label(#label_tokens)
                #description_fn_tokens
                .child(#child_tokens)
        )
    }
}

pub fn render_component_entity_field(
    field: &FieldVariant,
    component: &GpuiFormShape,
) -> TokenStream {
    let component_gpui_type = field.component_ident();
    let field_in_struct_name_ident = field.field_ident_with_behaviour();

    render_standard_field(
        field,
        component,
        quote! { #component_gpui_type::new(&self.fields.#field_in_struct_name_ident) },
    )
}

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

pub fn generate_description_fn_tokens(
    field: &FieldVariant,
    component: &GpuiFormShape,
) -> proc_macro2::TokenStream {
    let field_name_ident = field.field_ident();

    #[cfg(feature = "fluent")]
    let description_tokens = {
        let ftl_description_ident = component.ftl_description_ident();
        let field_name_pascal_case_ident = field.field_ident_pascal();
        quote! { #ftl_description_ident::#field_name_pascal_case_ident.to_fluent_string() }
    };
    #[cfg(not(feature = "fluent"))]
    let description_tokens = {
        use heck::ToTitleCase;
        let title = field.field_name.to_title_case();
        quote! { #title }
    };

    let field_has_validations = !field.validation_rules().is_empty() && component.has_koruma();
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
