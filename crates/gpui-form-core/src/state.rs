//! Snapshot-based dirty tracking: [`FormState`] pairs a baseline copy of a
//! holder value with the live value for reset and whole-value diffing.
//! Framework-agnostic: holder data only, no UI runtime state.

/// Snapshot-based form state: a `baseline` captured at construction (or
/// [`FormState::sync_baseline`]) and a live `current` the UI mutates, for
/// dirty tracking, reset, and whole-value diffing.
pub struct FormState<H> {
    baseline: H,
    current: H,
}

impl<H: Clone> FormState<H> {
    /// Create a state whose `baseline` and `current` both start as a clone
    /// of `holder`; a brand-new state is not dirty.
    pub fn new(holder: H) -> Self {
        let baseline = holder.clone();
        Self {
            baseline,
            current: holder,
        }
    }

    pub fn current(&self) -> &H {
        &self.current
    }

    pub fn baseline(&self) -> &H {
        &self.baseline
    }

    pub fn current_mut(&mut self) -> &mut H {
        &mut self.current
    }

    /// Overwrite `current` with `holder`; the `baseline` is untouched, so a
    /// replace with a value equal to the baseline clears dirty state.
    pub fn replace_current(&mut self, holder: H) {
        self.current = holder;
    }

    pub fn into_current(self) -> H {
        self.current
    }

    /// Reset `current` back to a clone of `baseline`, discarding the user's
    /// edits. After this call [`FormState::is_dirty`] returns `false`.
    pub fn reset_to_baseline(&mut self) {
        self.current = self.baseline.clone();
    }

    /// Snapshot `current` into `baseline` — call after a successful save to
    /// mark the form clean.
    pub fn sync_baseline(&mut self) {
        self.baseline = self.current.clone();
    }
}

impl<H: PartialEq> FormState<H> {
    /// Whether the live value differs from the saved baseline (whole-value
    /// equality; no field-level diff).
    pub fn is_dirty(&self) -> bool {
        self.current != self.baseline
    }

    /// Whether `current` differs from `other`, without disturbing the
    /// stored baseline.
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

        state.sync_baseline();
        assert!(!state.is_dirty());
        assert_eq!(state.baseline().age, 5);

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
        let other = Holder {
            name: "a".into(),
            age: 2,
        };
        assert!(state.diff_against(&other));

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
        state.replace_current(Holder {
            name: "a".into(),
            age: 1,
        });
        assert!(!state.is_dirty());

        state.replace_current(Holder {
            name: "b".into(),
            age: 9,
        });
        assert!(state.is_dirty());
        assert_eq!(state.current().name, "b");
        assert_eq!(state.current().age, 9);
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
