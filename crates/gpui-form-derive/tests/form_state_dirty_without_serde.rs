//! Regression test for the audit finding about PartialEq gating.
//!
//! `gpui_form_core::FormState::is_dirty` (and `diff_against`) require
//! `H: PartialEq` and are re-exported UNCONDITIONALLY by the facade. The
//! generated `...FormValueHolder` must therefore implement `PartialEq` on
//! default features too — otherwise `is_dirty` would not compile without the
//! `serde` feature.
//!
//! This file is deliberately NOT gated by `#![cfg(feature = "serde")]`. It must
//! compile under DEFAULT features, proving the holder now derives `PartialEq`
//! unconditionally (the derive emits it regardless of the `serde` feature;
//! only `Serialize`/`Deserialize` remain serde-gated). It contains no serde
//! usage whatsoever.

use gpui_form::FormState;
use gpui_form_derive::GpuiForm;

/// Minimal form with a single `input` String field. The generated holder is
/// `DirtyCheckFormValueHolder` with a `pub` field, matching the patterns in
/// `serde_round_trip.rs` and `form_state_integration.rs`.
#[derive(GpuiForm)]
struct DirtyCheckForm {
    #[gpui_form(component(input))]
    label: String,
}

#[test]
fn is_dirty_compiles_and_behaves_without_serde_feature() {
    // Source -> holder (holder stores every field as Option<T>).
    let holder = DirtyCheckFormFormValueHolder::from(DirtyCheckForm {
        label: "baseline".to_string(),
    });

    // A fresh state mirrors its baseline, so it is not dirty. This line is the
    // crux of the regression: it only compiles because the holder derives
    // `PartialEq` even though the `serde` feature is OFF.
    let mut state = FormState::new(holder);
    assert!(!state.is_dirty());

    // Mutate via current_mut() -> state diverges from baseline -> dirty.
    state.current_mut().label = Some("edited".to_string());
    assert!(state.is_dirty());

    // Reset restores the baseline -> not dirty again.
    state.reset_to_baseline();
    assert!(!state.is_dirty());
}
