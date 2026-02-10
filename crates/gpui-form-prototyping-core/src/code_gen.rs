use gpui_form_core::components::ComponentsBehaviour;
use gpui_form_core::registry::GpuiFormShape;
use proc_macro2::TokenStream;
use quote::quote;

use super::implementations::{
    ComponentShape, FieldGenerator, checkbox::CheckboxCodeGenerator, custom::CustomCodeGenerator,
    date_picker::DatePickerCodeGenerator, infinite_select::InfiniteSelectCodeGenerator,
    input::InputCodeGenerator, number_input::NumberInputCodeGenerator, select::SelectCodeGenerator,
    switch::SwitchCodeGenerator,
};

fn field_generator(behaviour: &ComponentsBehaviour) -> FieldGenerator {
    match behaviour {
        ComponentsBehaviour::Input => FieldGenerator::Input(InputCodeGenerator),
        ComponentsBehaviour::NumberInput => FieldGenerator::NumberInput(NumberInputCodeGenerator),
        ComponentsBehaviour::Checkbox => FieldGenerator::Checkbox(CheckboxCodeGenerator),
        ComponentsBehaviour::Switch => FieldGenerator::Switch(SwitchCodeGenerator),
        ComponentsBehaviour::Select(_) => FieldGenerator::Select(SelectCodeGenerator),
        ComponentsBehaviour::InfiniteSelect(_) => {
            FieldGenerator::InfiniteSelect(InfiniteSelectCodeGenerator)
        },
        ComponentsBehaviour::Custom => FieldGenerator::Custom(CustomCodeGenerator),
        ComponentsBehaviour::DatePicker => FieldGenerator::DatePicker(DatePickerCodeGenerator),
    }
}

pub struct FormShapeAdapter<'a> {
    pub shape_data: &'a GpuiFormShape,
}

impl<'a> FormShapeAdapter<'a> {
    pub fn new(shape_data: &'a GpuiFormShape) -> Self {
        Self { shape_data }
    }
}

impl<'a> ComponentShape for FormShapeAdapter<'a> {
    fn cx_new_calls(&self) -> Option<TokenStream> {
        let x: proc_macro2::TokenStream = self
            .shape_data
            .components
            .iter()
            .filter_map(|field| {
                let generator = field_generator(&field.behaviour);
                generator
                    .as_generator()
                    .generate_cx_new_call(field, self.shape_data)
            })
            .collect();

        if x.is_empty() { None } else { Some(x) }
    }

    fn field_initializers(&self) -> Option<TokenStream> {
        let x: proc_macro2::TokenStream = self
            .shape_data
            .components
            .iter()
            .filter_map(|field| {
                let generator = field_generator(&field.behaviour);
                generator
                    .as_generator()
                    .generate_field_initializers(field, self.shape_data)
            })
            .collect();

        if x.is_empty() { None } else { Some(x) }
    }

    fn child_elements(&self) -> TokenStream {
        self.shape_data
            .components
            .iter()
            .map(|field| {
                let generator = field_generator(&field.behaviour);
                generator
                    .as_generator()
                    .generate_render_child(field, self.shape_data)
            })
            .collect()
    }

    fn focusable_cycle(&self) -> Option<proc_macro2::TokenStream> {
        let x: proc_macro2::TokenStream = self
            .shape_data
            .components
            .iter()
            .filter(|field| field.behaviour.focusable())
            .filter_map(|field| {
                let generator = field_generator(&field.behaviour);
                generator
                    .as_generator()
                    .generate_focusable_cycle(field, self.shape_data)
            })
            .collect();

        if x.is_empty() { None } else { Some(x) }
    }

    fn subscription_calls(&self) -> Option<proc_macro2::TokenStream> {
        let calls: Vec<TokenStream> = self
            .shape_data
            .components
            .iter()
            .filter(|field| field.behaviour.subscribable())
            .filter_map(|field| {
                let generator = field_generator(&field.behaviour);
                generator
                    .as_generator()
                    .generate_subscription(field, self.shape_data)
            })
            .flat_map(|sub| sub.calls)
            .collect();

        if calls.is_empty() {
            None
        } else {
            Some(quote! {
                let mut _subscriptions = vec![#(#calls),*];
            })
        }
    }

    fn event_handlers(&self) -> Option<proc_macro2::TokenStream> {
        let handlers: Vec<TokenStream> = self
            .shape_data
            .components
            .iter()
            .filter(|field| field.behaviour.subscribable())
            .filter_map(|field| {
                let generator = field_generator(&field.behaviour);
                generator
                    .as_generator()
                    .generate_subscription(field, self.shape_data)
            })
            .flat_map(|sub| sub.handlers)
            .collect();

        if handlers.is_empty() {
            None
        } else {
            Some(quote! {
                #(#handlers)*
            })
        }
    }

    fn post_subscription_initialization(&self) -> Option<proc_macro2::TokenStream> {
        let x: proc_macro2::TokenStream = self
            .shape_data
            .components
            .iter()
            .filter_map(|field| {
                let generator = field_generator(&field.behaviour);
                generator
                    .as_generator()
                    .generate_post_subscription_initialization(field, self.shape_data)
            })
            .collect();

        if x.is_empty() { None } else { Some(x) }
    }
}
