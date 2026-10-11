//! Milestone M3 trybuild pass fixture (hazard 10): the nested + list
//! attribute surface compiles end to end — a nested form referenced by the
//! `<Name>FormValueHolder` naming convention, a scalar-item list with
//! `each(...)` element validators, and the composed path/error surfaces.
//!
//! No `koruma(fluent)` here, so no trybuild locale seeding is needed.

use gpui_form::core::FormFieldErrors;
use gpui_form_derive::GpuiForm;
use koruma_collection::collection::{LenValidation, NonEmptyValidation};

#[derive(Clone, Debug, Default, GpuiForm, koruma::Koruma)]
pub struct Address {
    #[gpui_form(component(input))]
    #[koruma(NonEmptyValidation::<_>::builder())]
    pub city: String,

    #[gpui_form(component(input))]
    pub zip: Option<String>,
}

#[derive(Clone, Debug, Default, GpuiForm, koruma::Koruma)]
pub struct PostEditor {
    #[gpui_form(component(input))]
    #[koruma(NonEmptyValidation::<_>::builder())]
    pub title: String,

    #[gpui_form(component(list(item = input)))]
    #[koruma(each(LenValidation::<_>::builder().min(2).max(32)))]
    pub tags: Vec<String>,

    #[gpui_form(component(nested))]
    #[koruma(nested)]
    pub address: Address,
}

fn main() {
    // Holder composition: the nested field stores the nested form's holder.
    let holder = PostEditorFormValueHolder::default();
    let _: AddressFormValueHolder = holder.address;

    // List mutation API.
    let mut holder = PostEditorFormValueHolder::default();
    holder.add_tags("gpui".to_string());
    holder.remove_tags(0);
    let _: &[String] = holder.tags_items();

    // Typed paths: roots, typed item indices, and composition.
    let root = PostEditorFormPath::address_root();
    let city = root.clone().join(&AddressFormPath::city());
    assert_eq!(city.to_string(), "address.city");
    assert_eq!(PostEditorFormPath::tags(2).to_string(), "tags.2");

    // Typed error access over composed paths.
    let errors = holder.validate_fields();
    let _: Option<PostEditorFormFieldError<'_>> = errors.error_for(&root);
    let _: Option<AddressFormFieldErrors> = errors.address_errors();
    let _: Option<&PostEditorFormValueHolderTagsElementKorumaValidationError> =
        errors.tags_element_error(0);
    fn assert_generic<E: FormFieldErrors>(errors: &E) -> Vec<String> {
        errors
            .failing_paths()
            .iter()
            .map(|path| path.to_string())
            .collect()
    }
    let _ = assert_generic(&errors);

    // The M1 field-level state facade composes over the nested root.
    let mut state = PostEditorFormState::new(holder);
    state.touch(&PostEditorFormPath::address_root());
    assert!(state.is_touched(&PostEditorFormPath::address_root()));
}
