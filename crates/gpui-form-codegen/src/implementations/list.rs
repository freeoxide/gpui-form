use crate::components::*;
use proc_macro2::TokenStream;

/// Milestone M3 (backlog #3): `component(list(item = ...))` is value-only in
/// v1 — the holder stores the plain item `Vec` plus generated
/// `add_<field>` / `remove_<field>` / `<field>_items` operations, so this
/// component emits no `<Name>FormFields` entity field and no
/// `<Name>FormComponents` constructor (the same shape as checkbox/switch).
/// Per-item entity management stays application-authored ([D5]/R17).
impl super::ComponentLayout for ListComponent {
    fn field_tokens(
        &self,
        _field_structure_tokens: &mut TokenStream,
        _field_base_declarations_tokens: &mut TokenStream,
    ) {
        let FieldInformation::<ListOptions> {
            options: _,
            name: _,
            r#type: _,
        } = &self.0;
    }
}
