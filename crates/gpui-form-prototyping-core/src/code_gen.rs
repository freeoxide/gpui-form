use gpui_form_core::components::ComponentsBehaviour;
use gpui_form_core::registry::GpuiFormShape;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::path::Path;

use crate::implementations::ComponentIdentities as _;
use crate::imports::{Alias, ImportItem, ImportSet};

use super::implementations::{
    ComponentShape, FieldGenerator, checkbox::CheckboxCodeGenerator, custom::CustomCodeGenerator,
    date_picker::DatePickerCodeGenerator, infinite_select::InfiniteSelectCodeGenerator,
    input::InputCodeGenerator, number_input::NumberInputCodeGenerator, select::SelectCodeGenerator,
    switch::SwitchCodeGenerator,
};

/// Imports every generated file needs regardless of which components it contains.
///
/// Component-specific items (e.g. `Input`, `Subscription`, `Checkbox`) are
/// declared by the individual generators and collected via [`FormShapeAdapter::required_imports`].
const FRAMEWORK_IMPORTS: &[ImportItem] = &[
    // gpui core — always hardcoded, never declared by individual generators
    ImportItem::path("gpui::App"),
    ImportItem::path("gpui::AppContext"),
    ImportItem::path("gpui::Context"),
    ImportItem::path("gpui::Entity"),
    ImportItem::path("gpui::FocusHandle"),
    ImportItem::path("gpui::Focusable"),
    ImportItem::path("gpui::IntoElement"),
    ImportItem::path("gpui::InteractiveElement"),
    ImportItem::aliased("gpui::ParentElement", Alias::Anonymous),
    ImportItem::path("gpui::Render"),
    ImportItem::path("gpui::Styled"),
    ImportItem::path("gpui::Subscription"),
    ImportItem::path("gpui::Window"),
    ImportItem::path("gpui::div"),
    ImportItem::aliased("gpui::prelude::FluentBuilder", Alias::Anonymous),
    // gpui_component layout / form helpers
    ImportItem::aliased("gpui_component::ActiveTheme", Alias::Anonymous),
    ImportItem::path("gpui_component::divider::Divider"),
    ImportItem::path("gpui_component::form::field"),
    ImportItem::path("gpui_component::form::v_form"),
    ImportItem::path("gpui_component::v_flex"),
    // i18n / fluent
    ImportItem::aliased("es_fluent::ThisFtl", Alias::Anonymous),
    ImportItem::aliased("es_fluent::ToFluentString", Alias::Anonymous),
];

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

    /// Collect all imports needed by this form's generated file.
    ///
    /// Starts with the universal [`FRAMEWORK_IMPORTS`] base, then asks each
    /// field's generator for its own requirements. The result can be rendered
    /// as grouped `use` statements via [`ImportSet::to_token_stream`].
    pub fn required_imports(&self) -> ImportSet {
        let mut set = ImportSet::default();
        set.extend_items(FRAMEWORK_IMPORTS);
        for field in self.shape_data.components {
            let generator = field_generator(&field.behaviour);
            set.extend(generator.as_generator().generate_imports(field));
        }
        set
    }

    /// Compute all token-stream fragments and identifiers for this form.
    ///
    /// Prefer this when you want to assemble a fully custom `quote!{}` template.
    /// All conditional / derived fragments (e.g. `subscriptions_field`,
    /// `current_data_field`) are pre-evaluated so you only need to splice them in.
    pub fn parts(&self) -> FormParts {
        let data = self.shape_data;

        let struct_name_ident = data.struct_name_ident();
        let form_value_holder_ident = format_ident!("{}FormValueHolder", struct_name_ident);
        let form_ident = data.struct_form_ident();
        let form_fields_ident = data.struct_form_fields_ident();
        let form_id_literal = data.form_id_literal();
        let context_str = format!("{}Form", data.struct_name);
        let source_module_path = source_path_to_use_path(data.source_path)
            .unwrap_or_else(|| panic!("Failed to parse source_path: {}", data.source_path));

        let is_empty = data.components.is_empty();
        let has_koruma = data.has_koruma();

        let component_creations = self.cx_new_calls().unwrap_or_default();
        let field_initializers = self.field_initializers().unwrap_or_default();
        let render_children = self.child_elements();
        let event_handlers = self.event_handlers().unwrap_or_default();
        let subscription_calls = self.subscription_calls().unwrap_or_default();
        let post_subscription_init = self.post_subscription_initialization().unwrap_or_default();

        let validation_binding = if has_koruma {
            quote! { let validation_errors = self.current_data.validate().err(); }
        } else {
            quote! {}
        };

        let (subscriptions_field, subscriptions_init) = if subscription_calls.is_empty() {
            (quote! {}, quote! {})
        } else {
            (
                quote! { _subscriptions: Vec<Subscription>, },
                quote! { _subscriptions, },
            )
        };

        let (current_data_field, current_data_let, current_data_init, fields_init, debug_child) =
            if is_empty {
                (
                    quote! {},
                    quote! {},
                    quote! {},
                    quote! { fields: #form_fields_ident, },
                    quote! {},
                )
            } else {
                (
                    quote! { current_data: #form_value_holder_ident, },
                    quote! { let current_data = #form_value_holder_ident::default(); },
                    quote! { current_data, },
                    quote! {
                        fields: #form_fields_ident {
                            #field_initializers
                        },
                    },
                    quote! { .child(format!("{:?}", self.current_data)) },
                )
            };

        let collected_imports = self.required_imports().to_token_stream();
        let imports = quote! {
            use #source_module_path::*;
            #collected_imports
        };

        FormParts {
            struct_name_ident,
            form_ident,
            form_fields_ident,
            form_value_holder_ident,
            source_module_path,
            context_str,
            form_id_literal,
            is_empty,
            has_koruma,
            imports,
            component_creations,
            field_initializers,
            render_children,
            event_handlers,
            subscription_calls,
            post_subscription_init,
            validation_binding,
            subscriptions_field,
            subscriptions_init,
            current_data_field,
            current_data_let,
            current_data_init,
            fields_init,
            debug_child,
        }
    }

    /// Generate a `syn::File` from a [`FormLayout`] implementation.
    ///
    /// ```rust,ignore
    /// struct MyLayout;
    /// impl FormLayout for MyLayout {
    ///     fn generate_file(&self, parts: &FormParts) -> syn::File {
    ///         let FormParts { imports, render_children, form_ident, .. } = parts;
    ///         syn::parse2(quote! {
    ///             #imports
    ///             pub struct #form_ident { /* ... */ }
    ///             // splice #render_children wherever you need it
    ///         }).unwrap()
    ///     }
    /// }
    /// FormShapeAdapter::new(shape).generate_file(&MyLayout);
    /// ```
    pub fn generate_file(&self, layout: &impl FormLayout) -> syn::File {
        layout.generate_file(&self.parts())
    }
}

// ── FormParts ─────────────────────────────────────────────────────────────────

/// All pre-computed token-stream fragments and identifiers for one form scaffold.
///
/// Obtained via [`FormShapeAdapter::parts`] and consumed by [`FormLayout::generate_file`].
/// Every field is `pub` so custom layouts can freely destructure and splice whichever
/// pieces they need.
pub struct FormParts {
    // ── Identifiers ──────────────────────────────────────────────────────────
    /// The original struct ident, e.g. `User`.
    pub struct_name_ident: syn::Ident,
    /// Generated form view ident, e.g. `UserForm`.
    pub form_ident: syn::Ident,
    /// Generated form fields ident, e.g. `UserFormFields`.
    pub form_fields_ident: syn::Ident,
    /// Generated value holder ident, e.g. `UserFormValueHolder`.
    pub form_value_holder_ident: syn::Ident,
    /// Glob import path for the source module, e.g. `some_lib::structs::user`.
    pub source_module_path: syn::Path,
    /// GPUI key context string, e.g. `"UserForm"`.
    pub context_str: String,
    /// GPUI element id, e.g. `"user-form"`.
    pub form_id_literal: String,

    // ── Flags ─────────────────────────────────────────────────────────────────
    /// True when the struct has no component fields.
    pub is_empty: bool,
    /// True when koruma validation is enabled.
    pub has_koruma: bool,

    // ── Raw generated fragments ───────────────────────────────────────────────
    /// Grouped `use` statements (source module glob + framework base + per-component items).
    pub imports: TokenStream,
    /// `cx.new(|cx| FormComponents::field(window, cx))` calls.
    pub component_creations: TokenStream,
    /// Field name tokens for the `FormFields { ... }` struct literal.
    pub field_initializers: TokenStream,
    /// `.child(field().label(...).child(...))` chains for the form body.
    pub render_children: TokenStream,
    /// Event handler `fn` items to place in an `impl` block.
    pub event_handlers: TokenStream,
    /// `let mut _subscriptions = vec![...]` binding.
    pub subscription_calls: TokenStream,
    /// Post-subscription setup (e.g. populating initial field values).
    pub post_subscription_init: TokenStream,
    /// `let validation_errors = ...` binding; empty when koruma is disabled.
    pub validation_binding: TokenStream,

    // ── Derived conditional fragments ─────────────────────────────────────────
    /// `_subscriptions: Vec<Subscription>,` struct field; empty when no subscriptions.
    pub subscriptions_field: TokenStream,
    /// `_subscriptions,` in `Self { ... }`; empty when no subscriptions.
    pub subscriptions_init: TokenStream,
    /// `current_data: FormValueHolder,` struct field; empty for empty forms.
    pub current_data_field: TokenStream,
    /// `let current_data = FormValueHolder::default();` binding; empty for empty forms.
    pub current_data_let: TokenStream,
    /// `current_data,` in `Self { ... }`; empty for empty forms.
    pub current_data_init: TokenStream,
    /// `fields: FormFields { #field_initializers }` initializer block.
    pub fields_init: TokenStream,
    /// `.child(format!("{:?}", self.current_data))` debug row; empty for empty forms.
    pub debug_child: TokenStream,
}

// ── FormLayout ────────────────────────────────────────────────────────────────

/// Template strategy for [`FormShapeAdapter::generate_file`].
///
/// Implement this to fully control the shape of the generated file while
/// reusing all the pre-computed [`FormParts`] fragments.
pub trait FormLayout {
    fn generate_file(&self, parts: &FormParts) -> syn::File;
}

/// Converts a `file!()` source path like
/// `examples/some-lib/src/structs/user.rs` into a use-path like
/// `some_lib::structs::user` for the glob import at the top of each generated file.
fn source_path_to_use_path(source_path: &str) -> Option<syn::Path> {
    let path = Path::new(source_path);
    let components: Vec<_> = path.components().collect();

    let src_index = components
        .iter()
        .position(|c| matches!(c, std::path::Component::Normal(s) if s.to_str() == Some("src")))?;

    if src_index == 0 {
        return None;
    }
    let crate_name = match &components[src_index - 1] {
        std::path::Component::Normal(s) => s.to_str()?.replace('-', "_"),
        _ => return None,
    };

    let mut path_segments = vec![crate_name];
    for component in &components[src_index + 1..] {
        if let std::path::Component::Normal(s) = component {
            let segment = s.to_str()?;
            if segment == "mod.rs" {
                continue;
            }
            path_segments.push(
                segment
                    .strip_suffix(".rs")
                    .unwrap_or(segment)
                    .replace('-', "_"),
            );
        }
    }

    syn::parse_str(&path_segments.join("::")).ok()
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
