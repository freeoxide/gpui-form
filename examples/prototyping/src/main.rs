#![allow(unused)]

use gpui_form::core::registry::GpuiFormShape;
use gpui_form_prototyping_core::{
    code_gen::FormShapeAdapter,
    implementations::{ComponentIdentities as _, ComponentShape as _},
};
use heck::{ToSnakeCase as _, ToUpperCamelCase as _};

use quote::{format_ident, quote};
use std::{collections::BTreeSet, fs, path::Path};

// import targetted lib to get inventory registrations
#[allow(unused_imports)]
use some_lib::*;

struct LayoutIdentities {
    struct_name_str: &'static str,
    context_str: String,
    struct_name_ident: syn::Ident,
    struct_name_uw_ident: syn::Ident,
    struct_name_form_ident: syn::Ident,
    struct_name_form_fields_ident: syn::Ident,
    struct_name_form_errors_ident: syn::Ident,
    struct_name_form_errors_ftl_ident: syn::Ident,
    form_id_literal: String,
    struct_name_path_qualifier: syn::Ident,
}

impl LayoutIdentities {
    fn new(shape: &GpuiFormShape) -> Self {
        let struct_name_str = shape.struct_name;
        let context_str = format!("{}Form", struct_name_str);
        let struct_name_ident = shape.struct_name_ident();
        let struct_name_uw_ident = format_ident!("{}FormValueHolder", struct_name_ident);
        let struct_name_form_ident = shape.struct_form_ident();
        let struct_name_form_fields_ident = shape.struct_form_fields_ident();
        let struct_name_form_errors_ident = shape.struct_form_errors_ident();
        let struct_name_form_errors_ftl_ident = shape.ftl_errors_ident();
        let form_id_literal = shape.form_id_literal();
        let struct_name_path_qualifier =
            syn::parse_str::<syn::Ident>(&shape.struct_name.to_snake_case()).unwrap();

        Self {
            struct_name_str,
            context_str,
            struct_name_ident,
            struct_name_uw_ident,
            struct_name_form_ident,
            struct_name_form_fields_ident,
            struct_name_form_errors_ident,
            struct_name_form_errors_ftl_ident,
            form_id_literal,
            struct_name_path_qualifier,
        }
    }
}

fn main() {
    let output_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("output");
    fs::create_dir_all(&output_dir).expect("Failed to create output directory");
    println!("Generating forms in: {}", output_dir.display());

    let mut modules: BTreeSet<String> = BTreeSet::new();

    for struct_info in inventory::iter::<GpuiFormShape>() {
        println!("Thing : {:?}", struct_info);

        let syn_file = layout(struct_info);
        let file_stem = struct_info.struct_name.to_snake_case();
        let file_path = output_dir.join(format!("{file_stem}.rs"));

        let formatted_code = prettyplease::unparse(&syn_file);

        fs::write(&file_path, formatted_code)
            .unwrap_or_else(|_| panic!("Failed to write file: {}", file_path.display()));

        modules.insert(file_stem);

        println!("Generated and formatted: {}", file_path.display());
    }

    let mod_rs_path = output_dir.join("mod.rs");
    let mut mod_rs = String::new();

    for m in modules {
        mod_rs.push_str(&format!("pub mod {m};\n"));
    }

    fs::write(&mod_rs_path, mod_rs)
        .unwrap_or_else(|_| panic!("Failed to write file: {}", mod_rs_path.display()));

    println!("Generated module index: {}", mod_rs_path.display());
    println!("Form generation complete.");
}

fn layout(data: &GpuiFormShape) -> syn::File {
    let adapter = FormShapeAdapter::new(data);
    let identities = LayoutIdentities::new(adapter.shape_data);
    let LayoutIdentities {
        struct_name_str,
        context_str,
        struct_name_ident,
        struct_name_uw_ident,
        struct_name_form_ident,
        struct_name_form_fields_ident,
        struct_name_form_errors_ident,
        struct_name_form_errors_ftl_ident,
        form_id_literal,
        struct_name_path_qualifier,
    } = identities;

    let target_types_import = quote! {
      use some_lib::structs::#struct_name_path_qualifier::*;
    };

    // Handle empty structs (no components)
    let is_empty = data.components.is_empty();

    let error_ftl_variants: Vec<proc_macro2::TokenStream> = data
        .components
        .iter()
        .map(|field| {
            let variant_name = format_ident!("{}", field.field_name.to_upper_camel_case());
            quote! {
                #variant_name { value: String },
            }
        })
        .collect();

    let component_creations_tokens = adapter.cx_new_calls().unwrap_or_default();

    let field_initializers_tokens = adapter.field_initializers().unwrap_or_default();

    let render_children_tokens = adapter.child_elements();

    let subscription_calls_tokens = adapter.subscription_calls().unwrap_or_default();

    let post_subscription_init_tokens = adapter.post_subscription_initialization().unwrap_or_default();

    let (subscriptions_field, subscriptions_init) = if subscription_calls_tokens.is_empty() {
        (quote! {}, quote! {})
    } else {
        (
            quote! {
                _subscriptions: Vec<Subscription>,
            },
            quote! {
              _subscriptions,
            },
        )
    };

    let event_handlers_tokens = adapter.event_handlers().unwrap_or_default();

    // For empty structs, don't generate FormValueHolder, FormErrors, or FormErrorsFtl
    let (
        error_ftl_enum,
        current_data_field,
        errors_field,
        current_data_init,
        errors_init,
        fields_init,
        debug_child,
    ) = if is_empty {
        (
            quote! {},
            quote! {},
            quote! {},
            quote! {},
            quote! {},
            quote! { fields: #struct_name_form_fields_ident, },
            quote! {},
        )
    } else {
        (
            quote! {
                #[derive(Clone, Debug, es_fluent::EsFluent)]
                pub enum #struct_name_form_errors_ftl_ident {
                    #(#error_ftl_variants)*
                }
            },
            quote! { current_data: #struct_name_uw_ident, },
            quote! { errors: #struct_name_form_errors_ident, },
            quote! { current_data: original_data.into(), },
            quote! { errors: #struct_name_form_errors_ident::default(), },
            quote! {
                fields: #struct_name_form_fields_ident {
                    #field_initializers_tokens
                },
            },
            quote! {
                .child(format!("{:?}", self.current_data))
            },
        )
    };

    let import_tokens = quote! {
      #target_types_import
      use gpui::{
          App, AppContext, Context, Entity, FocusHandle, Focusable, InteractiveElement, IntoElement,
          ParentElement as _, Render, Styled, Subscription, Window, div, prelude::FluentBuilder as _,
      };
      use gpui_component::{
          IndexPath,
          checkbox::Checkbox, date_picker::{DatePicker, DatePickerEvent, DatePickerState},
          divider::Divider, select::{Select, SelectEvent, SelectState, SearchableVec},
          form::{field, v_form},
          input::{
              InputEvent, InputState, NumberInput, NumberInputEvent, StepAction, Input,
          },
          switch::Switch, v_flex,
      };
      use gpui_form_component::TupleEnumInner;
      use rust_decimal::Decimal;
      use std::sync::Arc;
      use es_fluent::{ThisFtl as _, ToFluentString as _};

      #error_ftl_enum
    };

    let layout_tokens = quote! {
      #import_tokens

      const CONTEXT: &str = #context_str;

      #[gpui_storybook::story_init]
      pub fn init(cx: &mut App) {
      }

      #[gpui_storybook::story]
      pub struct #struct_name_form_ident {
          original_data: Arc<#struct_name_ident>,
          #current_data_field
          #errors_field
          fields: #struct_name_form_fields_ident,
          focus_handle: FocusHandle,
          #subscriptions_field
      }

      impl Focusable for #struct_name_form_ident {
          fn focus_handle(&self, cx: &App) -> FocusHandle {
              self.focus_handle.clone()
          }
      }

      impl gpui_storybook::Story for #struct_name_form_ident {
          fn title() -> String {
              #struct_name_ident::this_ftl()
          }

          fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render + Focusable> {
              Self::view(window, cx, #struct_name_ident::default())
          }
      }

      impl #struct_name_form_ident {
          pub fn view(window: &mut Window, cx: &mut App, original_data: #struct_name_ident) -> Entity<Self> {
              cx.new(|cx| Self::new(window, cx, original_data))
          }

          #event_handlers_tokens

          fn new(window: &mut Window, cx: &mut Context<Self>, original_data: #struct_name_ident) -> Self {
            #component_creations_tokens

            #subscription_calls_tokens

            #post_subscription_init_tokens

              Self {
                  original_data: Arc::new(original_data.clone()),
                  #current_data_init
                  #errors_init
                  #fields_init
                  focus_handle: cx.focus_handle(),
                  #subscriptions_init
              }
          }
      }

      impl Render for #struct_name_form_ident {
          fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
              v_flex()
                  .key_context(CONTEXT)
                  .id(#form_id_literal)
                  .size_full()
                  .p_4()
                  .justify_start()
                  .gap_3()
                  .child(Divider::horizontal())
                  .child(
                      v_form()
                        #render_children_tokens
                  )
                  .child(Divider::horizontal())
                  #debug_child
          }
      }
    };
    syn::parse2(layout_tokens)
        .expect("Failed to parse generated tokens into syn::File for form scaffold")
}
