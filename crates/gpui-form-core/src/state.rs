//! Pure, GPUI-free form-state helpers for dirty tracking, reset, and diffing.
//!
//! [`FormState<H>`] wraps a generated `...FormValueHolder` (or any cloneable
//! value) and snapshots a *baseline* copy at construction time so the caller
//! can answer "did the user edit this form?" without keeping a second copy by
//! hand. The type is intentionally framework-agnostic: it owns no GPUI types,
//! no entities, and no UI runtime state (open menus, scroll positions, or
//! `InfiniteSelectState` snapshots). It stores holder *data* only.
//!
//! # Scope
//!
//! This is the core of backlog feature #1 ("Form-state persistence and dirty
//! tracking"). It ships **boolean-level** dirty/diff only. Structured,
//! field-level patch and delta reporting is backlog feature #9; per-field
//! serde passthrough (rename/skip) is backlog feature #15. Undo/redo is out
//! of scope for this feature.
//!
//! When combined with the `serde` feature on the derive crate, the holder can
//! be serialized for persistence and later deserialized into a fresh
//! [`FormState`]. Note that a holder carrying `#[gpui_form(skip)]` fields
//! cannot fully reconstruct the source struct via `into_original` — this is
//! the same limitation surfaced by the existing `has_skipped_fields` behavior
//! and is documented, not fixed, by this feature.

/// Snapshot-based form state for dirty tracking, reset, and diffing.
///
/// Holds two copies of a holder value `H`:
/// - `baseline`: the value captured at [`FormState::new`] (or last
///   [`FormState::sync_baseline`]); the "saved" reference point.
/// - `current`: the live, possibly-edited value the UI mutates.
///
/// Both fields are private; callers reach them through the accessors so the
/// two copies can never drift out of the baseline/current pairing by accident.
///
/// # Trait bounds
///
/// - Construction and the mutating helpers ([`FormState::reset_to_baseline`],
///   [`FormState::sync_baseline`]) require `H: Clone`, since they copy the
///   holder between the two slots.
/// - [`FormState::is_dirty`] and [`FormState::diff_against`] require
///   `H: PartialEq`, since dirty tracking is a value-equality comparison
///   against the baseline.
///
/// # Example
///
/// ```
/// use gpui_form_core::FormState;
///
/// #[derive(Clone, Debug, PartialEq, Default)]
/// struct Holder { name: String }
///
/// let mut state = FormState::new(Holder { name: "a".into() });
/// assert!(!state.is_dirty());
///
/// state.current_mut().name = "b".into();
/// assert!(state.is_dirty());
///
/// state.reset_to_baseline();
/// assert!(!state.is_dirty());
/// ```
pub struct FormState<H> {
    baseline: H,
    current: H,
}

impl<H: Clone> FormState<H> {
    /// Create a new state whose `baseline` and `current` both start as a clone
    /// of `holder`.
    ///
    /// The passed-in `holder` becomes the initial `current` value; `baseline`
    /// is a fresh clone of it. A brand-new state is therefore not dirty.
    pub fn new(holder: H) -> Self {
        let baseline = holder.clone();
        Self {
            baseline,
            current: holder,
        }
    }

    /// Borrow the live (`current`) value.
    pub fn current(&self) -> &H {
        &self.current
    }

    /// Borrow the saved (`baseline`) value.
    pub fn baseline(&self) -> &H {
        &self.baseline
    }

    /// Mutably borrow the live (`current`) value so the UI can edit it in
    /// place. Mutating through this reference is what makes
    /// [`FormState::is_dirty`] subsequently return `true`.
    pub fn current_mut(&mut self) -> &mut H {
        &mut self.current
    }

    /// Overwrite `current` with `holder`, dropping the previous live value.
    /// The `baseline` is untouched, so a replace with a value equal to the
    /// baseline clears dirty state.
    pub fn replace_current(&mut self, holder: H) {
        self.current = holder;
    }

    /// Consume the state and return the live (`current`) value, dropping the
    /// `baseline`.
    pub fn into_current(self) -> H {
        self.current
    }

    /// Reset `current` back to a clone of `baseline`, discarding the user's
    /// edits. After this call [`FormState::is_dirty`] returns `false`.
    pub fn reset_to_baseline(&mut self) {
        self.current = self.baseline.clone();
    }

    /// Snapshots `current` into `baseline` — call this after a successful save
    /// to mark the form clean. After this call [`FormState::is_dirty`]
    /// returns `false`.
    pub fn sync_baseline(&mut self) {
        self.baseline = self.current.clone();
    }
}

impl<H: PartialEq> FormState<H> {
    /// Whether the live value differs from the saved baseline.
    ///
    /// This is a whole-value equality check (`current != baseline`); it does
    /// not report *which* fields changed. Field-level diff is backlog
    /// feature #9.
    pub fn is_dirty(&self) -> bool {
        self.current != self.baseline
    }

    /// Whether `current` differs from an arbitrary `other` holder.
    ///
    /// Useful for comparing against an externally-provided value (for example,
    /// a freshly deserialized holder) without disturbing the stored baseline.
    /// Returns `true` when `current != other`.
    pub fn diff_against(&self, other: &H) -> bool {
        &self.current != other
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq, Default)]
    struct Holder {
        name: String,
        age: u32,
    }

    #[test]
    fn new_state_is_not_dirty() {
        let state = FormState::new(Holder {
            name: "a".into(),
            age: 1,
        });
        assert!(!state.is_dirty());
    }

    #[test]
    fn mutate_via_current_mut_marks_dirty() {
        let mut state = FormState::new(Holder {
            name: "a".into(),
            age: 1,
        });
        assert!(!state.is_dirty());

        state.current_mut().age = 2;
        assert!(state.is_dirty());
    }

    #[test]
    fn reset_to_baseline_clears_dirty() {
        let mut state = FormState::new(Holder {
            name: "a".into(),
            age: 1,
        });
        state.current_mut().age = 99;
        assert!(state.is_dirty());

        state.reset_to_baseline();
        assert!(!state.is_dirty());
        assert_eq!(state.current().age, 1);
        assert_eq!(state.baseline().age, 1);
    }

    #[test]
    fn sync_baseline_marks_clean_then_mutation_redirties() {
        let mut state = FormState::new(Holder {
            name: "a".into(),
            age: 1,
        });
        state.current_mut().age = 5;
        assert!(state.is_dirty());

        // Simulate a successful save.
        state.sync_baseline();
        assert!(!state.is_dirty());
        assert_eq!(state.baseline().age, 5);

        // A further edit re-dirties; a second sync clears it again.
        state.current_mut().age = 6;
        assert!(state.is_dirty());
        state.sync_baseline();
        assert!(!state.is_dirty());
    }

    #[test]
    fn diff_against_detects_difference_and_equality() {
        let state = FormState::new(Holder {
            name: "a".into(),
            age: 1,
        });
        // Different holder.
        let other = Holder {
            name: "a".into(),
            age: 2,
        };
        assert!(state.diff_against(&other));

        // Equal holder.
        let same = Holder {
            name: "a".into(),
            age: 1,
        };
        assert!(!state.diff_against(&same));
    }

    #[test]
    fn replace_current_overwrites_live_value() {
        let mut state = FormState::new(Holder {
            name: "a".into(),
            age: 1,
        });
        // Replace with a value equal to baseline -> not dirty.
        state.replace_current(Holder {
            name: "a".into(),
            age: 1,
        });
        assert!(!state.is_dirty());

        // Replace with a different value -> dirty.
        state.replace_current(Holder {
            name: "b".into(),
            age: 9,
        });
        assert!(state.is_dirty());
        assert_eq!(state.current().name, "b");
        assert_eq!(state.current().age, 9);
        // Baseline is unaffected by replace.
        assert_eq!(state.baseline().name, "a");
    }

    #[test]
    fn into_current_returns_live_value_and_drops_baseline() {
        let state = FormState::new(Holder {
            name: "a".into(),
            age: 1,
        });
        let live = state.into_current();
        assert_eq!(live.name, "a");
        assert_eq!(live.age, 1);
    }

    #[test]
    fn baseline_and_current_accessors_are_distinct_after_edit() {
        let mut state = FormState::new(Holder {
            name: "a".into(),
            age: 1,
        });
        state.current_mut().name = "z".into();
        assert_eq!(state.baseline().name, "a");
        assert_eq!(state.current().name, "z");
    }
}
