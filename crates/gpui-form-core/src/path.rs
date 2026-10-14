//! Typed field paths: an ordered sequence of [`PathSegment`]s naming a
//! form field, shared by validation, dirty tracking, and schema export
//! instead of ad-hoc strings.

/// One typed segment of a [`FieldPath`]: a field name or a list-item index.
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

/// A typed path to a form field: an ordered sequence of [`PathSegment`]s.
/// Equality and hashing are by segment sequence; [`core::fmt::Display`]
/// renders segments joined by `.` (`address.city`, `contacts.2.name`).
pub struct FieldPath {
    segments: Box<[PathSegment]>,
}

impl FieldPath {
    /// Create a path from static string segments, each becoming a
    /// [`PathSegment::Name`]; an empty slice produces an empty path.
    pub fn new(segments: &[&'static str]) -> Self {
        Self {
            segments: segments.iter().copied().map(PathSegment::Name).collect(),
        }
    }

    /// Create a path from typed segments (names and indices), copied into
    /// an owned box.
    pub fn from_segments(segments: &[PathSegment]) -> Self {
        Self {
            segments: segments.into(),
        }
    }

    /// Borrow the ordered segment slice.
    pub fn segments(&self) -> &[PathSegment] {
        &self.segments
    }

    /// Whether the path has zero segments.
    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }

    /// Consume the path, append a field-name segment, and return the result.
    pub fn push_name(self, name: &'static str) -> Self {
        let mut segments = Vec::with_capacity(self.segments.len() + 1);
        segments.extend_from_slice(&self.segments);
        segments.push(PathSegment::Name(name));
        Self {
            segments: segments.into(),
        }
    }

    /// Consume the path, append a list-item index segment, and return the
    /// result. Indices are stored typed, never formatted into strings.
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
        assert_ne!(FieldPath::new(&["city", "address"]), a);
    }

    #[test]
    fn hash_by_segment_sequence_in_set() {
        let mut set = HashSet::new();
        set.insert(FieldPath::new(&["address", "city"]));

        assert!(set.contains(&FieldPath::new(&["address", "city"])));
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

        assert_eq!(original, copy);
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
        let path = FieldPath::new(&[]);
        assert!(path.is_empty());
        assert_eq!(path.segments(), &[] as &[PathSegment]);
    }

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

        let left = FieldPath::new(&["x"]).push_index(1);
        let right = FieldPath::new(&["y", "z"]);
        let joined = left.join(&right);
        assert_eq!(joined.segments().len(), 4);
        assert_eq!(joined.to_string(), "x.1.y.z");

        assert_eq!(joined.push_index(9).to_string(), "x.1.y.z.9");
    }

    #[test]
    fn join_appends_all_segments_of_the_other_path() {
        let outer = FieldPath::new(&["address"]);
        let inner = FieldPath::new(&["city"]);
        assert_eq!(outer.clone().join(&inner).to_string(), "address.city");

        let item = FieldPath::new(&["contacts"]).push_index(2);
        let field = FieldPath::new(&["name"]);
        assert_eq!(item.join(&field).to_string(), "contacts.2.name");

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
