use gpui_form_schema::registry::{FieldVariant, GpuiFormShape};
use proc_macro2::TokenStream;
use quote::quote;

use crate::imports::ImportItem;

use super::{
    FieldCodeGenerator, GeneratedSubscription, ResolvedField, generate_entity_creation,
    generate_entity_field_initializer, generate_entity_focus, render_standard_field,
};

pub struct FilePickerCodeGenerator;

const IMPORTS: &[ImportItem] = &[
    ImportItem::path("gpui_form::runtime::file_picker::FilePicker"),
    ImportItem::path("gpui_form::runtime::file_picker::FilePickerEvent"),
    ImportItem::path("gpui_form::runtime::file_picker::FilePickerState"),
];

impl FieldCodeGenerator for FilePickerCodeGenerator {
    fn generate_imports(&self, _field: &FieldVariant) -> Vec<ImportItem> {
        IMPORTS.to_vec()
    }

    fn generate_cx_new_call(
        &self,
        field: &ResolvedField<'_>,
        component: &GpuiFormShape,
    ) -> Option<TokenStream> {
        Some(generate_entity_creation(field, component))
    }

    fn generate_post_subscription_initialization(
        &self,
        field: &ResolvedField<'_>,
        _component: &GpuiFormShape,
    ) -> Option<TokenStream> {
        let field_var_name_ident = field.field_ident_with_behaviour();
        let field_name_ident = field.field_ident();

        Some(quote! {
            if let Some(value) = current_data.#field_name_ident.as_ref() {
                #field_var_name_ident.update(cx, |state, cx| {
                    state.set_path(value.clone(), window, cx);
                });
            }
        })
    }

    fn generate_field_initializers(
        &self,
        field: &ResolvedField<'_>,
        _component: &GpuiFormShape,
    ) -> Option<TokenStream> {
        Some(generate_entity_field_initializer(field))
    }

    fn generate_render_child(
        &self,
        field: &ResolvedField<'_>,
        component: &GpuiFormShape,
    ) -> TokenStream {
        let field_in_struct_name_ident = field.field_ident_with_behaviour();

        render_standard_field(
            field,
            component,
            quote! {
                FilePicker::new(&self.fields.#field_in_struct_name_ident)
                    .cleanable(true)
            },
        )
    }

    fn generate_focusable_cycle(
        &self,
        field: &ResolvedField<'_>,
        _component: &GpuiFormShape,
    ) -> Option<TokenStream> {
        Some(generate_entity_focus(field))
    }

    fn generate_subscription(
        &self,
        field: &ResolvedField<'_>,
        _component: &GpuiFormShape,
    ) -> Option<GeneratedSubscription> {
        let field_var_name_ident = field.field_ident_with_behaviour();
        let event_handler_fn_name_ident = field.event_handler_ident("file_picker_event");

        let calls = vec![
            quote! { cx.subscribe_in(&#field_var_name_ident, window, Self::#event_handler_fn_name_ident) },
        ];

        let field_name_ident = field.field_ident();

        let handler = quote! {
            fn #event_handler_fn_name_ident(
                &mut self,
                _this: &Entity<FilePickerState>,
                event: &FilePickerEvent,
                _: &mut Window,
                _: &mut Context<Self>,
            ) {
                match event {
                    FilePickerEvent::Change(paths) => {
                        self.current_data.#field_name_ident = paths.first().cloned();
                    }
                    FilePickerEvent::Cancel | FilePickerEvent::Error(_) => {}
                }
            }
        };

        Some(GeneratedSubscription {
            calls,
            handlers: vec![handler],
        })
    }
}
