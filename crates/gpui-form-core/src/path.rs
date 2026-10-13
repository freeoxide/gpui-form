//! Pure, GPUI-free typed field paths for naming form fields.
//!
//! [`FieldPath`] is a headless primitive: it owns no GPUI types, no entities,
//! and no UI runtime state, and it has no `serde` dependency of its own. It is
//! an ordered sequence of typed segments naming a field, so that every
//! consumer of a form (validation, dirty tracking, focus management,
//! analytics, schema export) can share one typed way to refer to fields
//! instead of ad-hoc strings.
//!
//! # Scope
//!
//! This is the core of backlog feature #8 ("Typed field paths and field
//! IDs"). FLAT v1 shipped name-only segments; milestone M3 (backlog #2
//! "Nested forms" and #3 "Repeated fields") extended the segment
//! representation to the typed [`PathSegment`] enum — decision [D8] of the
//! feature-parity plan — so a list-item path such as `contacts.2.name` is one
//! uniform [`FieldPath`] with a typed index in the middle:
//!
//! ```
//! use gpui_form_core::{FieldPath, PathSegment};
//!
//! let path = FieldPath::from_segments(&[
//!     PathSegment::Name("contacts"),
//!     PathSegment::Index(2),
//!     PathSegment::Name("name"),
//! ]);
//! assert_eq!(path.to_string(), "contacts.2.name");
//! ```
//!
//! String formatting of indices (`format!("tags.{}", i)`) is rejected by
//! design: indices are typed, never re-merged from strings. Name-only
//! construction via [`FieldPath::new`] keeps working, and typed composition
//! builds on [`FieldPath::push_name`] / [`FieldPath::push_index`] /
//! [`FieldPath::join`].
//!
//! This is the shared naming foundation for field-level validation (#6),
//! field-level diff/delta reporting (#9), schema export (#14), and
//! nested/list paths (#2/#3). When combined with the `serde` feature on the
//! derive crate, the generated `<Name>FormPath` types (which wrap this
//! primitive) may carry a serialization story; the core primitive itself
//! stays serde-free and unconditional, mirroring how
//! [`crate::state::FormState`] handles serialization.

/// One typed segment of a [`FieldPath`]: a field name or a list-item index.
///
/// Introduced with milestone M3 as decision [D8] option A of the
/// feature-parity plan: repeated-field paths need a typed index segment
/// (`contacts.2`), and `usize` is not representable as a `&'static str`
/// without leaking or string formatting — both rejected. Flat name-only
/// paths render and compare exactly as before; only [`core::fmt::Debug`]
/// output changed shape (`[Name("address")]` instead of `["address"]`).
///
/// # Example
///
/// ```
/// use gpui_form_core::PathSegment;
///
/// assert_eq!(PathSegment::Name("tags").as_name(), Some("tags"));
/// assert_eq!(PathSegment::Name("tags").as_index(), None);
/// assert_eq!(PathSegment::Index(3).as_index(), Some(3));
/// assert_eq!(PathSegment::Index(3).to_string(), "3");
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum PathSegment {
    /// A named field segment (`address`, `city`, `tags`).
    Name(&'static str),
    /// A repeated-field item index (`contacts.2` names item 2).
    Index(usize),
}

impl PathSegment {
    /// The field name, when this segment is a [`PathSegment::Name`].
    pub const fn as_name(&self) -> Option<&'static str> {
        match self {
            Self::Name(name) => Some(*name),
            Self::Index(_) => None,
        }
    }

    /// The list-item index, when this segment is an [`PathSegment::Index`].
    pub const fn as_index(&self) -> Option<usize> {
        match self {
            Self::Name(_) => None,
            Self::Index(index) => Some(*index),
        }
    }
}

impl core::fmt::Display for PathSegment {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Name(name) => f.write_str(name),
            Self::Index(index) => write!(f, "{index}"),
        }
    }
}

/// A pure, headless, GPUI-free typed path to a form field.
///
/// A `FieldPath` is an ordered sequence of typed [`PathSegment`]s naming a
/// field. For flat forms it is typically a single [`PathSegment::Name`] —
/// the field's own name. Nested-field paths append the nested form's field
/// names (`address.city`), and repeated-field paths carry a typed
/// [`PathSegment::Index`] in the middle (`contacts.2.name`).
///
/// The segments are stored as a cheap-to-clone `Box<[PathSegment]>`. Clones
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
/// the segments joined by `.` with indices rendered as numbers (for example
/// `address.city` or `contacts.2.name`); an empty path renders as the empty
/// string `""`.
///
/// # Example
///
/// ```
/// use gpui_form_core::{FieldPath, PathSegment};
/// use std::collections::HashSet;
///
/// let city = FieldPath::new(&["address", "city"]);
/// assert_eq!(
///     city.segments(),
///     &[PathSegment::Name("address"), PathSegment::Name("city")]
/// );
/// assert_eq!(city.to_string(), "address.city");
///
/// let item = FieldPath::new(&["contacts"]).push_index(2).push_name("name");
/// assert_eq!(item.to_string(), "contacts.2.name");
///
/// // Equality and hashing are by segment sequence.
/// let mut set = HashSet::new();
/// set.insert(city.clone());
/// assert!(set.contains(&FieldPath::new(&["address", "city"])));
/// ```
pub struct FieldPath {
    segments: Box<[PathSegment]>,
}

impl FieldPath {
    /// Create a new path from a slice of static string segments.
    ///
    /// The names-only constructor: every segment becomes a
    /// [`PathSegment::Name`]. Use [`FieldPath::from_segments`] for paths that
    /// carry list-item indices. The segments are copied into an owned
    /// `Box<[PathSegment]>`, so the caller's slice does not need to outlive
    /// the path. An empty slice produces an empty path ([`FieldPath::is_empty`]
    /// returns `true` and [`core::fmt::Display`] renders `""`).
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
            segments: segments.iter().copied().map(PathSegment::Name).collect(),
        }
    }

    /// Create a new path from a slice of typed segments (names and indices).
    ///
    /// The M3 composition constructor: `list(item = ...)` fields build
    /// `[Name("tags"), Index(i)]` paths with it. The segments are copied into
    /// an owned box, exactly like [`FieldPath::new`].
    ///
    /// # Example
    ///
    /// ```
    /// use gpui_form_core::{FieldPath, PathSegment};
    ///
    /// let path = FieldPath::from_segments(&[
    ///     PathSegment::Name("tags"),
    ///     PathSegment::Index(4),
    /// ]);
    /// assert_eq!(path.to_string(), "tags.4");
    /// ```
    pub fn from_segments(segments: &[PathSegment]) -> Self {
        Self {
            segments: segments.into(),
        }
    }

    /// Borrow the ordered segment slice.
    ///
    /// The returned slice preserves segment order. For a flat form path this
    /// is typically a single-element slice like `&[PathSegment::Name("name")]`.
    pub fn segments(&self) -> &[PathSegment] {
        &self.segments
    }

    /// Whether the path has zero segments.
    ///
    /// An empty path renders as `""` via [`core::fmt::Display`].
    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }

    /// Consume the path, append a field-name segment, and return the result.
    ///
    /// The consuming appender behind generated nested-field path
    /// composition (`<Name>FormPath` emits one `<field>(self)` per nested
    /// field over this primitive).
    ///
    /// # Example
    ///
    /// ```
    /// use gpui_form_core::FieldPath;
    ///
    /// let path = FieldPath::new(&["address"]).push_name("city");
    /// assert_eq!(path.to_string(), "address.city");
    /// ```
    pub fn push_name(self, name: &'static str) -> Self {
        let mut segments = Vec::with_capacity(self.segments.len() + 1);
        segments.extend_from_slice(&self.segments);
        segments.push(PathSegment::Name(name));
        Self {
            segments: segments.into(),
        }
    }

    /// Consume the path, append a list-item index segment, and return the
    /// result.
    ///
    /// The typed-index constructor behind generated list-field path
    /// composition (`<Name>FormPath::<field>(index)`). Indices are stored as
    /// [`PathSegment::Index`] — never formatted into strings.
    ///
    /// # Example
    ///
    /// ```
    /// use gpui_form_core::FieldPath;
    ///
    /// let path = FieldPath::new(&["tags"]).push_index(2);
    /// assert_eq!(path.to_string(), "tags.2");
    /// ```
    pub fn push_index(self, index: usize) -> Self {
        let mut segments = Vec::with_capacity(self.segments.len() + 1);
        segments.extend_from_slice(&self.segments);
        segments.push(PathSegment::Index(index));
        Self {
            segments: segments.into(),
        }
    }

    /// Consume the path, append every segment of `other`, and return the
    /// result.
    ///
    /// Composition seam for nested forms: a parent form's root path joined
    /// with the nested form's field path (`address_root().join(&city())`)
    /// produces the fully composed path (`address.city`).
    ///
    /// # Example
    ///
    /// ```
    /// use gpui_form_core::FieldPath;
    ///
    /// let root = FieldPath::new(&["address"]);
    /// let inner = FieldPath::new(&["city", "zip"]);
    /// assert_eq!(root.join(&inner).to_string(), "address.city.zip");
    /// ```
    pub fn join(mut self, other: &Self) -> Self {
        let mut segments = Vec::with_capacity(self.segments.len() + other.segments.len());
        segments.extend_from_slice(&self.segments);
        segments.extend_from_slice(&other.segments);
        self.segments = segments.into();
        self
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

impl AsRef<FieldPath> for FieldPath {
    fn as_ref(&self) -> &Self {
        self
    }
}

impl core::fmt::Display for FieldPath {
    /// Renders the segments joined by `.`; indices render as numbers. An
    /// empty path renders as `""`.
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        for (i, segment) in self.segments.iter().enumerate() {
            if i > 0 {
                f.write_str(".")?;
            }
            core::fmt::Display::fmt(segment, f)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{HashMap, HashSet};

    #[test]
    fn new_round_trips_name_segments() {
        let path = FieldPath::new(&["name"]);
        assert_eq!(path.segments(), &[PathSegment::Name("name")]);
    }

    #[test]
    fn single_segment_path() {
        let path = FieldPath::new(&["email"]);
        assert_eq!(path.segments(), &[PathSegment::Name("email")]);
        assert!(!path.is_empty());
    }

    #[test]
    fn multi_segment_path_preserves_order() {
        let path = FieldPath::new(&["address", "city", "zip"]);
        assert_eq!(
            path.segments(),
            &[
                PathSegment::Name("address"),
                PathSegment::Name("city"),
                PathSegment::Name("zip")
            ]
        );
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
        assert_eq!(s, r#"[Name("address"), Name("city")]"#);
    }

    #[test]
    fn empty_segments_empty_path() {
        // Zero segments -> empty, even though the constructor accepted a slice.
        let path = FieldPath::new(&[]);
        assert!(path.is_empty());
        assert_eq!(path.segments(), &[] as &[PathSegment]);
    }

    // ── M3: typed index segments (D8 option A) ─────────────────────────────

    #[test]
    fn from_segments_stores_indices_without_strings() {
        let path = FieldPath::from_segments(&[
            PathSegment::Name("contacts"),
            PathSegment::Index(2),
            PathSegment::Name("name"),
        ]);
        assert_eq!(
            path.segments(),
            &[
                PathSegment::Name("contacts"),
                PathSegment::Index(2),
                PathSegment::Name("name")
            ]
        );
        assert_eq!(path.to_string(), "contacts.2.name");
    }

    #[test]
    fn index_segments_render_as_numbers_in_display() {
        let path = FieldPath::new(&["tags"]).push_index(0);
        assert_eq!(path.to_string(), "tags.0");
        let deep = path.push_name("label");
        assert_eq!(deep.to_string(), "tags.0.label");
    }

    #[test]
    fn name_and_index_segments_compare_and_hash_correctly() {
        // An index segment never equals a name segment, so "tags.2" cannot
        // collide with a hypothetical field literally named "2".
        let by_index = FieldPath::new(&["tags"]).push_index(2);
        let by_name = FieldPath::new(&["tags", "2"]);
        assert_ne!(by_index, by_name);

        let mut set = HashSet::new();
        set.insert(by_index);
        assert!(set.contains(&FieldPath::new(&["tags"]).push_index(2)));
        assert!(!set.contains(&by_name));
    }

    #[test]
    fn push_name_and_push_index_compose_consumingly() {
        let path = FieldPath::new(&["contacts"])
            .push_index(1)
            .push_name("email");
        assert_eq!(path.segments().len(), 3);
        assert_eq!(path.to_string(), "contacts.1.email");
    }

    #[test]
    fn deep_push_chains_and_joins_preserve_every_segment() {
        // Composition through the exact-capacity path: every push/join step
        // rebuilds the boxed segment slice at exactly `len + 1` (or
        // `len + other_len` for join). This test pins the OBSERVABLE result
        // — segment order and content preserved end to end — not an
        // allocator-per-step count, which is a std/allocator detail.
        let path = FieldPath::new(&["a"])
            .push_name("b")
            .push_index(3)
            .push_name("c")
            .push_index(4)
            .push_name("d");
        assert_eq!(
            path.segments(),
            &[
                PathSegment::Name("a"),
                PathSegment::Name("b"),
                PathSegment::Index(3),
                PathSegment::Name("c"),
                PathSegment::Index(4),
                PathSegment::Name("d"),
            ]
        );
        assert_eq!(path.to_string(), "a.b.3.c.4.d");

        // join of an index-bearing self with a multi-segment other.
        let left = FieldPath::new(&["x"]).push_index(1);
        let right = FieldPath::new(&["y", "z"]);
        let joined = left.join(&right);
        assert_eq!(joined.segments().len(), 4);
        assert_eq!(joined.to_string(), "x.1.y.z");

        // Joined paths keep composing (the join result feeds further pushes).
        assert_eq!(joined.push_index(9).to_string(), "x.1.y.z.9");
    }

    #[test]
    fn join_appends_all_segments_of_the_other_path() {
        let outer = FieldPath::new(&["address"]);
        let inner = FieldPath::new(&["city"]);
        assert_eq!(outer.clone().join(&inner).to_string(), "address.city");

        // join works on index-bearing paths too (list-of-nested composition).
        let item = FieldPath::new(&["contacts"]).push_index(2);
        let field = FieldPath::new(&["name"]);
        assert_eq!(item.join(&field).to_string(), "contacts.2.name");

        // Joining an empty path is the identity in both directions.
        assert_eq!(FieldPath::new(&[]).join(&inner).to_string(), "city");
        assert_eq!(outer.join(&FieldPath::new(&[])).to_string(), "address");
    }

    #[test]
    fn path_segment_accessors_distinguish_variants() {
        assert_eq!(PathSegment::Name("tags").as_name(), Some("tags"));
        assert_eq!(PathSegment::Name("tags").as_index(), None);
        assert_eq!(PathSegment::Index(7).as_index(), Some(7));
        assert_eq!(PathSegment::Index(7).as_name(), None);
        assert_eq!(PathSegment::Name("tags").to_string(), "tags");
        assert_eq!(PathSegment::Index(7).to_string(), "7");
    }

    #[test]
    fn new_is_equivalent_to_from_segments_of_names() {
        assert_eq!(
            FieldPath::new(&["a", "b"]),
            FieldPath::from_segments(&[PathSegment::Name("a"), PathSegment::Name("b")])
        );
    }

    #[test]
    fn as_ref_returns_self() {
        let path = FieldPath::new(&["name"]);
        let same: &FieldPath = path.as_ref();
        assert_eq!(same, &path);
    }
}
