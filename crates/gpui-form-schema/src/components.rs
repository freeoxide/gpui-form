use serde::{Deserialize, Serialize};
use strum::{Display, EnumString, IntoStaticStr};

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
pub enum ComponentKind {
    #[default]
    Input,
    NumberInput,
    PhoneInput,
    Checkbox,
    Switch,
    Select,
    InfiniteSelect,
    Custom,
    DatePicker,
    FilePicker,
    /// A field whose type is itself a `#[derive(GpuiForm)]` struct, composed
    /// through the nested form's own generated holder. Value-only: renders no
    /// leaf control.
    Nested,
    /// An editable `Vec<T>` field. Value-only: the derive emits holder
    /// add/remove/items operations, not per-item entities.
    List,
}

impl ComponentKind {
    pub fn component_name(self) -> &'static str {
        self.into()
    }

    pub const fn is_value_only_field(self) -> bool {
        matches!(
            self,
            Self::Checkbox | Self::Switch | Self::Nested | Self::List
        )
    }

    pub const fn needs_value_field(self) -> bool {
        matches!(self, Self::NumberInput)
    }

    pub const fn subscribable(self) -> bool {
        matches!(
            self,
            Self::Input
                | Self::NumberInput
                | Self::PhoneInput
                | Self::Select
                | Self::InfiniteSelect
                | Self::DatePicker
                | Self::FilePicker
        )
    }

    pub const fn focusable(self) -> bool {
        matches!(
            self,
            Self::Input
                | Self::NumberInput
                | Self::PhoneInput
                | Self::Select
                | Self::InfiniteSelect
                | Self::FilePicker
        )
    }

    pub const fn default_wraps_in_option(self) -> bool {
        matches!(
            self,
            Self::Input | Self::NumberInput | Self::PhoneInput | Self::FilePicker
        )
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SelectBehaviour {
    pub partial: bool,
    pub searchable: bool,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct InfiniteSelectBehaviour {
    pub searchable: bool,
    pub max_depth: Option<usize>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NumberInputKind {
    #[default]
    Float,
    SignedInteger,
    UnsignedInteger,
    Custom,
}

impl NumberInputKind {
    pub const fn is_unsigned(self) -> bool {
        matches!(self, Self::UnsignedInteger)
    }

    /// The one numeric-family classifier, shared by every emission path so the
    /// recorded kind can never disagree with the generated validation branch.
    /// `type_str` is the caller's type string: the `as = ...` override, else raw.
    pub fn classify(type_str: &str) -> Self {
        if type_str.starts_with('f') {
            Self::Float
        } else if type_str.starts_with('u') {
            Self::UnsignedInteger
        } else if type_str.starts_with('i') {
            Self::SignedInteger
        } else {
            Self::Custom
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct NumberInputBehaviour {
    /// Optional `as = ...` override used for numeric validation semantics.
    pub validation_type: Option<&'static str>,
    /// The numeric family this field validates as, produced by
    /// [`NumberInputKind::classify`] and carried through the schema export to
    /// tooling and prototyping.
    pub kind: NumberInputKind,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PhoneInputBehaviour {
    /// When set, the phone number must match the country selected by this
    /// sibling field (`phone_input(country = <field>)`); `None` accepts any
    /// globally valid number. Cross-field enforcement is app/render-layer.
    pub country_field: Option<&'static str>,
}

/// Behaviour payload for `component(nested)`: `form_struct` names the nested
/// form struct so generators can resolve its generated types by naming
/// convention. Stays `Copy` — behaviours box nothing (the inventory `const` contract).
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct NestedBehaviour {
    /// The nested form struct name (e.g. `"Address"`), not a type path.
    pub form_struct: &'static str,
}

/// Behaviour payload for `component(list(item = ...))`: the scalar item kind,
/// with `item_struct` reserved for nested item forms (not shipped in v1).
/// Stays `Copy` — behaviours box nothing (the inventory `const` contract).
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ListBehaviour {
    /// The scalar component kind rendered per item (`item = input`, ...).
    pub item: ComponentKind,
    /// The nested item form struct name; always `None` in v1 (nested item
    /// forms are not shipped yet).
    pub item_struct: Option<&'static str>,
    /// Optional add-button label override from `list(add_label = "...")`;
    /// generators fall back to the `{form}.{field}_add` i18n key.
    pub add_label: Option<&'static str>,
}

#[derive(Clone, Debug, Display, EnumString, Eq, IntoStaticStr, PartialEq)]
#[strum(serialize_all = "snake_case")]
pub enum ComponentsBehaviour {
    Input,
    NumberInput(NumberInputBehaviour),
    PhoneInput(PhoneInputBehaviour),
    Checkbox,
    Switch,
    Select(SelectBehaviour),
    InfiniteSelect(InfiniteSelectBehaviour),
    Custom,
    DatePicker,
    FilePicker,
    Nested(NestedBehaviour),
    List(ListBehaviour),
}

impl ComponentsBehaviour {
    pub const fn kind(&self) -> ComponentKind {
        match self {
            Self::Input => ComponentKind::Input,
            Self::NumberInput(_) => ComponentKind::NumberInput,
            Self::PhoneInput(_) => ComponentKind::PhoneInput,
            Self::Checkbox => ComponentKind::Checkbox,
            Self::Switch => ComponentKind::Switch,
            Self::Select(_) => ComponentKind::Select,
            Self::InfiniteSelect(_) => ComponentKind::InfiniteSelect,
            Self::Custom => ComponentKind::Custom,
            Self::DatePicker => ComponentKind::DatePicker,
            Self::FilePicker => ComponentKind::FilePicker,
            Self::Nested(_) => ComponentKind::Nested,
            Self::List(_) => ComponentKind::List,
        }
    }

    pub fn component_name(&self) -> &'static str {
        self.kind().component_name()
    }

    pub fn is_value_only_field(&self) -> bool {
        self.kind().is_value_only_field()
    }

    pub fn needs_value_field(&self) -> bool {
        self.kind().needs_value_field()
    }

    pub fn partial(&self) -> bool {
        match self {
            Self::Select(options) => options.partial,
            _ => false,
        }
    }

    pub fn subscribable(&self) -> bool {
        self.kind().subscribable()
    }

    pub fn focusable(&self) -> bool {
        self.kind().focusable()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ComponentKind, ComponentsBehaviour, ListBehaviour, NestedBehaviour, NumberInputKind,
        SelectBehaviour,
    };

    #[test]
    fn number_input_kind_classifies_the_shared_numeric_families() {
        assert_eq!(NumberInputKind::classify("f32"), NumberInputKind::Float);
        assert_eq!(NumberInputKind::classify("f64"), NumberInputKind::Float);
        assert!(!NumberInputKind::classify("f64").is_unsigned());

        for ty in ["u8", "u16", "u32", "u64", "u128", "usize"] {
            assert_eq!(
                NumberInputKind::classify(ty),
                NumberInputKind::UnsignedInteger,
                "{ty} must classify as the unsigned family"
            );
            assert!(NumberInputKind::classify(ty).is_unsigned());
        }

        for ty in ["i8", "i16", "i32", "i64", "i128", "isize"] {
            assert_eq!(
                NumberInputKind::classify(ty),
                NumberInputKind::SignedInteger,
                "{ty} must classify as the signed family"
            );
            assert!(!NumberInputKind::classify(ty).is_unsigned());
        }

        assert_eq!(
            NumberInputKind::classify("::rust_decimal::Decimal"),
            NumberInputKind::Custom
        );
        assert!(!NumberInputKind::classify("::rust_decimal::Decimal").is_unsigned());
    }

    #[test]
    fn nested_and_list_kinds_are_value_only_non_focusable_non_subscribable() {
        for kind in [ComponentKind::Nested, ComponentKind::List] {
            assert!(kind.is_value_only_field(), "{kind:?} must be value-only");
            assert!(
                !kind.subscribable(),
                "{kind:?} must not be subscribable in v1"
            );
            assert!(!kind.focusable(), "{kind:?} must not be focusable in v1");
            assert!(
                !kind.default_wraps_in_option(),
                "{kind:?} must not wrap in Option"
            );
            assert!(!kind.needs_value_field());
        }
    }

    #[test]
    fn component_names_are_snake_case_for_the_new_kinds() {
        assert_eq!(ComponentKind::Nested.component_name(), "nested");
        assert_eq!(ComponentKind::List.component_name(), "list");
    }

    #[test]
    fn behaviour_variants_round_trip_to_their_kinds() {
        let nested = ComponentsBehaviour::Nested(NestedBehaviour {
            form_struct: "Address",
        });
        assert_eq!(nested.kind(), ComponentKind::Nested);
        assert!(nested.is_value_only_field());

        let list = ComponentsBehaviour::List(ListBehaviour {
            item: ComponentKind::Input,
            item_struct: None,
            add_label: None,
        });
        assert_eq!(list.kind(), ComponentKind::List);
        assert_eq!(list.component_name(), "list");
        assert!(list.is_value_only_field());

        assert_eq!(
            ComponentsBehaviour::Select(SelectBehaviour::default()).kind(),
            ComponentKind::Select
        );
    }

    #[test]
    fn behaviour_payloads_stay_const_constructible_and_copy() {
        const NESTED: ComponentsBehaviour =
            ComponentsBehaviour::Nested(NestedBehaviour { form_struct: "A" });
        const LIST: ComponentsBehaviour = ComponentsBehaviour::List(ListBehaviour {
            item: ComponentKind::Input,
            item_struct: None,
            add_label: None,
        });
        assert_eq!(
            ListBehaviour::default(),
            ListBehaviour {
                item: ComponentKind::Input,
                item_struct: None,
                add_label: None,
            }
        );
        assert!(matches!(NESTED, ComponentsBehaviour::Nested(_)));
        assert!(matches!(LIST, ComponentsBehaviour::List(_)));
        fn assert_copy(_: impl Copy) {}
        assert_copy(NESTED.kind());
        assert_copy(ListBehaviour::default());
    }
}
