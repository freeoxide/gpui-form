use gpui_form::schema::registry::GpuiFormShape;
use gpui_form_prototyping_core::{FormLayout, FormParts, FormShapeAdapter};
use heck::ToSnakeCase as _;
use quote::quote;
use std::{collections::BTreeSet, fs, path::Path};

rust_i18n::i18n!("locales", fallback = "en");

// import targeted lib to get inventory registrations
extern crate some_lib;

struct ComponentLayout;

impl FormLayout for ComponentLayout {
    fn generate_file(&self, parts: &FormParts) -> syn::File {
        let FormParts {
            struct_name_ident,
            form_ident,
            form_fields_ident,
            form_value_holder_ident,
            context_str,
            form_id_literal,
            is_empty,
            has_koruma,
            has_skipped_fields,
            imports,
            i18n_key_items,
            component_creations,
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
            render_children,
            ..
        } = parts;

        let submit_payload_type = if *is_empty {
            quote! { () }
        } else if *has_skipped_fields {
            if *has_koruma {
                quote! { Result<#form_value_holder_ident, String> }
            } else {
                quote! { #form_value_holder_ident }
            }
        } else if *has_koruma {
            quote! { Result<Option<#struct_name_ident>, String> }
        } else {
            quote! { #struct_name_ident }
        };

        let submit_payload_expr = if *is_empty {
            quote! { () }
        } else if *has_skipped_fields {
            if *has_koruma {
                quote! {
                    match self.current_data.validate() {
                        Ok(_) => Ok(self.current_data.clone()),
                        Err(error) => Err(format!("{error:?}")),
                    }
                }
            } else {
                quote! { self.current_data.clone() }
            }
        } else if *has_koruma {
            quote! {
                match self.current_data.validate() {
                    Ok(_) => Ok(#form_value_holder_ident::try_from(self.current_data.clone()).ok()),
                    Err(error) => Err(format!("{error:?}")),
                }
            }
        } else {
            quote! { self.current_data.clone().into() }
        };

        let submit_disabled = if *has_koruma {
            quote! { self.current_data.validate().is_err() }
        } else {
            quote! { false }
        };

        let form_action_helpers = if *is_empty {
            quote! {}
        } else {
            quote! {
                fn reset_form(&mut self, window: &mut Window, cx: &mut Context<Self>) {
                    *self = Self::new(window, cx);
                    cx.notify();
                }

                fn submit_payload(&self) -> #submit_payload_type {
                    #submit_payload_expr
                }

                fn submit_button(
                    &self,
                    cx: &mut Context<Self>,
                    label: impl Into<::gpui::SharedString>,
                    on_submit: impl Fn(#submit_payload_type, &mut Window, &mut Context<Self>) + 'static,
                ) -> ::gpui_kit::component::button::Button {
                    ::gpui_kit::component::button::Button::new(format!("{}-submit-button", #form_id_literal))
                        .label(label)
                        .disabled(#submit_disabled)
                        .on_click(cx.listener(move |this, _, window, cx| {
                            on_submit(this.submit_payload(), window, cx);
                        }))
                }

                fn reset_button(
                    &self,
                    cx: &mut Context<Self>,
                    label: impl Into<::gpui::SharedString>,
                ) -> ::gpui_kit::component::button::Button {
                    ::gpui_kit::component::button::Button::new(format!("{}-reset-button", #form_id_literal))
                        .label(label)
                        .on_click(cx.listener(|this, _, window, cx| {
                            this.reset_form(window, cx);
                        }))
                }

                fn action_buttons(
                    &self,
                    cx: &mut Context<Self>,
                    on_submit: impl Fn(#submit_payload_type, &mut Window, &mut Context<Self>) + 'static,
                ) -> impl IntoElement {
                    div()
                        .flex()
                        .gap_2()
                        .child(self.submit_button(cx, localize(cx, FormAction::Submit.key()), on_submit))
                        .child(self.reset_button(cx, localize(cx, FormAction::Reset.key())))
                }
            }
        };

        let action_buttons_child = if *is_empty {
            quote! {}
        } else {
            quote! {
                .child(
                    field()
                        .label_indent(false)
                        .child(self.action_buttons(cx, |payload, _, _| {
                            // User-defined submit handler goes here.
                            let _ = payload;
                        })),
                )
            }
        };

        let form_action_import = if *is_empty {
            quote! {}
        } else {
            quote! { use ::some_lib::structs::form_action::FormAction; }
        };

        let title_key = format!("{}_label", struct_name_ident.to_string().to_snake_case());

        let scaffold_imports = if *is_empty {
            quote! {
                use ::gpui::{App, Context, FocusHandle, Focusable, InteractiveElement, IntoElement, ParentElement as _, Render, Styled, Window};
            }
        } else {
            quote! {
                use ::gpui::{App, AppContext, Context, Entity, FocusHandle, Focusable, InteractiveElement, IntoElement, ParentElement as _, Render, Styled, Window};
                use ::gpui_kit::component::Disableable as _;
            }
        };

        syn::parse2(quote! {
            #imports
            #i18n_key_items
            #scaffold_imports
            use ::gpui_kit::component::separator::Separator;
            use ::gpui_kit::component::form::v_form;
            use ::gpui_kit::component::v_flex;
            #form_action_import

            const CONTEXT: &str = #context_str;

            fn localize(cx: &impl ::std::borrow::Borrow<App>, key: &str) -> String {
                crate::i18n::localize_message(cx, key)
            }

            pub struct #form_ident {
                #current_data_field
                fields: #form_fields_ident,
                focus_handle: FocusHandle,
                #subscriptions_field
            }

            impl Focusable for #form_ident {
                fn focus_handle(&self, _cx: &App) -> FocusHandle {
                    self.focus_handle.clone()
                }
            }

            impl #form_ident {
                #event_handlers

                pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
                    #current_data_let

                    #component_creations

                    #subscription_calls

                    #post_subscription_init

                    Self {
                        #current_data_init
                        #fields_init
                        focus_handle: cx.focus_handle(),
                        #subscriptions_init
                    }
                }

                #form_action_helpers
            }

            impl Render for #form_ident {
                fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
#validation_binding
                    v_flex()
                        .key_context(CONTEXT)
                        .id(#form_id_literal)
                        .size_full()
                        .p_4()
                        .justify_start()
                        .gap_3()
                        .child(
                            v_flex()
                                .text_lg()
                                .font_semibold()
                                .child(localize(cx, #title_key)),
                        )
                        .child(Separator::horizontal())
                        .child(
                            v_form()
                                #render_children
                                #action_buttons_child
                        )
                        .child(Separator::horizontal())
                        #debug_child
                }
            }
        })
        .expect("Failed to parse generated tokens into syn::File for form scaffold")
    }
}

fn main() {
    let output_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("output");
    fs::create_dir_all(&output_dir).expect("Failed to create output directory");
    for entry in fs::read_dir(&output_dir).expect("Failed to read output directory") {
        let entry = entry.expect("Failed to inspect output entry");
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "rs")
            && path.file_name().is_none_or(|name| name != "mod.rs")
        {
            fs::remove_file(&path)
                .unwrap_or_else(|_| panic!("Failed to remove stale file: {}", path.display()));
        }
    }
    println!("Generating forms in: {}", output_dir.display());

    let mut modules: BTreeSet<String> = BTreeSet::new();

    for shape in inventory::iter::<GpuiFormShape>() {
        println!("Shape: {:?}", shape);

        let syn_file = FormShapeAdapter::new(shape)
            .generate_file(&ComponentLayout)
            .unwrap_or_else(|err| {
                panic!(
                    "Failed to generate prototyping scaffold for {}: {err}",
                    shape.struct_name
                )
            });
        let file_stem = shape.struct_name.to_snake_case();
        let file_path = output_dir.join(format!("{file_stem}.rs"));

        fs::write(&file_path, prettyplease::unparse(&syn_file))
            .unwrap_or_else(|_| panic!("Failed to write file: {}", file_path.display()));

        modules.insert(file_stem);
        println!("Generated and formatted: {}", file_path.display());
    }

    let mod_rs_path = output_dir.join("mod.rs");
    let mod_rs = modules
        .iter()
        .map(|m| format!("pub mod {m};\n"))
        .collect::<String>();

    fs::write(&mod_rs_path, mod_rs)
        .unwrap_or_else(|_| panic!("Failed to write file: {}", mod_rs_path.display()));

    println!("Generated module index: {}", mod_rs_path.display());
    println!("Form generation complete.");
}

#[cfg(test)]
mod i18n_tests {
    use gpui_form::schema::registry::GpuiFormShape;
    use gpui_form_prototyping_core::implementations::collect_i18n_keys;

    const FORM_TITLE_KEYS: &[&str] = &[
        "empty_label",
        "item_label",
        "location_form_label",
        "user_label",
    ];

    const ACTION_KEYS: &[&str] = &["form_action.submit", "form_action.reset"];

    const LOCALES: &[&str] = &["en", "fr-FR", "zh-CN"];

    fn all_keys() -> Vec<String> {
        let mut keys: Vec<String> = FORM_TITLE_KEYS
            .iter()
            .chain(ACTION_KEYS.iter())
            .map(|key| key.to_string())
            .collect();
        for shape in inventory::iter::<GpuiFormShape>() {
            keys.extend(collect_i18n_keys(shape));
        }
        keys
    }

    #[test]
    fn every_key_resolves_in_every_locale() {
        let keys = all_keys();
        assert!(
            !keys.is_empty(),
            "prototyping key inventory must not be empty"
        );
        for key in &keys {
            let key: &str = key;
            for locale in LOCALES.iter().copied() {
                let translated = rust_i18n::t!(key, locale = locale);
                assert_ne!(
                    &*translated, key,
                    "key {key} unresolved for locale {locale}"
                );
            }
        }
    }
}
