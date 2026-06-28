use gpui_form_schema::registry::GpuiFormShape;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::path::Path;

use crate::error::{PrototypingError, PrototypingResult};
use crate::implementations::{
    GeneratedSubscription, ResolvedField, ShapeIdentities as _, field_generator,
};
use crate::imports::{Alias, ImportItem, ImportSet};

/// Imports required by prototyping-core's own generated fragments.
///
/// Layout-specific imports such as `Render`, `Focusable`, `v_form`, or
/// `Divider` belong in the caller's [`FormLayout`] implementation rather than
/// in the shared form-shape adapter output.
const FRAGMENT_IMPORTS: &[ImportItem] = &[
    ImportItem::path("gpui::InteractiveElement"),
    ImportItem::aliased("gpui::ParentElement", Alias::Anonymous),
    ImportItem::path("gpui::Styled"),
    ImportItem::path("gpui::div"),
    ImportItem::aliased("gpui::prelude::FluentBuilder", Alias::Anonymous),
    ImportItem::aliased("gpui_component::ActiveTheme", Alias::Anonymous),
    ImportItem::path("gpui_component::form::field"),
];

#[cfg(feature = "fluent")]
const FLUENT_FRAGMENT_IMPORTS: &[ImportItem] = &[ImportItem::aliased(
    "es_fluent::FluentMessage",
    Alias::Anonymous,
)];

const SUBSCRIPTION_IMPORTS: &[ImportItem] = &[ImportItem::path("gpui::Subscription")];

struct GeneratedField<'a> {
    imports: Vec<ImportItem>,
    cx_new_call: Option<TokenStream>,
    field_initializer: Option<TokenStream>,
    render_child: TokenStream,
    subscription: Option<GeneratedSubscription>,
    post_subscription_initialization: Option<TokenStream>,
    _resolved: ResolvedField<'a>,
}

fn parse_ident(kind: &'static str, value: &str) -> PrototypingResult<syn::Ident> {
    syn::parse_str::<syn::Ident>(value).map_err(|_| PrototypingError::InvalidIdentifier {
        kind,
        value: value.to_string(),
    })
}

pub struct FormShapeAdapter<'a> {
    pub shape_data: &'a GpuiFormShape,
}

impl<'a> FormShapeAdapter<'a> {
    pub fn new(shape_data: &'a GpuiFormShape) -> Self {
        Self { shape_data }
    }

    fn validate_shape_data(&self) -> PrototypingResult<()> {
        let data = self.shape_data;

        parse_ident("struct name", data.struct_name)?;
        parse_ident("generated form ident", &format!("{}Form", data.struct_name))?;
        parse_ident(
            "generated form fields ident",
            &format!("{}FormFields", data.struct_name),
        )?;
        parse_ident(
            "generated form value holder ident",
            &format!("{}FormValueHolder", data.struct_name),
        )?;

        source_path_to_use_path(data.source_path).ok_or_else(|| {
            PrototypingError::InvalidSourcePath {
                source_path: data.source_path.to_string(),
            }
        })?;

        Ok(())
    }

    fn collect_fields(&self) -> PrototypingResult<Vec<GeneratedField<'a>>> {
        self.shape_data
            .components
            .iter()
            .map(|field| {
                parse_ident("field name", field.field_name)?;
                parse_ident("field pascal ident", &field.field_name_pascal())?;
                parse_ident("field component ident", &field.field_name_with_behaviour())?;

                let resolved = ResolvedField::new(field)?;
                let generator = field_generator(resolved.behaviour());
                let imports = generator.generate_imports(field);
                let subscription = if field.subscribable() {
                    generator.generate_subscription(&resolved, self.shape_data)
                } else {
                    None
                };

                Ok(GeneratedField {
                    imports,
                    cx_new_call: generator.generate_cx_new_call(&resolved, self.shape_data),
                    field_initializer: generator
                        .generate_field_initializers(&resolved, self.shape_data),
                    render_child: generator.generate_render_child(&resolved, self.shape_data),
                    subscription,
                    post_subscription_initialization: generator
                        .generate_post_subscription_initialization(&resolved, self.shape_data),
                    _resolved: resolved,
                })
            })
            .collect()
    }

    /// Collect all imports needed by this form's generated file.
    ///
    /// Starts with imports needed by prototyping-core's own generated fragments,
    /// then asks each field's generator for its own requirements. The result can
    /// be rendered as grouped `use` statements via [`ImportSet::to_token_stream`].
    pub fn required_imports(&self) -> ImportSet {
        let mut set = ImportSet::default();
        if !self.shape_data.components.is_empty() {
            set.extend_items(FRAGMENT_IMPORTS);
            #[cfg(feature = "fluent")]
            set.extend_items(FLUENT_FRAGMENT_IMPORTS);
        }
        if self
            .shape_data
            .components
            .iter()
            .any(|field| field.subscribable())
        {
            set.extend_items(SUBSCRIPTION_IMPORTS);
        }
        for field in self.shape_data.components {
            let generator = field_generator(&field.behaviour);
            set.extend(generator.generate_imports(field));
        }
        set
    }

    /// Compute all token-stream fragments and identifiers for this form.
    ///
    /// Prefer this when you want to assemble a fully custom `quote!{}` template.
    /// All conditional / derived fragments (e.g. `subscriptions_field`,
    /// `current_data_field`) are pre-evaluated so you only need to splice them in.
    /// Returns a [`PrototypingError`] when the input shape metadata cannot be
    /// converted into valid Rust identifiers, types, or paths.
    pub fn parts(&self) -> PrototypingResult<FormParts> {
        self.validate_shape_data()?;
        let data = self.shape_data;
        let generated_fields = self.collect_fields()?;

        let struct_name_ident = parse_ident("struct name", data.struct_name)?;
        let form_value_holder_ident = format_ident!("{}FormValueHolder", struct_name_ident);
        let form_ident = parse_ident("generated form ident", &format!("{}Form", data.struct_name))?;
        let form_fields_ident = parse_ident(
            "generated form fields ident",
            &format!("{}FormFields", data.struct_name),
        )?;
        let form_id_literal = data.form_id_literal();
        let context_str = format!("{}Form", data.struct_name);
        let source_module_path = source_path_to_use_path(data.source_path).ok_or_else(|| {
            PrototypingError::InvalidSourcePath {
                source_path: data.source_path.to_string(),
            }
        })?;
        let has_skipped_fields = data.has_skipped_fields();

        let is_empty = data.components.is_empty();
        let has_koruma = data.has_koruma();

        let component_creations: TokenStream = generated_fields
            .iter()
            .filter_map(|field| field.cx_new_call.clone())
            .collect();
        let field_initializers: TokenStream = generated_fields
            .iter()
            .filter_map(|field| field.field_initializer.clone())
            .collect();
        // METADATA-FIRST v1 — section grouping.
        //
        // `layout.section` groups *consecutive* fields (order-preserving): each
        // time the section name changes between adjacent non-skipped fields we
        // emit a section heading before that field's render child. The heading
        // reuses the already-imported `field()` builder (see `FRAGMENT_IMPORTS`)
        // so no new imports are introduced — important because
        // `required_imports_only_include_subscription_when_needed` asserts the
        // import set stays minimal.
        //
        // The first field with a declared section also emits a heading (its
        // "previous section" is `None`). Fields without a section never emit a
        // heading and reset the tracker so the next declared section re-heading
        // fires. This is a heading *hint* rendered by the scaffold; it is NOT a
        // layout engine and carries no column/collapsible semantics yet.
        let mut render_children_items: Vec<TokenStream> = Vec::with_capacity(generated_fields.len());
        let mut prev_section: Option<&'static str> = None;
        for field in &generated_fields {
            let current_section = field._resolved.layout().section;
            if current_section.is_some() && current_section != prev_section {
                let heading = current_section.unwrap_or_default();
                render_children_items.push(quote! {
                    .child(
                        field()
                            .label(#heading)
                    )
                });
            }
            prev_section = current_section;
            render_children_items.push(field.render_child.clone());
        }
        let render_children: TokenStream = render_children_items.into_iter().collect();
        let subscription_call_items: Vec<TokenStream> = generated_fields
            .iter()
            .filter_map(|field| field.subscription.as_ref())
            .flat_map(|subscription| subscription.calls.iter().cloned())
            .collect();
        let event_handler_items: Vec<TokenStream> = generated_fields
            .iter()
            .filter_map(|field| field.subscription.as_ref())
            .flat_map(|subscription| subscription.handlers.iter().cloned())
            .collect();
        let subscription_calls = if subscription_call_items.is_empty() {
            TokenStream::new()
        } else {
            quote! {
                let mut _subscriptions = vec![#(#subscription_call_items),*];
            }
        };
        let event_handlers = if event_handler_items.is_empty() {
            TokenStream::new()
        } else {
            quote! {
                #(#event_handler_items)*
            }
        };
        let post_subscription_init: TokenStream = generated_fields
            .iter()
            .filter_map(|field| field.post_subscription_initialization.clone())
            .collect();

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
                let into_original_debug_child = if has_skipped_fields {
                    quote! {
                        .child(format!(
                            "into_original: incomplete; present_fields_json: {}",
                            self.current_data.present_fields_json()
                        ))
                    }
                } else {
                    quote! {
                        .child(format!(
                            "into_original: {:?}",
                            #form_value_holder_ident::try_from(self.current_data.clone())
                        ))
                    }
                };

                (
                    quote! { current_data: #form_value_holder_ident, },
                    quote! { let current_data = #form_value_holder_ident::default(); },
                    quote! { current_data, },
                    quote! {
                        fields: #form_fields_ident {
                            #field_initializers
                        },
                    },
                    quote! {
                        .child(format!("value_holder: {:?}", self.current_data))
                        #into_original_debug_child
                    },
                )
            };

        let replace_current_data_fn = if is_empty {
            quote! {}
        } else {
            quote! {
                pub fn replace_current_data(
                    &mut self,
                    current_data: #form_value_holder_ident,
                    window: &mut Window,
                    cx: &mut Context<Self>,
                ) {
                    *self = Self::new_with_current_data(current_data, window, cx);
                    cx.notify();
                }
            }
        };

        let mut collected_imports = ImportSet::default();
        if !generated_fields.is_empty() {
            collected_imports.extend_items(FRAGMENT_IMPORTS);
            #[cfg(feature = "fluent")]
            collected_imports.extend_items(FLUENT_FRAGMENT_IMPORTS);
        }
        if !subscription_call_items.is_empty() {
            collected_imports.extend_items(SUBSCRIPTION_IMPORTS);
        }
        for field in &generated_fields {
            collected_imports.extend(field.imports.clone());
        }
        let collected_imports = collected_imports.to_token_stream();
        let imports = quote! {
            use #source_module_path::*;
            #collected_imports
        };

        Ok(FormParts {
            struct_name_ident,
            form_ident,
            form_fields_ident,
            form_value_holder_ident,
            source_module_path,
            context_str,
            form_id_literal,
            is_empty,
            has_koruma,
            has_skipped_fields,
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
            replace_current_data_fn,
        })
    }

    /// Generate a `syn::File` from a [`FormLayout`] implementation.
    ///
    /// Returns a [`PrototypingError`] when the shape metadata is malformed.
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
    /// FormShapeAdapter::new(shape).generate_file(&MyLayout)?;
    /// ```
    pub fn generate_file(&self, layout: &impl FormLayout) -> PrototypingResult<syn::File> {
        let parts = self.parts()?;
        Ok(layout.generate_file(&parts))
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
    /// True when at least one source field was marked with `#[gpui_form(skip)]`.
    pub has_skipped_fields: bool,

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
    /// Debug rows for value-holder and into-original status; empty for empty forms.
    pub debug_child: TokenStream,
    /// `replace_current_data(...)` helper method; empty for empty forms.
    pub replace_current_data_fn: TokenStream,
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

#[cfg(test)]
mod tests {
    use super::FormShapeAdapter;
    use crate::error::PrototypingError;
    use gpui_form_schema::{
        components::ComponentsBehaviour,
        layout::FieldLayout,
        registry::{FieldVariant, GpuiFormShape},
    };
    #[cfg(not(feature = "fluent"))]
    use gpui_form_schema::layout::LayoutWidth;

    fn compact(input: &str) -> String {
        input.chars().filter(|c| !c.is_whitespace()).collect()
    }

    #[test]
    fn parts_return_error_for_invalid_source_path() {
        const SHAPE: GpuiFormShape = GpuiFormShape::new("Demo", &[], "demo.rs", false);

        let error = match FormShapeAdapter::new(&SHAPE).parts() {
            Ok(_) => panic!("invalid source paths should return an error"),
            Err(error) => error,
        };

        assert_eq!(
            error,
            PrototypingError::InvalidSourcePath {
                source_path: "demo.rs".to_string(),
            }
        );
    }

    #[test]
    fn parts_return_error_for_invalid_field_type_metadata() {
        const FIELDS: [FieldVariant; 1] = [FieldVariant::new(
            "country",
            "Vec<",
            false,
            ComponentsBehaviour::Select(gpui_form_schema::components::SelectBehaviour {
                partial: false,
                searchable: false,
            }),
        )];
        const SHAPE: GpuiFormShape =
            GpuiFormShape::new("Demo", &FIELDS, "examples/some-lib/src/demo.rs", false);

        let error = match FormShapeAdapter::new(&SHAPE).parts() {
            Ok(_) => panic!("invalid field types should return an error"),
            Err(error) => error,
        };

        assert!(
            matches!(
                error,
                PrototypingError::InvalidType {
                    ref field_name,
                    ref value,
                    ..
                } if field_name == "country" && value == "Vec<"
            ),
            "unexpected error: {error:?}"
        );
    }

    #[test]
    fn required_imports_only_include_subscription_when_needed() {
        const FIELDS: [FieldVariant; 1] = [FieldVariant::new(
            "enabled",
            "bool",
            false,
            ComponentsBehaviour::Checkbox,
        )];
        const SHAPE: GpuiFormShape =
            GpuiFormShape::new("Demo", &FIELDS, "examples/some-lib/src/demo.rs", false);

        let parts = FormShapeAdapter::new(&SHAPE)
            .parts()
            .expect("valid checkbox shapes should generate parts");
        let compact = compact(&parts.imports.to_string());

        assert!(
            !compact.contains("usegpui::Subscription;"),
            "subscription import should be omitted when no generated subscriptions exist: {compact}"
        );
    }

    // ── METADATA-FIRST v1: section grouping + label/description hints ────────

    // Helper: a FieldVariant with an Input behaviour and an explicit layout.
    const fn input_field(name: &'static str, layout: FieldLayout) -> FieldVariant {
        FieldVariant::new(name, "String", false, ComponentsBehaviour::Input).with_layout(layout)
    }

    #[test]
    fn render_children_emits_section_heading_when_section_changes() {
        const LAYOUT_ACCOUNT: FieldLayout = FieldLayout::new().with_section(Some("Account"));
        const LAYOUT_PROFILE: FieldLayout = FieldLayout::new().with_section(Some("Profile"));
        const FIELDS: [FieldVariant; 3] = [
            input_field("username", LAYOUT_ACCOUNT),
            input_field("email", LAYOUT_ACCOUNT),
            input_field("bio", LAYOUT_PROFILE),
        ];
        const SHAPE: GpuiFormShape =
            GpuiFormShape::new("Demo", &FIELDS, "examples/some-lib/src/demo.rs", false);

        let parts = FormShapeAdapter::new(&SHAPE)
            .parts()
            .expect("valid shapes should generate parts");
        let render = parts.render_children.to_string();

        // Two distinct sections => two headings ("Account" then "Profile").
        // Each heading reuses the existing `field().label(...)` builder.
        assert!(
            render.matches("Account").count() >= 1,
            "Account section heading should be emitted: {render}"
        );
        assert!(
            render.matches("Profile").count() >= 1,
            "Profile section heading should be emitted: {render}"
        );

        // The heading appears as a `field().label("Account")` style child, not
        // just as a bare string literal.
        let compact = compact(&render);
        assert!(
            compact.contains("field().label(\"Account\")"),
            "section heading should reuse the field() builder: {compact}"
        );
    }

    #[test]
    fn render_children_does_not_repeat_heading_for_consecutive_same_section() {
        const LAYOUT_ACCOUNT: FieldLayout = FieldLayout::new().with_section(Some("Account"));
        const FIELDS: [FieldVariant; 2] = [
            input_field("username", LAYOUT_ACCOUNT),
            input_field("email", LAYOUT_ACCOUNT),
        ];
        const SHAPE: GpuiFormShape =
            GpuiFormShape::new("Demo", &FIELDS, "examples/some-lib/src/demo.rs", false);

        let parts = FormShapeAdapter::new(&SHAPE)
            .parts()
            .expect("valid shapes should generate parts");
        let render = parts.render_children.to_string();

        // Same section for both fields => exactly one "Account" heading
        // (emitted only on section change). The field-name label tokens must
        // not coincidentally match "Account".
        assert_eq!(
            render.matches("Account").count(),
            1,
            "consecutive same-section fields should emit exactly one heading: {render}"
        );
    }

    #[test]
    fn render_children_omits_heading_when_no_section_declared() {
        // No layout hints at all — defaults to empty FieldLayout.
        const FIELDS: [FieldVariant; 2] = [
            FieldVariant::new("name", "String", false, ComponentsBehaviour::Input),
            FieldVariant::new("email", "String", false, ComponentsBehaviour::Input),
        ];
        const SHAPE: GpuiFormShape =
            GpuiFormShape::new("Demo", &FIELDS, "examples/some-lib/src/demo.rs", false);

        let parts = FormShapeAdapter::new(&SHAPE)
            .parts()
            .expect("valid shapes should generate parts");
        let compact = compact(&parts.render_children.to_string());

        // No section => no synthetic `field().label(...)` heading tokens beyond
        // the per-field ones the generators already emit.
        assert!(
            !compact.contains("field().label(\"\")"),
            "no empty section heading should be emitted when section is absent: {compact}"
        );
    }

    #[test]
    fn render_children_heading_re_appears_after_unsectioned_field_resets_tracker() {
        // Order: [Account, (none), Account] — the middle field has no section,
        // so the trailing Account field must re-emit a heading because the
        // tracker resets to None when a field without a section is encountered.
        const LAYOUT_ACCOUNT: FieldLayout = FieldLayout::new().with_section(Some("Account"));
        const FIELDS: [FieldVariant; 3] = [
            input_field("username", LAYOUT_ACCOUNT),
            FieldVariant::new("nickname", "String", false, ComponentsBehaviour::Input),
            input_field("recovery_email", LAYOUT_ACCOUNT),
        ];
        const SHAPE: GpuiFormShape =
            GpuiFormShape::new("Demo", &FIELDS, "examples/some-lib/src/demo.rs", false);

        let parts = FormShapeAdapter::new(&SHAPE)
            .parts()
            .expect("valid shapes should generate parts");
        let render = parts.render_children.to_string();

        assert_eq!(
            render.matches("Account").count(),
            2,
            "section heading should re-appear after an intervening unsectioned field resets the tracker: {render}"
        );
    }

    // The label override only applies in the non-fluent label branch; the
    // fluent branch localizes through `es-fluent` keys instead (v1 minimal).
    #[test]
    #[cfg(not(feature = "fluent"))]
    fn layout_label_override_appears_in_render_children() {
        const LAYOUT: FieldLayout = FieldLayout::new()
            .with_label(Some("Enable experiments"))
            .with_width(LayoutWidth::Half);
        const FIELDS: [FieldVariant; 1] = [input_field("enable_experimental", LAYOUT)];
        const SHAPE: GpuiFormShape =
            GpuiFormShape::new("Demo", &FIELDS, "examples/some-lib/src/demo.rs", false);

        let parts = FormShapeAdapter::new(&SHAPE)
            .parts()
            .expect("valid shapes should generate parts");
        let render = parts.render_children.to_string();

        assert!(
            render.contains("Enable experiments"),
            "layout.label override should be used as the field's display label: {render}"
        );
        // The label call itself should carry the override verbatim, while the
        // description (which has no layout.description hint here) legitimately
        // falls back to the title-cased field name.
        assert!(
            compact(&render).contains("field().label(\"Enableexperiments\")"),
            "the field() label call should use the override verbatim: {render}"
        );
    }
}
