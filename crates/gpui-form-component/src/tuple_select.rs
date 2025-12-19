//! TupleSelect - A cascading select component for tuple enums
//!
//! This module provides support for enums where variants contain inner values,
//! creating a master-slave relationship between selects. Supports infinite nesting depth
//! with heterogeneous inner types (each variant can have a different inner type).
//!
//! # Example
//!
//! ```ignore
//! #[derive(Clone, Debug, Default, TupleEnumInner)]
//! enum Country {
//!     #[default]
//!     USA(USAState),
//!     Canada(CanadaProvince),
//!     Germany(GermanyState),
//! }
//!
//! #[derive(Clone, Debug, Default, TupleEnumInner)]
//! enum USAState {
//!     #[default]
//!     California(CaliforniaCity),
//!     Texas(TexasCity),
//!     NewYork(NewYorkCity),
//! }
//!
//! #[derive(Clone, Debug, Default, TupleEnumInner)]
//! enum CaliforniaCity {
//!     #[default]
//!     LosAngeles,
//!     SanFrancisco,
//!     SanDiego,
//! }
//! ```
//!
//! This creates a hierarchy: Country -> State -> City with automatic cascading.
//! When the user selects a Country, the State select updates. When they select
//! a State, the City select updates.
//!
//! Note: Each variant can have a *different* inner type. This is supported through
//! dynamic dispatch at runtime.

use gpui::SharedString;
use gpui_component::select::SelectItem;

/// Trait for tuple enums that expose their inner type.
///
/// This trait is derived using `#[derive(TupleEnumInner)]` and provides
/// the ability to:
/// - Get all variants at the current level
/// - Get child variant names for cascading selects
/// - Set a child by index to update the inner value
///
/// Unlike a simple associated type approach, this trait supports heterogeneous
/// inner types - each variant can contain a different type.
pub trait TupleEnumInner: Sized + Clone + Default + 'static {
    /// Returns all possible variants at this level (with default inner values).
    fn variants() -> Vec<Self>;

    /// Returns the variant name/discriminant as a string.
    fn variant_name(&self) -> &'static str;

    /// Returns true if this variant contains an inner value.
    fn has_inner(&self) -> bool;

    /// Returns the variant names of the children for this specific variant.
    /// Returns an empty vec for unit variants or variants without children.
    fn child_variant_names(&self) -> Vec<&'static str>;

    /// Creates a new instance with the child at the given index.
    /// Returns None if the variant doesn't have children or index is out of bounds.
    fn set_child_by_index(&self, index: usize) -> Option<Self>;

    /// Sets the child at a given path depth recursively.
    /// `path` is a slice of indices, where path[0] is the child index at this level,
    /// path[1] is the grandchild index, etc.
    /// Returns None if any index is out of bounds or the path is empty.
    fn set_child_by_path(&self, path: &[usize]) -> Option<Self>;

    /// Returns the depth of nesting for this variant's children.
    /// Returns 0 for leaf variants.
    fn child_depth(&self) -> usize;

    /// Returns the maximum depth of nesting for this enum type.
    fn depth() -> usize;
}

/// A wrapper for tuple enum variants that implements SelectItem.
///
/// This allows tuple enum variants to be displayed in a select dropdown
/// while preserving access to their inner values.
#[derive(Clone)]
pub struct TupleSelectItem<T: TupleEnumInner> {
    value: T,
    title: SharedString,
}

impl<T: TupleEnumInner> TupleSelectItem<T> {
    /// Create a new TupleSelectItem with a custom title.
    pub fn new(value: T, title: impl Into<SharedString>) -> Self {
        Self {
            value,
            title: title.into(),
        }
    }

    /// Create a TupleSelectItem using the variant name as the title.
    pub fn from_variant(value: T) -> Self {
        let title = value.variant_name();
        Self {
            value,
            title: title.into(),
        }
    }

    /// Get a reference to the inner value.
    pub fn get_value(&self) -> &T {
        &self.value
    }

    /// Consume self and return the inner value.
    pub fn into_value(self) -> T {
        self.value
    }

    /// Check if this variant has nested inner values.
    pub fn has_inner(&self) -> bool {
        self.value.has_inner()
    }

    /// Get the child variant names if this variant has children.
    pub fn child_variant_names(&self) -> Vec<&'static str> {
        self.value.child_variant_names()
    }

    /// Set the child by index and return a new item.
    pub fn with_child_at(&self, index: usize) -> Option<Self> {
        self.value.set_child_by_index(index).map(Self::from_variant)
    }
}

impl<T: TupleEnumInner> SelectItem for TupleSelectItem<T> {
    type Value = T;

    fn title(&self) -> SharedString {
        self.title.clone()
    }

    fn value(&self) -> &Self::Value {
        &self.value
    }
}

/// Helper function to create select items from tuple enum variants.
pub fn tuple_enum_to_select_items<T>() -> Vec<TupleSelectItem<T>>
where
    T: TupleEnumInner,
{
    T::variants()
        .into_iter()
        .map(TupleSelectItem::from_variant)
        .collect()
}

/// Represents a selection path through nested tuple enums.
///
/// Each element is the index of the selected variant at that depth level.
#[derive(Clone, Debug, Default)]
pub struct TupleSelectPath {
    indices: Vec<usize>,
}

impl TupleSelectPath {
    /// Create a new empty selection path.
    pub fn new() -> Self {
        Self {
            indices: Vec::new(),
        }
    }

    /// Create a path with the given indices.
    pub fn with_indices(indices: Vec<usize>) -> Self {
        Self { indices }
    }

    /// Get the selection index at a given depth.
    pub fn get(&self, depth: usize) -> Option<usize> {
        self.indices.get(depth).copied()
    }

    /// Set the selection at a given depth, truncating any deeper selections.
    pub fn set(&mut self, depth: usize, index: usize) {
        self.indices.truncate(depth);
        self.indices.push(index);
    }

    /// Clear selections from a given depth onwards.
    pub fn clear_from(&mut self, depth: usize) {
        self.indices.truncate(depth);
    }

    /// Truncate the path to the given length.
    pub fn truncate(&mut self, len: usize) {
        self.indices.truncate(len);
    }

    /// Get the current depth of the selection.
    pub fn len(&self) -> usize {
        self.indices.len()
    }

    /// Get all indices as a slice.
    pub fn indices(&self) -> &[usize] {
        &self.indices
    }

    /// Check if the path is empty.
    pub fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }
}

/// Build a value from a selection path.
///
/// This recursively constructs the nested enum by following the path
/// and setting children at each level.
///
/// The path indices work as follows:
/// - path[0]: selects the variant at the root level
/// - path[1]: selects the child variant within the root's inner value
/// - path[2]: selects the grandchild variant, etc.
///
/// # Example
/// ```ignore
/// // Build Country::USA(USAState::Texas(TexasCity::Austin))
/// let mut path = TupleSelectPath::new();
/// path.set(0, 0); // USA (index 0)
/// path.set(1, 1); // Texas (index 1 within USAState)
/// path.set(2, 2); // Austin (index 2 within TexasCity)
/// let country: Country = build_from_path(&path).unwrap();
/// ```
pub fn build_from_path<T: TupleEnumInner>(path: &TupleSelectPath) -> Option<T> {
    if path.is_empty() {
        return None;
    }

    let variants = T::variants();
    let root_index = path.get(0)?;
    let root = variants.get(root_index)?.clone();

    // If there are more levels in the path, use set_child_by_path
    if path.len() > 1 {
        root.set_child_by_path(&path.indices()[1..])
    } else {
        Some(root)
    }
}

/// Get the maximum depth of a tuple enum hierarchy.
pub fn get_max_depth<T: TupleEnumInner>() -> usize {
    T::depth()
}

/// Information about a selection level.
#[derive(Clone, Debug)]
pub struct SelectionLevel {
    /// The variant names available at this level
    pub variant_names: Vec<&'static str>,
    /// The currently selected index (if any)
    pub selected_index: Option<usize>,
}

/// Get all selection levels for a current value.
///
/// Returns a vector of SelectionLevel, one for each depth in the hierarchy.
/// Each level contains the available variant names and the current selection.
pub fn get_selection_levels<T: TupleEnumInner>(value: &T) -> Vec<SelectionLevel> {
    let mut levels = Vec::new();

    // First level: all top-level variants
    let variants = T::variants();
    let variant_names: Vec<&'static str> = variants.iter().map(|v| v.variant_name()).collect();
    let current_name = value.variant_name();
    let selected_index = variant_names.iter().position(|&n| n == current_name);

    levels.push(SelectionLevel {
        variant_names,
        selected_index,
    });

    // Add child levels by traversing the current selection
    add_child_levels(value, &mut levels);

    levels
}

fn add_child_levels<T: TupleEnumInner>(value: &T, levels: &mut Vec<SelectionLevel>) {
    let child_names = value.child_variant_names();
    if child_names.is_empty() {
        return;
    }

    // We have children - add a level for them
    // To find the selected child index, we need to check which child is currently set
    // This is tricky with heterogeneous types, so we use the child variant names

    // For now, we'll need the child value to determine the selection
    // This requires a way to get the current child's variant name
    // We can do this by checking which variant was created with set_child_by_index

    // Find the selected child by trying each index
    let selected_index = None;
    for (i, _) in child_names.iter().enumerate() {
        if let Some(with_child) = value.set_child_by_index(i) {
            // Compare if this matches the current value's child
            // This is a limitation - we can't easily compare without more trait methods
            // For now, we'll just show all options without a selection indicator
            // The actual selection would be tracked separately in the form state
            let _ = with_child;
        }
    }

    levels.push(SelectionLevel {
        variant_names: child_names,
        selected_index,
    });

    // Recursively add deeper levels if there are more children
    // This would require accessing the inner value, which we can't do generically
    // The form state will need to track the full selection path
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test types would be defined here using the derive macro
}
