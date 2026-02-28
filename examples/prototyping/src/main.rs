use gpui_form::core::registry::GpuiFormShape;
use gpui_form_prototyping_core::{FormLayout, FormParts, FormShapeAdapter};
use heck::ToSnakeCase as _;
use quote::quote;
use std::{collections::BTreeSet, fs, path::Path};

// import targeted lib to get inventory registrations
extern crate some_lib;

struct StorybookLayout;

impl FormLayout for StorybookLayout {
    fn generate_file(&self, parts: &FormParts) -> syn::File {
        let FormParts {
            struct_name_ident,
            form_ident,
            form_fields_ident,
            context_str,
            form_id_literal,
            imports,
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

        syn::parse2(quote! {
            #imports
            use rust_decimal::Decimal;

            const CONTEXT: &str = #context_str;

            #[gpui_storybook::story_init]
            pub fn init(cx: &mut App) {}

            #[gpui_storybook::story]
            pub struct #form_ident {
                #current_data_field
                fields: #form_fields_ident,
                focus_handle: FocusHandle,
                #subscriptions_field
            }

            impl Focusable for #form_ident {
                fn focus_handle(&self, cx: &App) -> FocusHandle {
                    self.focus_handle.clone()
                }
            }

            impl gpui_storybook::Story for #form_ident {
                fn title() -> String {
                    #struct_name_ident::this_ftl()
                }

                fn new_view(window: &mut Window, cx: &mut App) -> Entity<impl Render + Focusable> {
                    cx.new(|cx| Self::new(window, cx))
                }
            }

            impl #form_ident {
                #event_handlers

                fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
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
                        .child(Divider::horizontal())
                        .child(
                            v_form()
                                #render_children
                        )
                        .child(Divider::horizontal())
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
    println!("Generating forms in: {}", output_dir.display());

    let mut modules: BTreeSet<String> = BTreeSet::new();

    for shape in inventory::iter::<GpuiFormShape>() {
        println!("Shape: {:?}", shape);

        let syn_file = FormShapeAdapter::new(shape).generate_file(&StorybookLayout);
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
