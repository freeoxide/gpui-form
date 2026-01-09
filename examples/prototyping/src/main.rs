#![allow(unused)]

use gpui_form::core::registry::GpuiFormShape;
use gpui_form_prototyping_core::{
    code_gen::FormShapeAdapter,
    implementations::{ComponentIdentities as _, ComponentShape as _},
};
use heck::{ToSnakeCase as _, ToUpperCamelCase as _};

use quote::{format_ident, quote};
use std::{collections::BTreeSet, fs, path::Path};

// import targeted lib to get inventory registrations
extern crate some_lib;

fn source_path_to_use_path(source_path: &str) -> Option<syn::Path> {
    let path = Path::new(source_path);
    let components: Vec<_> = path.components().collect();

    let src_index = components
        .iter()
        .position(|c| matches!(c, std::path::Component::Normal(s) if s.to_str() == Some("src")))?;

    if src_index == 0 {
        return None;
    }
    let crate_component = &components[src_index - 1];
    let crate_name = match crate_component {
        std::path::Component::Normal(s) => s.to_str()?.replace('-', "_"),
        _ => return None,
    };

    let mut path_segments = vec![crate_name];

    for component in &components[src_index + 1..] {
        if let std::path::Component::Normal(s) = component {
            let segment = s.to_str()?;
            let segment = segment.strip_suffix(".rs").unwrap_or(segment);
            path_segments.push(segment.replace('-', "_"));
        }
    }

    let path_str = path_segments.join("::");
    syn::parse_str(&path_str).ok()
}

struct LayoutIdentities {
    struct_name_str: &'static str,
    context_str: String,
    struct_name_ident: syn::Ident,
    struct_name_uw_ident: syn::Ident,
    struct_name_form_ident: syn::Ident,
    struct_name_form_fields_ident: syn::Ident,
    form_id_literal: String,
    /// The full module path to the source file, derived from source_path.
    /// e.g., `some_lib::structs::empty` for `examples/some-lib/src/structs/empty.rs`
    source_module_path: syn::Path,
}

impl LayoutIdentities {
    fn new(shape: &GpuiFormShape) -> Self {
        let struct_name_str = shape.struct_name;
        let context_str = format!("{}Form", struct_name_str);
        let struct_name_ident = shape.struct_name_ident();
        let struct_name_uw_ident = format_ident!("{}FormValueHolder", struct_name_ident);
        let struct_name_form_ident = shape.struct_form_ident();
        let struct_name_form_fields_ident = shape.struct_form_fields_ident();
        let form_id_literal = shape.form_id_literal();
        let source_module_path = source_path_to_use_path(shape.source_path)
            .unwrap_or_else(|| panic!("Failed to parse source_path: {}", shape.source_path));

        Self {
            struct_name_str,
            context_str,
            struct_name_ident,
            struct_name_uw_ident,
            struct_name_form_ident,
            struct_name_form_fields_ident,
            form_id_literal,
            source_module_path,
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
        form_id_literal,
        source_module_path,
    } = identities;

    let target_types_import = quote! {
      use #source_module_path::*;
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

    let any_validations = data
        .components
        .iter()
        .any(|field| !field.validations.is_empty());

    let validation_binding = if any_validations {
        quote! {
            let validation_errors = #struct_name_ident::from(self.current_data.clone()).validate().err();
        }
    } else {
        quote! {}
    };

    let subscription_calls_tokens = adapter.subscription_calls().unwrap_or_default();

    let post_subscription_init_tokens = adapter
        .post_subscription_initialization()
        .unwrap_or_default();

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
            quote! {},
            quote! { current_data: #struct_name_uw_ident, },
            quote! {},
            quote! { current_data: original_data.into(), },
            quote! {},
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
          ActiveTheme as _, IndexPath,
          checkbox::Checkbox,
          date_picker::{DatePicker, DatePickerEvent, DatePickerState},
          divider::Divider,
          form::{field, v_form},
          input::{Input, InputEvent, InputState, NumberInput, NumberInputEvent, StepAction},
          select::{SearchableVec, Select, SelectEvent, SelectState},
          switch::Switch,
          v_flex,
      };
      use gpui_form_component::tuple_select::TupleEnumInner;
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
              #validation_binding
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
