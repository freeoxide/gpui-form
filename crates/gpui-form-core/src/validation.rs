//! GPUI-free validation contracts: the form-agnostic [`FormFieldErrors`]
//! error-map trait, the snapshot-compared [`ValidationCache`] outcome cache,
//! and the [`required_when`] conditional-required predicate.

use crate::FieldPath;

/// Form-agnostic view of a per-form field-error map, implemented by the
/// generated `<Name>FormFieldErrors` for every koruma form.
pub trait FormFieldErrors {
    /// Whether no field has errors (`true` == validation would succeed).
    fn is_empty(&self) -> bool;

    /// Failing field paths in declaration order.
    fn failing_paths(&self) -> Vec<FieldPath>;
}

/// Pure conditional-required predicate that generated `required_if` koruma
/// validators delegate to: the value passes when the controlling field does
/// not require it (`requires == false`) or when it is present.
pub const fn required_when(requires: bool, present: bool) -> bool {
    !requires || present
}

/// Outcome cache for validation results, fresh iff `current` equals the
/// validated holder snapshot; a mutated holder simply stops comparing
/// equal, so no invalidation discipline is required.
#[derive(Clone, Debug)]
pub struct ValidationCache<H, O> {
    stored: Option<(H, O)>,
}

impl<H: PartialEq, O> ValidationCache<H, O> {
    pub const fn new() -> Self {
        Self { stored: None }
    }

    /// Borrow the cached outcome iff `current` equals the validated snapshot;
    /// `None` when the cache is empty or the holder changed (holders with
    /// non-reflexive `PartialEq` values report stale, never wrong data).
    pub fn get(&self, current: &H) -> Option<&O> {
        match &self.stored {
            Some((snapshot, outcome)) if snapshot == current => Some(outcome),
            _ => None,
        }
    }

    /// Borrow the last stored outcome without a freshness check, staying
    /// exact even when equality would report stale.
    pub fn stored(&self) -> Option<&O> {
        self.stored.as_ref().map(|(_, outcome)| outcome)
    }

    /// Record the outcome for a holder snapshot; store the value actually
    /// validated (typically a clone of `current`).
    pub fn store(&mut self, validated: H, outcome: O) {
        self.stored = Some((validated, outcome));
    }

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

        let same = validated;
        assert_eq!(cache.get(&same), Some(&false));

        let edited = Holder {
            name: "a".into(),
            age: 2,
        };
        assert_eq!(cache.get(&edited), None);
    }

    #[test]
    fn mutation_through_arbitrary_routes_never_serves_a_stale_outcome() {
        let mut cache = ValidationCache::new();
        let mut current = Holder::sample("a");
        cache.store(current.clone(), true);
        assert_eq!(cache.get(&current), Some(&true));

        current.age = 7;
        assert_eq!(cache.get(&current), None);

        current.age = 1;
        assert_eq!(cache.get(&current), Some(&true));

        let replaced = Holder::sample("z");
        cache.store(replaced.clone(), false);
        assert_eq!(cache.get(&replaced), Some(&false));
        assert_eq!(cache.get(&current), None);
    }

    #[test]
    fn invalidate_clears_even_a_matching_snapshot() {
        let mut cache = ValidationCache::new();
        let holder = Holder::sample("a");
        cache.store(holder.clone(), false);
        assert!(cache.get(&holder).is_some());

        cache.invalidate();
        assert_eq!(cache.get(&holder), None, "invalidate beats equality");

        cache.store(holder.clone(), true);
        assert_eq!(cache.get(&holder), Some(&true));
    }

    #[test]
    fn stored_returns_the_last_outcome_without_a_freshness_check() {
        let mut cache: ValidationCache<f64, bool> = ValidationCache::new();
        cache.store(f64::NAN, true);
        assert_eq!(cache.get(&f64::NAN), None, "NaN never compares equal");
        assert_eq!(cache.stored(), Some(&true), "stored() skips the compare");

        cache.invalidate();
        assert_eq!(cache.stored(), None);
    }

    #[test]
    fn outcome_type_is_generic() {
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
