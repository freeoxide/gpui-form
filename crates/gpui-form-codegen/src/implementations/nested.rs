use crate::components::*;
use proc_macro2::TokenStream;

/// Milestone M3 (backlog #2): `component(nested)` is value-only in v1 — the
/// holder stores the nested form's own generated value holder, so this
/// component emits no `<Name>FormFields` entity field and no
/// `<Name>FormComponents` constructor (the same shape as checkbox/switch).
/// Rendering the nested group is application- or prototyping-authored.
impl super::ComponentLayout for NestedComponent {
    fn field_tokens(
        &self,
        _field_structure_tokens: &mut TokenStream,
        _field_base_declarations_tokens: &mut TokenStream,
    ) {
        let FieldInformation::<NestedOptions> {
            options: _,
            name: _,
            r#type: _,
        } = &self.0;
    }
}
