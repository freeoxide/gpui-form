//! Pure, GPUI-free generic contracts and state for validation.
//!
//! Three surfaces live here:
//!
//! - [`FormFieldErrors`]: the form-agnostic contract the derive's generated
//!   `<Name>FormFieldErrors` maps implement, so GPUI glue, tests, and
//!   application code can be generic over any form's error map using the
//!   shared [`crate::FieldPath`] primitive.
//! - [`ValidationCache`]: the outcome cache behind a live `is_valid`
//!   (validation-state model, M2), kept honest by *snapshot comparison*
//!   rather than staleness flags.
//! - [`required_when`]: the pure conditional-required predicate (M4,
//!   backlog #13) every generated `required_if` koruma validator delegates
//!   to, so the rule integrates with the M1 typed error surface instead of
//!   living as display-only metadata.

use crate::FieldPath;

/// Form-agnostic view of a per-form field-error map.
///
/// Implemented by the generated `<Name>FormFieldErrors` for every koruma
/// form, so generic code (render all failing fields, attach errors to
/// path-keyed UI state) can consume any form's errors through the shared
/// [`FieldPath`] primitive without knowing the form's typed accessors.
///
/// `failing_paths` returns failing field paths in declaration order.
pub trait FormFieldErrors {
    /// Whether no field has errors (`true` == validation would succeed).
    fn is_empty(&self) -> bool;

    /// Failing field paths in declaration order.
    fn failing_paths(&self) -> Vec<FieldPath>;
}

/// Pure conditional-required predicate (M4, backlog #13).
///
/// A `required_if = <field>` rule desugars at derive time to a generated
/// koruma validator on the holder field (never to an expression language —
/// feature-parity plan R19); this function is the single source of truth
/// that validator delegates to. Because the rule runs *inside* koruma's
/// `validate()`, a violation surfaces through the M1 typed error surface
/// (`validate_fields()`, `error_for(<Name>FormPath::field())`,
/// `issue_keys()` reporting `"validation.required"`) instead of living as
/// display-only metadata.
///
/// Semantics: a value passes when the controlling field does **not** require
/// it (`requires == false`), or when it is present (`present == true`).
///
/// # Example
///
/// ```
/// use gpui_form_core::required_when;
///
/// // Not required: passes regardless of presence.
/// assert!(required_when(false, false));
/// assert!(required_when(false, true));
///
/// // Required: only presence passes.
/// assert!(!required_when(true, false));
/// assert!(required_when(true, true));
/// ```
pub const fn required_when(requires: bool, present: bool) -> bool {
    !requires || present
}

/// Outcome cache for validation, keyed by value equality.
///
/// The freshness rule is **snapshot-compare** ([D6] of the feature-parity
/// plan): the cache stores the holder snapshot that was validated next to
/// the outcome, and a read is fresh iff `current == snapshot`. That makes it
/// sound under *every* mutation route — `current_mut()` escapes,
/// `replace_current`, resets — with zero invalidation discipline: a mutated
/// holder simply stops comparing equal. The cost is one holder clone per
/// `store` and one `PartialEq` per `get` (form-scale data, not per-field
/// allocation).
///
/// `O` is typically a generated `<Name>FormFieldErrors` map (or plain `bool`
/// for non-koruma validity). The cache is transient by construction: it
/// carries no serde surface and never lives on the value holder.
///
/// # Example
///
/// ```
/// use gpui_form_core::ValidationCache;
///
/// #[derive(Clone, Debug, PartialEq)]
/// struct Holder { name: String }
///
/// let mut cache: ValidationCache<Holder, bool> = ValidationCache::new();
/// let a = Holder { name: "a".into() };
///
/// assert_eq!(cache.get(&a), None, "nothing cached yet");
///
/// cache.store(a.clone(), false);
/// assert_eq!(cache.get(&a), Some(&false), "fresh while current == snapshot");
///
/// let b = Holder { name: "b".into() };
/// assert_eq!(cache.get(&b), None, "mutated holder no longer compares equal");
///
/// cache.invalidate();
/// assert_eq!(cache.get(&a), None, "manual invalidation clears everything");
/// ```
#[derive(Clone, Debug)]
pub struct ValidationCache<H, O> {
    stored: Option<(H, O)>,
}

impl<H: PartialEq, O> ValidationCache<H, O> {
    /// Create an empty cache (no snapshot, no outcome).
    pub const fn new() -> Self {
        Self { stored: None }
    }

    /// Borrow the cached outcome iff `current` equals the validated
    /// snapshot; `None` when the cache is empty or the holder changed.
    ///
    /// One `PartialEq` comparison — cheap enough to call from a render path.
    /// Note the equality caveat: for holders containing NaN (or other
    /// non-reflexive `PartialEq` values) a stored clone may not compare
    /// equal, in which case this conservatively reports stale — never wrong,
    /// just uncached.
    pub fn get(&self, current: &H) -> Option<&O> {
        match &self.stored {
            Some((snapshot, outcome)) if snapshot == current => Some(outcome),
            _ => None,
        }
    }

    /// Borrow the last stored outcome regardless of freshness.
    ///
    /// Companion to [`store`](Self::store) for the "validate and hand back
    /// what was just stored" read: unlike [`get`](Self::get) it does not
    /// re-compare, so it stays exact even for non-reflexive `PartialEq`
    /// field values (a holder containing NaN never compares equal to its
    /// own clone).
    pub fn stored(&self) -> Option<&O> {
        self.stored.as_ref().map(|(_, outcome)| outcome)
    }

    /// Record the outcome for a holder snapshot. Callers should store the
    /// same value they validated (typically a clone of `current`).
    pub fn store(&mut self, validated: H, outcome: O) {
        self.stored = Some((validated, outcome));
    }

    /// Drop the cached snapshot and outcome. Facade mutators call this on
    /// reset-style value changes so a reverted-to-same-value form does not
    /// keep displaying the old verdict.
    pub fn invalidate(&mut self) {
        self.stored = None;
    }
}

impl<H, O> Default for ValidationCache<H, O> {
    fn default() -> Self {
        Self { stored: None }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PathSegment;

    #[test]
    fn required_when_truth_table_passes_everything_but_required_and_absent() {
        // The single failing combination: the controlling field makes the
        // value required AND the value is absent.
        assert!(required_when(false, false), "not required: absent is fine");
        assert!(required_when(false, true), "not required: present is fine");
        assert!(!required_when(true, false), "required + absent fails");
        assert!(required_when(true, true), "required + present passes");
    }

    struct ManualErrors {
        failing: Vec<&'static str>,
    }

    impl FormFieldErrors for ManualErrors {
        fn is_empty(&self) -> bool {
            self.failing.is_empty()
        }

        fn failing_paths(&self) -> Vec<FieldPath> {
            self.failing
                .iter()
                .map(|name| FieldPath::new(&[name]))
                .collect()
        }
    }

    #[test]
    fn manual_impl_satisfies_the_trait() {
        let empty = ManualErrors { failing: vec![] };
        assert!(empty.is_empty());
        assert!(empty.failing_paths().is_empty());

        let failing = ManualErrors {
            failing: vec!["username", "age"],
        };
        assert!(!failing.is_empty());
        let paths = failing.failing_paths();
        assert_eq!(paths[0].segments(), &[PathSegment::Name("username")]);
        assert_eq!(paths[1].to_string(), "age");

        fn assert_generic<E: FormFieldErrors>(errors: &E) -> bool {
            errors.is_empty()
        }
        assert!(!assert_generic(&failing));
    }

    #[derive(Clone, Debug, PartialEq)]
    struct Holder {
        name: String,
        age: u32,
    }

    impl Holder {
        fn sample(name: &str) -> Self {
            Self {
                name: name.into(),
                age: 1,
            }
        }
    }

    #[test]
    fn cache_starts_empty_and_default_matches_new() {
        let cache: ValidationCache<Holder, bool> = ValidationCache::new();
        let default: ValidationCache<Holder, bool> = ValidationCache::default();
        assert_eq!(cache.get(&Holder::sample("a")), None);
        assert_eq!(default.get(&Holder::sample("a")), None);
    }

    #[test]
    fn stored_outcome_is_fresh_only_for_the_validated_snapshot() {
        let mut cache = ValidationCache::new();
        let validated = Holder::sample("a");
        cache.store(validated.clone(), false);

        // Same value again: fresh.
        let same = validated;
        assert_eq!(cache.get(&same), Some(&false));

        // Different value: stale, even though only one field differs.
        let edited = Holder {
            name: "a".into(),
            age: 2,
        };
        assert_eq!(cache.get(&edited), None);
    }

    #[test]
    fn mutation_through_arbitrary_routes_never_serves_a_stale_outcome() {
        // The D6 rule: because freshness is `current == snapshot`, NO mutation
        // route can serve a stale outcome — there is no invalidation
        // discipline to forget. Simulate the routes by handing out `&mut H`.
        let mut cache = ValidationCache::new();
        let mut current = Holder::sample("a");
        cache.store(current.clone(), true);
        assert_eq!(cache.get(&current), Some(&true));

        // Route 1: `&mut` escape (FormState::current_mut shape).
        current.age = 7;
        assert_eq!(cache.get(&current), None);

        // Edit back to the validated value: fresh again — correctly, since
        // validation is a pure function of the value.
        current.age = 1;
        assert_eq!(cache.get(&current), Some(&true));

        // Route 2: whole-value replace.
        let replaced = Holder::sample("z");
        cache.store(replaced.clone(), false);
        assert_eq!(cache.get(&replaced), Some(&false));
        assert_eq!(cache.get(&current), None);
    }

    #[test]
    fn invalidate_clears_even_a_matching_snapshot() {
        // Reset semantics: a reset-to-baseline that lands on the previously
        // validated value must NOT resurrect the old verdict via equality.
        let mut cache = ValidationCache::new();
        let holder = Holder::sample("a");
        cache.store(holder.clone(), false);
        assert!(cache.get(&holder).is_some());

        cache.invalidate();
        assert_eq!(cache.get(&holder), None, "invalidate beats equality");

        // Re-store works after invalidation.
        cache.store(holder.clone(), true);
        assert_eq!(cache.get(&holder), Some(&true));
    }

    #[test]
    fn stored_returns_the_last_outcome_without_a_freshness_check() {
        // The exact-read companion to `get`: after `store` it always hands
        // back what was stored, even when equality would report stale
        // (NaN holders never compare equal to their own clones).
        let mut cache: ValidationCache<f64, bool> = ValidationCache::new();
        cache.store(f64::NAN, true);
        assert_eq!(cache.get(&f64::NAN), None, "NaN never compares equal");
        assert_eq!(cache.stored(), Some(&true), "stored() skips the compare");

        cache.invalidate();
        assert_eq!(cache.stored(), None);
    }

    #[test]
    fn outcome_type_is_generic() {
        // The cache is not tied to error maps; `bool` is the non-koruma
        // validity shape.
        let mut bool_cache: ValidationCache<Holder, bool> = ValidationCache::new();
        bool_cache.store(Holder::sample("a"), true);
        assert_eq!(bool_cache.get(&Holder::sample("a")), Some(&true));

        let mut map_cache: ValidationCache<Holder, ManualErrors> = ValidationCache::new();
        map_cache.store(
            Holder::sample("a"),
            ManualErrors {
                failing: vec!["name"],
            },
        );
        assert!(map_cache.get(&Holder::sample("a")).is_some());
    }
}
