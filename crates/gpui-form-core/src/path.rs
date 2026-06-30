//! Pure, GPUI-free typed field paths for naming form fields.
//!
//! [`FieldPath`] is a headless primitive: it owns no GPUI types, no entities,
//! and no UI runtime state, and it has no `serde` dependency of its own. It is
//! an ordered sequence of static string segments naming a field, so that every
//! consumer of a form (validation, dirty tracking, focus management,
//! analytics, schema export) can share one typed way to refer to fields
//! instead of ad-hoc strings.
//!
//! # Scope
//!
//! This is the core of backlog feature #8 ("Typed field paths and field
//! IDs"). It ships **FLAT v1 only**: a path is a list of static segments,
//! typically a single field name. Typed nested-path and list-item-path
//! constructors arrive with backlog feature #2 ("Nested forms") and feature
//! #3 ("Repeated fields"). Hand-built multi-segment paths via
//! [`FieldPath::new(&["a", "b"])`](FieldPath::new) work today; typed
//! composition of paths is later.
//!
//! This is the shared naming foundation for future backlog features:
//! field-level validation (#6), field-level diff/delta reporting (#9),
//! schema export (#14), and nested/list paths (#2/#3). When combined with the
//! `serde` feature on the derive crate, the generated `<Name>FormPath` types
//! (which wrap this primitive) may carry a serialization story; the core
//! primitive itself stays serde-free and unconditional, mirroring how
//! [`crate::state::FormState`] handles serialization.

/// A pure, headless, GPUI-free typed path to a form field.
///
/// A `FieldPath` is an ordered sequence of static string segments naming a
/// field. For FLAT v1 forms it is typically a single segment — the field's
/// own name. Multi-segment paths can still be built by hand via
/// [`FieldPath::new`]; typed constructors for nested and list-item paths are
/// added by backlog features #2 and #3.
///
/// The segments are stored as a cheap-to-clone `Box<[&'static str]>`. Clones
/// share the same allocation shape and never perform per-segment heap work
/// beyond the box itself, so handing a path to validation, dirty tracking, or
/// analytics code is cheap.
///
/// `FieldPath` is the shared naming foundation for field-level validation
/// (#6), field-level diff (#9), schema export (#14), and nested/list paths
/// (#2/#3). It is intentionally free of GPUI and `serde` so any crate — core,
/// derive, or downstream consumer — can rely on it without pulling extra
/// dependencies.
///
/// # Trait impls
///
/// `FieldPath` implements [`Clone`], [`Debug`], [`Eq`] / [`PartialEq`],
/// [`Hash`], and [`core::fmt::Display`]. Equality and hashing are by segment
/// sequence (same segments in the same order). [`core::fmt::Display`] renders
/// the segments joined by `.` (for example `address.city`); an empty path
/// renders as the empty string `""`.
///
/// # Example
///
/// ```
/// use gpui_form_core::FieldPath;
/// use std::collections::HashSet;
///
/// let city = FieldPath::new(&["address", "city"]);
/// assert_eq!(city.segments(), &["address", "city"]);
/// assert_eq!(city.to_string(), "address.city");
///
/// let flat = FieldPath::new(&["name"]);
/// assert!(!flat.is_empty());
/// assert_eq!(flat.to_string(), "name");
///
/// // Equality and hashing are by segment sequence.
/// let mut set = HashSet::new();
/// set.insert(city.clone());
/// assert!(set.contains(&FieldPath::new(&["address", "city"])));
/// ```
pub struct FieldPath {
    segments: Box<[&'static str]>,
}

impl FieldPath {
    /// Create a new path from a slice of static string segments.
    ///
    /// The segments are copied into an owned `Box<[&'static str]>`, so the
    /// caller's slice does not need to outlive the path. An empty slice
    /// produces an empty path ([`FieldPath::is_empty`] returns `true` and
    /// [`core::fmt::Display`] renders `""`).
    ///
    /// # Example
    ///
    /// ```
    /// use gpui_form_core::FieldPath;
    ///
    /// let empty = FieldPath::new(&[]);
    /// assert!(empty.is_empty());
    /// assert_eq!(empty.to_string(), "");
    /// ```
    pub fn new(segments: &[&'static str]) -> Self {
        Self {
            segments: segments.into(),
        }
    }

    /// Borrow the ordered segment slice.
    ///
    /// The returned slice preserves segment order. For a FLAT v1 path this is
    /// typically a single-element slice like `&["name"]`.
    pub fn segments(&self) -> &[&'static str] {
        &self.segments
    }

    /// Whether the path has zero segments.
    ///
    /// An empty path renders as `""` via [`core::fmt::Display`].
    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }
}

impl Clone for FieldPath {
    fn clone(&self) -> Self {
        Self {
            segments: self.segments.clone(),
        }
    }
}

impl core::fmt::Debug for FieldPath {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_list().entries(self.segments.iter()).finish()
    }
}

impl PartialEq for FieldPath {
    fn eq(&self, other: &Self) -> bool {
        self.segments == other.segments
    }
}

impl Eq for FieldPath {}

impl core::hash::Hash for FieldPath {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.segments.hash(state);
    }
}

impl core::fmt::Display for FieldPath {
    /// Renders the segments joined by `.`. An empty path renders as `""`.
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        for (i, segment) in self.segments.iter().enumerate() {
            if i > 0 {
                f.write_str(".")?;
            }
            f.write_str(segment)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{HashMap, HashSet};

    #[test]
    fn new_round_trips_segments() {
        let path = FieldPath::new(&["name"]);
        assert_eq!(path.segments(), &["name"]);
    }

    #[test]
    fn single_segment_path() {
        let path = FieldPath::new(&["email"]);
        assert_eq!(path.segments(), &["email"]);
        assert!(!path.is_empty());
    }

    #[test]
    fn multi_segment_path_preserves_order() {
        let path = FieldPath::new(&["address", "city", "zip"]);
        assert_eq!(path.segments(), &["address", "city", "zip"]);
        assert!(!path.is_empty());
    }

    #[test]
    fn empty_path_is_empty_and_renders_blank() {
        let empty = FieldPath::new(&[]);
        assert!(empty.is_empty());
        assert!(empty.segments().is_empty());
        assert_eq!(empty.to_string(), "");
    }

    #[test]
    fn display_dotted_join_single_segment() {
        let path = FieldPath::new(&["name"]);
        assert_eq!(path.to_string(), "name");
    }

    #[test]
    fn display_dotted_join_multi_segment() {
        let path = FieldPath::new(&["address", "city"]);
        assert_eq!(path.to_string(), "address.city");
    }

    #[test]
    fn eq_is_by_segment_sequence() {
        let a = FieldPath::new(&["address", "city"]);
        let b = FieldPath::new(&["address", "city"]);
        let c = FieldPath::new(&["address", "zip"]);

        assert_eq!(a, b);
        assert_ne!(a, c);
        // Order matters: different order is a different path.
        assert_ne!(FieldPath::new(&["city", "address"]), a);
    }

    #[test]
    fn hash_by_segment_sequence_in_set() {
        let mut set = HashSet::new();
        set.insert(FieldPath::new(&["address", "city"]));

        // Same segments hash-collide into the same entry.
        assert!(set.contains(&FieldPath::new(&["address", "city"])));
        // Different segments do not.
        assert!(!set.contains(&FieldPath::new(&["address", "zip"])));
    }

    #[test]
    fn hash_by_segment_sequence_in_map_key() {
        let mut map = HashMap::new();
        map.insert(FieldPath::new(&["name"]), 1u32);
        map.insert(FieldPath::new(&["email"]), 2u32);

        assert_eq!(map.get(&FieldPath::new(&["name"])), Some(&1));
        assert_eq!(map.get(&FieldPath::new(&["email"])), Some(&2));
        assert_eq!(map.get(&FieldPath::new(&["missing"])), None);
    }

    #[test]
    fn clone_is_independent_of_original() {
        let original = FieldPath::new(&["name"]);
        let copy = original.clone();

        // Equal in value...
        assert_eq!(original, copy);
        // ...and distinct objects (cloning does not alias internals in a way
        // observable through the public API).
        assert_eq!(original.segments(), copy.segments());
    }

    #[test]
    fn debug_renders_segment_list() {
        let path = FieldPath::new(&["address", "city"]);
        let s = format!("{:?}", path);
        assert_eq!(s, r#"["address", "city"]"#);
    }

    #[test]
    fn empty_segments_empty_path() {
        // Zero segments -> empty, even though the constructor accepted a slice.
        let path = FieldPath::new(&[]);
        assert!(path.is_empty());
        assert_eq!(path.segments(), &[] as &[&str]);
    }
}
