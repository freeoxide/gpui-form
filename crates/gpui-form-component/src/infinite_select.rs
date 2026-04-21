//! Infinite-select runtime support for nested enum trees.
//!
//! This module exposes both the low-level trait and path helpers used by
//! generated code and a higher-level `InfiniteSelectState` entity that owns the
//! cascading `SelectState`s for a field.

use gpui::{
    App, AppContext as _, Context, Empty, Entity, EventEmitter, FocusHandle, Focusable,
    IntoElement, Render, SharedString, Subscription, Window,
};
use gpui_component::{
    IndexPath,
    select::{SearchableVec, SelectDelegate, SelectEvent, SelectItem, SelectState},
};

/// Trait for infinite-select enums that expose their nested structure.
///
/// This trait is derived using `#[derive(InfiniteSelect)]` and provides:
/// - the variant list at each level
/// - child variant names for cascading selects
/// - index-based child selection to update nested values
/// - the current selection path for a concrete value
///
/// Unlike a simple associated-type approach, this trait supports heterogeneous
/// inner types: each variant can contain a different inner type.
pub trait InfiniteSelect: Sized + Clone + Default + 'static {
    /// Returns all possible variants at this level with default inner values.
    fn variants() -> Vec<Self>;

    /// Returns the variant name/discriminant as a string.
    fn variant_name(&self) -> &'static str;

    /// Returns true if this variant contains an inner value.
    fn has_inner(&self) -> bool;

    /// Returns the variant names of the children for this specific variant.
    /// Returns an empty vec for unit variants or variants without children.
    fn child_variant_names(&self) -> Vec<&'static str>;

    /// Creates a new instance with the child at the given index.
    /// Returns None if the variant doesn't have children or the index is out of bounds.
    fn set_child_by_index(&self, index: usize) -> Option<Self>;

    /// Sets the child at a given path depth recursively.
    /// `path[0]` is the child index at this level, `path[1]` the grandchild, and so on.
    fn set_child_by_path(&self, path: &[usize]) -> Option<Self>;

    /// Returns the depth of nesting for this variant's children.
    /// Returns 0 for leaf variants.
    fn child_depth(&self) -> usize;

    /// Returns the maximum depth of nesting for this enum type.
    fn depth() -> usize;

    /// Returns the current selection path for this concrete value.
    ///
    /// The returned path includes the root variant index at depth 0.
    fn selection_path(&self) -> InfiniteSelectPath;

    /// Returns the variant names of the inner value's children.
    fn inner_child_variant_names(&self) -> Vec<&'static str>;

    /// Sets a child on the inner value and wraps it back.
    fn inner_set_child_by_index(&self, index: usize) -> Option<Self>;

    /// Returns true if the inner value itself has children.
    fn inner_has_inner(&self) -> bool;

    /// Returns the localized label for this type (level).
    fn type_label(&self) -> SharedString;

    /// Returns the localized description for this type (level).
    fn type_description(&self) -> SharedString;

    /// Returns the localized label for the child at the given depth relative to this node.
    /// `depth = 0` is the immediate child.
    fn child_label_at_depth(&self, depth: usize) -> Option<SharedString>;

    /// Returns the localized description for the child at the given depth.
    fn child_description_at_depth(&self, depth: usize) -> Option<SharedString>;

    /// Internal method to delegate label lookup to the inner value.
    fn inner_child_label_at_depth(&self, depth: usize) -> Option<SharedString>;

    /// Internal method to delegate description lookup to the inner value.
    fn inner_child_description_at_depth(&self, depth: usize) -> Option<SharedString>;
}

/// A wrapper for infinite-select enum variants that implements `SelectItem`.
///
/// This allows infinite-select enum variants to be displayed in a select dropdown
/// while preserving access to the nested value.
#[derive(Clone)]
pub struct InfiniteSelectItem<T: InfiniteSelect> {
    value: T,
    title: SharedString,
}

impl<T: InfiniteSelect> InfiniteSelectItem<T> {
    /// Creates a new item with a custom title.
    pub fn new(value: T, title: impl Into<SharedString>) -> Self {
        Self {
            value,
            title: title.into(),
        }
    }

    /// Creates an item using `variant_name()` as the title.
    pub fn from_variant(value: T) -> Self {
        let title = value.variant_name();
        Self {
            value,
            title: title.into(),
        }
    }

    /// Returns a reference to the wrapped value.
    pub fn get_value(&self) -> &T {
        &self.value
    }

    /// Consumes the item and returns the wrapped value.
    pub fn into_value(self) -> T {
        self.value
    }

    /// Returns true if the wrapped value has nested inner values.
    pub fn has_inner(&self) -> bool {
        self.value.has_inner()
    }

    /// Returns the child variant names if the wrapped value has children.
    pub fn child_variant_names(&self) -> Vec<&'static str> {
        self.value.child_variant_names()
    }

    /// Returns a new item with a child selected at the given index.
    pub fn with_child_at(&self, index: usize) -> Option<Self> {
        self.value.set_child_by_index(index).map(Self::from_variant)
    }
}

impl<T: InfiniteSelect> SelectItem for InfiniteSelectItem<T> {
    type Value = T;

    fn title(&self) -> SharedString {
        self.title.clone()
    }

    fn value(&self) -> &Self::Value {
        &self.value
    }
}

/// Creates root select items from `T::variants()`.
pub fn to_select_items<T>() -> Vec<InfiniteSelectItem<T>>
where
    T: InfiniteSelect,
{
    T::variants()
        .into_iter()
        .map(InfiniteSelectItem::from_variant)
        .collect()
}

/// Represents a selection path through nested infinite-select enums.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct InfiniteSelectPath {
    indices: Vec<usize>,
}

impl InfiniteSelectPath {
    /// Creates a new empty selection path.
    pub fn new() -> Self {
        Self {
            indices: Vec::new(),
        }
    }

    /// Creates a path with the given indices.
    pub fn with_indices(indices: Vec<usize>) -> Self {
        Self { indices }
    }

    /// Returns the selection index at a given depth.
    pub fn get(&self, depth: usize) -> Option<usize> {
        self.indices.get(depth).copied()
    }

    /// Sets the selection at a given depth, truncating deeper selections.
    pub fn set(&mut self, depth: usize, index: usize) {
        self.indices.truncate(depth);
        self.indices.push(index);
    }

    /// Clears selections from a given depth onwards.
    pub fn clear_from(&mut self, depth: usize) {
        self.indices.truncate(depth);
    }

    /// Truncates the path to the given length.
    pub fn truncate(&mut self, len: usize) {
        self.indices.truncate(len);
    }

    /// Returns the current depth of the selection.
    pub fn len(&self) -> usize {
        self.indices.len()
    }

    /// Returns all indices as a slice.
    pub fn indices(&self) -> &[usize] {
        &self.indices
    }

    /// Returns true if the path is empty.
    pub fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }
}

/// Returns the current selection path for a concrete infinite-select value.
pub fn path_from_value<T: InfiniteSelect>(value: &T) -> InfiniteSelectPath {
    value.selection_path()
}

/// Rebuilds a value from a selection path.
pub fn build_from_path<T: InfiniteSelect>(path: &InfiniteSelectPath) -> Option<T> {
    if path.is_empty() {
        return None;
    }

    let variants = T::variants();
    let root_index = path.get(0)?;
    let root = variants.get(root_index)?.clone();

    if path.len() > 1 {
        root.set_child_by_path(&path.indices()[1..])
    } else {
        Some(root)
    }
}

/// Options for the runtime `InfiniteSelectState`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct InfiniteSelectStateOptions {
    searchable: bool,
    max_depth: Option<usize>,
}

impl InfiniteSelectStateOptions {
    /// Enables or disables search on the backing select widgets.
    pub fn searchable(mut self, searchable: bool) -> Self {
        self.searchable = searchable;
        self
    }

    /// Limits how many levels the state will render.
    ///
    /// The stored value and selection path still preserve deeper default selections.
    pub fn max_depth(mut self, max_depth: usize) -> Self {
        self.max_depth = Some(max_depth);
        self
    }
}

/// Event emitted by `InfiniteSelectState` whenever the selection changes.
#[derive(Clone)]
pub enum InfiniteSelectEvent<T: InfiniteSelect> {
    Change(T),
}

/// Runtime state for a cascading infinite-select field.
pub struct InfiniteSelectState<T, D = Vec<InfiniteSelectItem<T>>>
where
    T: InfiniteSelect,
    D: SelectDelegate<Item = InfiniteSelectItem<T>> + From<Vec<InfiniteSelectItem<T>>> + 'static,
{
    value: T,
    path: InfiniteSelectPath,
    master_select: Entity<SelectState<D>>,
    child_selects: Vec<Entity<SelectState<D>>>,
    options: InfiniteSelectStateOptions,
    _master_subscription: Subscription,
    _child_subscriptions: Vec<Subscription>,
}

/// Search-enabled state alias for `component(infinite_select(searchable))`.
pub type SearchableInfiniteSelectState<T> =
    InfiniteSelectState<T, SearchableVec<InfiniteSelectItem<T>>>;

impl<T, D> Focusable for InfiniteSelectState<T, D>
where
    T: InfiniteSelect,
    D: SelectDelegate<Item = InfiniteSelectItem<T>> + From<Vec<InfiniteSelectItem<T>>> + 'static,
{
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.master_select.read(cx).focus_handle(cx)
    }
}

impl<T, D> EventEmitter<InfiniteSelectEvent<T>> for InfiniteSelectState<T, D>
where
    T: InfiniteSelect,
    D: SelectDelegate<Item = InfiniteSelectItem<T>> + From<Vec<InfiniteSelectItem<T>>> + 'static,
{
}

impl<T, D> InfiniteSelectState<T, D>
where
    T: InfiniteSelect,
    D: SelectDelegate<Item = InfiniteSelectItem<T>> + From<Vec<InfiniteSelectItem<T>>> + 'static,
{
    /// Creates a new state from the given initial value.
    pub fn new(initial_value: T, window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self::new_with_options(
            initial_value,
            InfiniteSelectStateOptions::default(),
            window,
            cx,
        )
    }

    /// Creates a new state with explicit options.
    pub fn new_with_options(
        initial_value: T,
        options: InfiniteSelectStateOptions,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let path = path_from_value(&initial_value);
        let root_selected = path.get(0);
        let master_select = cx.new(|cx| {
            build_select_state::<T, D>(
                to_select_items::<T>(),
                root_selected,
                options.searchable,
                window,
                cx,
            )
        });
        let master_subscription = cx.subscribe_in(&master_select, window, Self::on_select_event);

        let mut this = Self {
            value: initial_value,
            path,
            master_select,
            child_selects: Vec::new(),
            options,
            _master_subscription: master_subscription,
            _child_subscriptions: Vec::new(),
        };
        this.rebuild_child_selects(window, cx);
        this
    }

    /// Returns the current concrete selection.
    pub fn value(&self) -> &T {
        &self.value
    }

    /// Returns the current selection path.
    pub fn path(&self) -> &InfiniteSelectPath {
        &self.path
    }

    /// Returns the root select entity.
    pub fn master_select(&self) -> Entity<SelectState<D>> {
        self.master_select.clone()
    }

    /// Returns the currently visible child selects.
    pub fn child_selects(&self) -> Vec<Entity<SelectState<D>>> {
        self.child_selects.clone()
    }

    /// Programmatically sets the current selection.
    pub fn set_value(&mut self, value: T, window: &mut Window, cx: &mut Context<Self>) {
        self.apply_selection(value, false, window, cx);
    }

    fn max_depth(&self) -> usize {
        match self.options.max_depth {
            Some(max_depth) => max_depth.clamp(1, T::depth()),
            None => T::depth(),
        }
    }

    fn on_select_event(
        &mut self,
        _this: &Entity<SelectState<D>>,
        event: &SelectEvent<D>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let SelectEvent::Confirm(Some(selected)) = event else {
            return;
        };

        self.apply_selection(selected.clone(), true, window, cx);
    }

    fn apply_selection(
        &mut self,
        value: T,
        emit: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.value = value.clone();
        self.path = path_from_value(&value);
        self.sync_master_select(window, cx);
        self.rebuild_child_selects(window, cx);
        if emit {
            cx.emit(InfiniteSelectEvent::Change(value));
        }
        cx.notify();
    }

    fn sync_master_select(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let selected_index = selected_index(self.path.get(0).unwrap_or(0));
        self.master_select.update(cx, |state, cx| {
            state.set_selected_index(selected_index, window, cx);
        });
    }

    fn rebuild_child_selects(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let child_selects = build_child_selects::<T, D>(
            &self.value,
            &self.path,
            self.max_depth(),
            self.options.searchable,
            window,
            cx,
        );

        self._child_subscriptions = child_selects
            .iter()
            .map(|child| cx.subscribe_in(child, window, Self::on_select_event))
            .collect();
        self.child_selects = child_selects;
    }
}

impl<T, D> Render for InfiniteSelectState<T, D>
where
    T: InfiniteSelect,
    D: SelectDelegate<Item = InfiniteSelectItem<T>> + From<Vec<InfiniteSelectItem<T>>> + 'static,
{
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        Empty
    }
}

fn build_child_selects<T, D>(
    parent: &T,
    path: &InfiniteSelectPath,
    max_depth: usize,
    searchable: bool,
    window: &mut Window,
    cx: &mut Context<InfiniteSelectState<T, D>>,
) -> Vec<Entity<SelectState<D>>>
where
    T: InfiniteSelect,
    D: SelectDelegate<Item = InfiniteSelectItem<T>> + From<Vec<InfiniteSelectItem<T>>> + 'static,
{
    let mut current_value = parent.clone();
    let mut selects = Vec::new();

    for level in 0..max_depth.saturating_sub(1) {
        let items = child_items_for_level(&current_value, level);
        if items.is_empty() {
            break;
        }

        let selected_row = path.get(level + 1).unwrap_or(0).min(items.len() - 1);
        let child_select = cx.new(|cx| {
            build_select_state::<T, D>(items.clone(), Some(selected_row), searchable, window, cx)
        });
        current_value = items[selected_row].get_value().clone();
        selects.push(child_select);
    }

    selects
}

fn child_items_for_level<T: InfiniteSelect>(
    current_value: &T,
    level: usize,
) -> Vec<InfiniteSelectItem<T>> {
    let (has_more, child_names) = if level == 0 {
        (
            current_value.has_inner(),
            current_value.child_variant_names(),
        )
    } else {
        (
            current_value.inner_has_inner(),
            current_value.inner_child_variant_names(),
        )
    };

    if !has_more || child_names.is_empty() {
        return Vec::new();
    }

    child_names
        .into_iter()
        .enumerate()
        .filter_map(|(index, title)| {
            let value = if level == 0 {
                current_value.set_child_by_index(index)
            } else {
                current_value.inner_set_child_by_index(index)
            };
            value.map(|value| InfiniteSelectItem::new(value, title))
        })
        .collect()
}

fn build_select_state<T, D>(
    items: Vec<InfiniteSelectItem<T>>,
    selected_row: Option<usize>,
    searchable: bool,
    window: &mut Window,
    cx: &mut Context<SelectState<D>>,
) -> SelectState<D>
where
    T: InfiniteSelect,
    D: SelectDelegate<Item = InfiniteSelectItem<T>> + From<Vec<InfiniteSelectItem<T>>> + 'static,
{
    let mut state = SelectState::new(
        items.into(),
        selected_row.and_then(selected_index),
        window,
        cx,
    );
    if searchable {
        state = state.searchable(true);
    }
    state
}

fn selected_index(row: usize) -> Option<IndexPath> {
    Some(IndexPath {
        section: 0,
        row,
        column: 0,
    })
}
