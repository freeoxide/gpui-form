//! Non-rendering layout hints (`section`/`label`/`description`/`placeholder`/
//! `width`) declared through `#[gpui_form(...)]` attributes; consumers decide
//! how to interpret each hint. `section` grouping is order-preserving.

use serde::{Deserialize, Serialize};
use strum::{Display, EnumString, IntoStaticStr};

/// Relative width hint for a single field: a hint, not a layout engine.
/// Serialized as the snake-case identifier (`full`, `half`, `third`) to match
/// the bare-ident attribute syntax; defaults to [`LayoutWidth::Full`].
#[derive(
    Clone,
    Copy,
    Debug,
    Default,
    Display,
    EnumString,
    Eq,
    IntoStaticStr,
    PartialEq,
    Serialize,
    Deserialize,
)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum LayoutWidth {
    #[default]
    Full,
    Half,
    Third,
}

/// Non-rendering layout hints attached to a [`crate::registry::FieldVariant`].
/// String hints are `&'static str` because the derive emits literals; builders
/// stay `const fn` so the derive can emit chains inside `inventory::submit!`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FieldLayout {
    /// Optional section name for grouping consecutive fields. Order-preserving.
    pub section: Option<&'static str>,
    /// Preferred label; when `None`, consumers fall back to the field name.
    pub label: Option<&'static str>,
    /// Optional help text / comment hint shown alongside the field.
    pub description: Option<&'static str>,
    pub placeholder: Option<&'static str>,
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

    #[must_use]
    pub const fn with_width(mut self, width: LayoutWidth) -> Self {
        self.width = width;
        self
    }

    /// Returns `true` when every hint is `None` and the width is
    /// [`LayoutWidth::Full`] — consumers can skip emitting for the default case.
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
        const FV: FieldVariant =
            FieldVariant::new("username", "String", false, ComponentsBehaviour::Input);
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
        const FV: FieldVariant =
            FieldVariant::new("email", "String", false, ComponentsBehaviour::Input).with_layout(
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
