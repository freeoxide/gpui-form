use gpui_form_core::registry::{FieldVariant, GpuiFormShape};
use proc_macro2::TokenStream;
use quote::quote;

use crate::implementations::ComponentIdentities as _;

use super::{FieldCodeGenerator, GeneratedSubscription};

pub struct NumberInputCodeGenerator;

impl FieldCodeGenerator for NumberInputCodeGenerator {
    fn generate_cx_new_call(
        &self,
        field: &FieldVariant,
        component: &GpuiFormShape,
    ) -> Option<TokenStream> {
        let form_components_struct_ident = component.struct_form_components_ident();
        let var_name_ident = field.field_ident_with_behaviour();
        let fn_name_ident = var_name_ident.clone();

        Some(quote! {
            let #var_name_ident =
                cx.new(|cx| #form_components_struct_ident::#fn_name_ident(window, cx));
        })
    }

    fn generate_field_initializers(
        &self,
        field: &FieldVariant,
        _component: &GpuiFormShape,
    ) -> Option<TokenStream> {
        let state_initializer = field.field_ident_with_behaviour();

        Some(quote! {
          #state_initializer,
        })
    }

    fn generate_render_child(
        &self,
        field: &FieldVariant,
        component: &GpuiFormShape,
    ) -> TokenStream {
        let ftl_label_ident = component.ftl_label_ident();
        let ftl_description_ident = component.ftl_description_ident();
        let field_name_pascal_case_ident = field.field_ident_pascal();
        let field_name_ident = field.field_ident();

        let component_gpui_type = field.behaviour.as_component_ident();

        let field_in_struct_name_ident = field.field_ident_with_behaviour();

        // Show description always, and error below it when present (hidden when empty)
        quote! {
            .child(
                field()
                    .label(#ftl_label_ident::#field_name_pascal_case_ident.to_fluent_string())
                    .description_fn({
                        let error = self.errors.#field_name_ident.clone();
                        let description = #ftl_description_ident::#field_name_pascal_case_ident.to_fluent_string();
                        move |_, _| {
                            div()
                                .flex()
                                .flex_col()
                                .gap_1()
                                .child(div().child(description.clone()))
                                .when(!error.is_empty(), |this| {
                                    this.child(
                                        div()
                                            .text_color(gpui::red())
                                            .child(error.clone())
                                    )
                                })
                        }
                    })
                    .child(#component_gpui_type::new(&self.fields.#field_in_struct_name_ident))
            )
        }
    }

    fn generate_focusable_cycle(
        &self,
        field: &FieldVariant,
        _component: &GpuiFormShape,
    ) -> Option<TokenStream> {
        let field_var_name_ident = field.field_ident_with_behaviour();
        let x = quote! {
          self.fields.#field_var_name_ident.focus_handle(cx),
        };
        Some(x)
    }

    fn generate_subscription(
        &self,
        field: &FieldVariant,
        component: &GpuiFormShape,
    ) -> Option<GeneratedSubscription> {
        let field_var_name_ident = field.field_ident_with_behaviour();

        let on_input_event_handler_fn_name = format!("on_{}_input_event", field.field_name);
        let on_input_event_handler_fn_name_ident =
            syn::parse_str::<syn::Ident>(&on_input_event_handler_fn_name).unwrap();
        let on_number_input_event_handler_fn_name =
            format!("on_{}_number_input_event", field.field_name);
        let on_number_input_event_handler_fn_name_ident =
            syn::parse_str::<syn::Ident>(&on_number_input_event_handler_fn_name).unwrap();

        let calls = vec![
            quote! { cx.subscribe_in(&#field_var_name_ident, window, Self::#on_input_event_handler_fn_name_ident) },
            quote! { cx.subscribe_in(&#field_var_name_ident, window, Self::#on_number_input_event_handler_fn_name_ident) },
        ];

        let mut handlers = vec![];

        let field_name_ident = field.field_ident();

        // Get the type to use for parsing (item_type if set, otherwise field_type)
        let parse_type_str = field.parse_type();
        let parse_type_path = syn::parse_str::<syn::Type>(parse_type_str).unwrap();

        // Get the type to use for final validation (always field_type)
        let validation_type_str = field.validation_type();
        let validation_type_path = syn::parse_str::<syn::Type>(validation_type_str).unwrap();

        // Get the FTL errors enum ident and field variant for this form
        let ftl_errors_ident = component.ftl_errors_ident();
        let field_name_pascal_case_ident = field.field_ident_pascal();

        // Generate different assignment logic based on whether we have two-phase validation
        let on_input_event_handler = if field.has_item_type() {
            // Two-phase validation: parse with item_type, use try_new to create nutype
            // On Change: parse with item_type, use try_new to create the nutype
            // On Blur: validate with field_type, show error if invalid
            quote! {
                fn #on_input_event_handler_fn_name_ident(
                    &mut self,
                    state: &Entity<InputState>,
                    event: &InputEvent,
                    _window: &mut Window,
                    _cx: &mut Context<Self>,
                ) {
                    match event {
                        InputEvent::Change => {
                            let text = state.read(_cx).value();
                            // Parse using the intermediate type (e.g., u8 for Age)
                            // Then try to create the nutype - if it fails, that's ok during typing
                            match text.parse::<#parse_type_path>() {
                                Ok(parsed_value) => {
                                    // Try to create the nutype from the parsed value
                                    if let Ok(validated) = #validation_type_path::try_new(parsed_value) {
                                        self.current_data.#field_name_ident = validated.into();
                                        self.errors.#field_name_ident.clear();
                                    }
                                    // If try_new fails, we keep the old value - error shown on blur
                                }
                                _ => {}
                            }
                        }
                        InputEvent::Blur => {
                            let text = state.read(_cx).value();
                            // On blur, try to parse and validate
                            match text.parse::<#parse_type_path>() {
                                Ok(parsed_value) => match #validation_type_path::try_new(parsed_value) {
                                    Ok(validated_value) => {
                                        self.current_data.#field_name_ident = validated_value.into();
                                        self.errors.#field_name_ident.clear();
                                    }
                                    Err(e) => {
                                        // Use FTL enum for localized error message
                                        self.errors.#field_name_ident = #ftl_errors_ident::#field_name_pascal_case_ident {
                                            value: format!("{:?}", e)
                                        }.to_fluent_string();
                                    }
                                }
                                Err(_) => {
                                    self.errors.#field_name_ident = #ftl_errors_ident::#field_name_pascal_case_ident {
                                        value: format!("Invalid {} format", stringify!(#parse_type_path))
                                    }.to_fluent_string();
                                }
                            }
                            _cx.notify();
                        }
                        _ => {}
                    }
                }
            }
        } else {
            // Single-phase: parse and validate with the same type
            quote! {
                fn #on_input_event_handler_fn_name_ident(
                    &mut self,
                    state: &Entity<InputState>,
                    event: &InputEvent,
                    _window: &mut Window,
                    _cx: &mut Context<Self>,
                ) {
                    match event {
                        InputEvent::Change => {
                            let text = state.read(_cx).value();
                            match text.parse::<#parse_type_path>() {
                                Ok(value) => {
                                    self.current_data.#field_name_ident = value.into();
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                }
            }
        };
        handlers.push(on_input_event_handler);

        // Generate increment/decrement logic
        // For two-phase validation (nutype): read from input text, parse, step, then validate with try_new
        // For single-phase: use the original arithmetic logic directly on the field
        let (decrement_logic, increment_logic) = if field.has_item_type() {
            // For nutype fields: read the current input text, parse it, apply step, then validate
            // This ensures we operate on what the user actually typed, not the last valid value
            let step_expr = if parse_type_str.starts_with('f') {
                // f32, f64
                quote! { 1 as #parse_type_path }
            } else if parse_type_str.starts_with('u') || parse_type_str.starts_with('i') {
                // i*, u* - saturating ops take same type
                quote! { 1 }
            } else {
                // External types (e.g., Decimal) - use From<i32>
                quote! { #parse_type_path::from(1) }
            };

            if parse_type_str.starts_with('f') {
                // Floats don't have saturating ops
                (
                    quote! {
                        let text = this.read(cx).value();
                        if let Ok(current_value) = text.parse::<#parse_type_path>() {
                            let new_value = current_value - #step_expr;
                            match #validation_type_path::try_new(new_value) {
                                Ok(validated) => {
                                    self.current_data.#field_name_ident = validated;
                                    self.errors.#field_name_ident.clear();
                                }
                                Err(e) => {
                                    self.errors.#field_name_ident = #ftl_errors_ident::#field_name_pascal_case_ident {
                                        value: format!("{:?}", e)
                                    }.to_fluent_string();
                                }
                            }
                            // Always update input with new value (even if validation failed)
                            this.update(cx, |input, cx| {
                                input.set_value(new_value.to_string(), window, cx);
                            });
                        }
                    },
                    quote! {
                        let text = this.read(cx).value();
                        if let Ok(current_value) = text.parse::<#parse_type_path>() {
                            let new_value = current_value + #step_expr;
                            match #validation_type_path::try_new(new_value) {
                                Ok(validated) => {
                                    self.current_data.#field_name_ident = validated;
                                    self.errors.#field_name_ident.clear();
                                }
                                Err(e) => {
                                    self.errors.#field_name_ident = #ftl_errors_ident::#field_name_pascal_case_ident {
                                        value: format!("{:?}", e)
                                    }.to_fluent_string();
                                }
                            }
                            // Always update input with new value (even if validation failed)
                            this.update(cx, |input, cx| {
                                input.set_value(new_value.to_string(), window, cx);
                            });
                        }
                    },
                )
            } else {
                // Use saturating ops after parsing from input text
                (
                    quote! {
                        let text = this.read(cx).value();
                        if let Ok(current_value) = text.parse::<#parse_type_path>() {
                            let new_value = current_value.saturating_sub(#step_expr);
                            match #validation_type_path::try_new(new_value) {
                                Ok(validated) => {
                                    self.current_data.#field_name_ident = validated;
                                    self.errors.#field_name_ident.clear();
                                }
                                Err(e) => {
                                    self.errors.#field_name_ident = #ftl_errors_ident::#field_name_pascal_case_ident {
                                        value: format!("{:?}", e)
                                    }.to_fluent_string();
                                }
                            }
                            // Always update input with new value (even if validation failed)
                            this.update(cx, |input, cx| {
                                input.set_value(new_value.to_string(), window, cx);
                            });
                        }
                    },
                    quote! {
                        let text = this.read(cx).value();
                        if let Ok(current_value) = text.parse::<#parse_type_path>() {
                            let new_value = current_value.saturating_add(#step_expr);
                            match #validation_type_path::try_new(new_value) {
                                Ok(validated) => {
                                    self.current_data.#field_name_ident = validated;
                                    self.errors.#field_name_ident.clear();
                                }
                                Err(e) => {
                                    self.errors.#field_name_ident = #ftl_errors_ident::#field_name_pascal_case_ident {
                                        value: format!("{:?}", e)
                                    }.to_fluent_string();
                                }
                            }
                            // Always update input with new value (even if validation failed)
                            this.update(cx, |input, cx| {
                                input.set_value(new_value.to_string(), window, cx);
                            });
                        }
                    },
                )
            }
        } else if field.field_type.starts_with('f') {
            // f32, f64
            (
                quote! {
                    let new_value = self.current_data.#field_name_ident - 1 as #parse_type_path;
                    self.current_data.#field_name_ident = new_value;
                },
                quote! {
                    let new_value = self.current_data.#field_name_ident + 1 as #parse_type_path;
                    self.current_data.#field_name_ident = new_value;
                },
            )
        } else if field.field_type.starts_with('u') || field.field_type.starts_with('i') {
            // i*, u*
            (
                quote! {
                    let new_value = self.current_data.#field_name_ident.saturating_sub(1);
                    self.current_data.#field_name_ident = new_value;
                },
                quote! {
                    let new_value = self.current_data.#field_name_ident.saturating_add(1);
                    self.current_data.#field_name_ident = new_value;
                },
            )
        } else {
            // External types (e.g., Decimal) - assume saturating operations with From<i32>
            (
                quote! {
                    let new_value = self.current_data.#field_name_ident.saturating_sub(#parse_type_path::from(1));
                    self.current_data.#field_name_ident = new_value;
                },
                quote! {
                    let new_value = self.current_data.#field_name_ident.saturating_add(#parse_type_path::from(1));
                    self.current_data.#field_name_ident = new_value;
                },
            )
        };

        // For nutype fields, the input update is handled inside the step logic
        // For non-nutype fields, we update the input after the step
        let on_number_input_event_handler = if field.has_item_type() {
            quote! {
                fn #on_number_input_event_handler_fn_name_ident(
                    &mut self,
                    this: &Entity<InputState>,
                    event: &NumberInputEvent,
                    window: &mut Window,
                    cx: &mut Context<Self>,
                ) {
                    match event {
                        NumberInputEvent::Step(step_action) => match step_action {
                            StepAction::Decrement => {
                                #decrement_logic
                                cx.notify();
                            }
                            StepAction::Increment => {
                                #increment_logic
                                cx.notify();
                            }
                        },
                    }
                }
            }
        } else {
            quote! {
                fn #on_number_input_event_handler_fn_name_ident(
                    &mut self,
                    this: &Entity<InputState>,
                    event: &NumberInputEvent,
                    window: &mut Window,
                    cx: &mut Context<Self>,
                ) {
                    match event {
                        NumberInputEvent::Step(step_action) => match step_action {
                            StepAction::Decrement => {
                                #decrement_logic
                                this.update(cx, |input, cx| {
                                    input.set_value(self.current_data.#field_name_ident.to_string(), window, cx);
                                });
                            }
                            StepAction::Increment => {
                                #increment_logic
                                this.update(cx, |input, cx| {
                                    input.set_value(self.current_data.#field_name_ident.to_string(), window, cx);
                                });
                            }
                        },
                    }
                }
            }
        };
        handlers.push(on_number_input_event_handler);

        Some(GeneratedSubscription { calls, handlers })
    }
}
