//! Non-rendering layout hints for form fields (METADATA-FIRST v1).
//!
//! This module exposes metadata that fields can declare through
//! `#[gpui_form(...)]` attributes so generated or prototyped forms can consume
//! them. It is deliberately **metadata-only**: nothing here describes how a
//! field is rendered, painted, or laid out by GPUI. Consumers (application
//! code, prototyping generators) decide how to interpret each hint.
//!
//! # v1 scope
//!
//! The v1 attribute set is:
//!
//! - `section` — groups consecutive fields under a named section.
//! - `label` — preferred human-readable label for the field.
//! - `description` — help text or comment hint for the field.
//! - `placeholder` — placeholder text for inputs that support it.
//! - `width` — relative width hint ([`LayoutWidth`]: `full` | `half` | `third`).
//!
//! All hints are optional. When `label` is absent, consumers should fall back
//! to the field name at consumption time.
//!
//! # Const constructibility
//!
//! Both [`FieldLayout`] and [`LayoutWidth`] are `const`-constructible so that
//! the derive can emit `FieldVariant::new(...).with_layout(...)` chains inside
//! `inventory::submit! { ... }` in the user crate. All builder methods are
//! `const fn`.
//!
//! # Order preservation
//!
//! `section` grouping is order-preserving: consumers should treat consecutive
//! fields with the same `section` as a single group, not reorder fields across
//! the form. This keeps v1 minimal and gives richer layouts (columns,
//! collapsible sections) a stable foundation to build on later.

use strum::{Display, EnumString, IntoStaticStr};

/// Relative width hint for a single field.
///
/// This is a **hint**, not a layout engine. Consumers are free to ignore it or
/// map it onto whatever grid/column system they use. The v1 set is intentionally
/// small so that generators can emit reasonable scaffolds without a full layout
/// model.
///
/// Serialized as the snake-case identifier (`full`, `half`, `third`) to match
/// the bare-ident attribute syntax in `#[gpui_form(width = half)]` (quoted
/// forms are also accepted by the derive).
///
/// Defaults to [`LayoutWidth::Full`] — fields take their natural width unless a
/// narrower hint is provided.
#[derive(
    Clone, Copy, Debug, Default, Display, EnumString, Eq, IntoStaticStr, PartialEq,
)]
#[strum(serialize_all = "snake_case")]
pub enum LayoutWidth {
    /// Full available width (the default).
    #[default]
    Full,
    /// Half of the available width (e.g. one of two columns).
    Half,
    /// A third of the available width (e.g. one of three columns).
    Third,
}

/// Non-rendering layout hints attached to a [`crate::registry::FieldVariant`].
///
/// All string hints are `&'static str` because the derive emits them as string
/// literals in the user crate (see [`crate::registry::FieldVariant`]).
///
/// Construct with [`FieldLayout::new`] (all hints absent, width full) and chain
/// the `with_*` builders:
///
/// ```
/// use gpui_form_schema::layout::{FieldLayout, LayoutWidth};
///
/// const LAYOUT: FieldLayout = FieldLayout::new()
///     .with_section(Some("Account"))
///     .with_label(Some("Username"))
///     .with_description(Some("Your unique handle"))
///     .with_placeholder(Some("username"))
///     .with_width(LayoutWidth::Half);
///
/// assert!(!LAYOUT.is_empty());
/// ```
///
/// `is_empty()` returns `true` only when every hint is absent **and** the width
/// is [`LayoutWidth::Full`]; such a layout carries no information and consumers
/// may skip emitting anything for it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FieldLayout {
    /// Optional section name for grouping consecutive fields. Order-preserving.
    pub section: Option<&'static str>,
    /// Preferred human-readable label. When `None`, consumers fall back to the
    /// field name.
    pub label: Option<&'static str>,
    /// Optional help text / comment hint shown alongside the field.
    pub description: Option<&'static str>,
    /// Optional placeholder text for inputs that support one.
    pub placeholder: Option<&'static str>,
    /// Relative width hint. See [`LayoutWidth`].
    pub width: LayoutWidth,
}

impl FieldLayout {
    /// An empty layout: all hints `None`, width [`LayoutWidth::Full`].
    #[must_use]
    pub const fn new() -> Self {
        Self {
            section: None,
            label: None,
            description: None,
            placeholder: None,
            width: LayoutWidth::Full,
        }
    }

    /// Attach (or clear with `None`) the section hint.
    #[must_use]
    pub const fn with_section(mut self, section: Option<&'static str>) -> Self {
        self.section = section;
        self
    }

    /// Attach (or clear with `None`) the label hint.
    #[must_use]
    pub const fn with_label(mut self, label: Option<&'static str>) -> Self {
        self.label = label;
        self
    }

    /// Attach (or clear with `None`) the description hint.
    #[must_use]
    pub const fn with_description(mut self, description: Option<&'static str>) -> Self {
        self.description = description;
        self
    }

    /// Attach (or clear with `None`) the placeholder hint.
    #[must_use]
    pub const fn with_placeholder(mut self, placeholder: Option<&'static str>) -> Self {
        self.placeholder = placeholder;
        self
    }

    /// Attach the relative width hint.
    #[must_use]
    pub const fn with_width(mut self, width: LayoutWidth) -> Self {
        self.width = width;
        self
    }

    /// Returns `true` when the layout carries no information.
    ///
    /// True exactly when `section`, `label`, `description`, and `placeholder`
    /// are all `None` **and** `width` is [`LayoutWidth::Full`]. Consumers can
    /// use this to skip emitting anything for the default case.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.section.is_none()
            && self.label.is_none()
            && self.description.is_none()
            && self.placeholder.is_none()
            && matches!(self.width, LayoutWidth::Full)
    }
}

impl Default for FieldLayout {
    /// Equivalent to [`FieldLayout::new`].
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::ComponentsBehaviour;
    use crate::registry::FieldVariant;

    #[test]
    fn layout_width_default_is_full() {
        let width: LayoutWidth = Default::default();
        assert_eq!(width, LayoutWidth::Full);
    }

    #[test]
    fn layout_width_display_is_snake_case() {
        assert_eq!(LayoutWidth::Full.to_string(), "full");
        assert_eq!(LayoutWidth::Half.to_string(), "half");
        assert_eq!(LayoutWidth::Third.to_string(), "third");
    }

    #[test]
    fn layout_width_enum_string_round_trips() {
        assert_eq!("full".parse::<LayoutWidth>().unwrap(), LayoutWidth::Full);
        assert_eq!("half".parse::<LayoutWidth>().unwrap(), LayoutWidth::Half);
        assert_eq!("third".parse::<LayoutWidth>().unwrap(), LayoutWidth::Third);
        assert!("quarter".parse::<LayoutWidth>().is_err());
    }

    #[test]
    fn layout_width_into_static_str() {
        let s: &'static str = LayoutWidth::Half.into();
        assert_eq!(s, "half");
    }

    #[test]
    fn field_layout_new_is_empty() {
        const EMPTY: FieldLayout = FieldLayout::new();
        assert!(EMPTY.is_empty());
        assert_eq!(EMPTY.width, LayoutWidth::Full);
        assert!(EMPTY.section.is_none());
        assert!(EMPTY.label.is_none());
        assert!(EMPTY.description.is_none());
        assert!(EMPTY.placeholder.is_none());
    }

    #[test]
    fn field_layout_default_matches_new() {
        assert_eq!(FieldLayout::default(), FieldLayout::new());
    }

    #[test]
    fn field_layout_builders_populate_fields() {
        const LAYOUT: FieldLayout = FieldLayout::new()
            .with_section(Some("Account"))
            .with_label(Some("Username"))
            .with_description(Some("Your unique handle"))
            .with_placeholder(Some("username"))
            .with_width(LayoutWidth::Half);
        assert!(!LAYOUT.is_empty());
        assert_eq!(LAYOUT.section, Some("Account"));
        assert_eq!(LAYOUT.label, Some("Username"));
        assert_eq!(LAYOUT.description, Some("Your unique handle"));
        assert_eq!(LAYOUT.placeholder, Some("username"));
        assert_eq!(LAYOUT.width, LayoutWidth::Half);
    }

    #[test]
    fn field_layout_clearing_with_none_keeps_empty() {
        const CLEARED: FieldLayout = FieldLayout::new()
            .with_section(Some("x"))
            .with_section(None)
            .with_label(None)
            .with_width(LayoutWidth::Full);
        assert!(CLEARED.is_empty());
    }

    #[test]
    fn field_layout_width_alone_is_not_empty() {
        const WIDTH_ONLY: FieldLayout = FieldLayout::new().with_width(LayoutWidth::Third);
        assert!(!WIDTH_ONLY.is_empty());
    }

    #[test]
    fn field_variant_new_defaults_layout_to_empty() {
        const FV: FieldVariant = FieldVariant::new(
            "username",
            "String",
            false,
            ComponentsBehaviour::Input,
        );
        assert!(FV.layout.is_empty());
        assert_eq!(FV.layout.width, LayoutWidth::Full);
    }

    #[test]
    fn field_variant_with_layout_round_trips() {
        const LAYOUT: FieldLayout = FieldLayout::new()
            .with_section(Some("Account"))
            .with_label(Some("Username"))
            .with_width(LayoutWidth::Half);
        const FV: FieldVariant =
            FieldVariant::new("username", "String", false, ComponentsBehaviour::Input)
                .with_layout(LAYOUT);
        assert_eq!(FV.layout.section, Some("Account"));
        assert_eq!(FV.layout.label, Some("Username"));
        assert_eq!(FV.layout.width, LayoutWidth::Half);
        assert!(!FV.layout.is_empty());
    }

    #[test]
    fn field_variant_with_layout_is_const_constructible() {
        // Const chain mirroring what the derive emits.
        const FV: FieldVariant = FieldVariant::new(
            "email",
            "String",
            false,
            ComponentsBehaviour::Input,
        )
        .with_layout(
            FieldLayout::new()
                .with_section(Some("Contact"))
                .with_placeholder(Some("you@example.com"))
                .with_width(LayoutWidth::Half),
        );
        assert_eq!(FV.field_name, "email");
        assert_eq!(FV.layout.section, Some("Contact"));
        assert_eq!(FV.layout.placeholder, Some("you@example.com"));
        assert_eq!(FV.layout.width, LayoutWidth::Half);
    }
}
