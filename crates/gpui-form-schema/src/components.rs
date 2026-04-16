use strum::{Display, EnumString, IntoStaticStr};

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

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
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
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct NumberInputBehaviour {
    /// Optional `as = ...` override used for numeric validation semantics.
    pub validation_type: Option<&'static str>,
    pub kind: NumberInputKind,
}

#[derive(Clone, Debug, Display, EnumString, Eq, IntoStaticStr, PartialEq)]
#[strum(serialize_all = "snake_case")]
pub enum ComponentsBehaviour {
    Input,
    NumberInput(NumberInputBehaviour),
    Checkbox,
    Switch,
    Select(SelectBehaviour),
    InfiniteSelect(InfiniteSelectBehaviour),
    Custom,
    DatePicker,
}

impl ComponentsBehaviour {
    pub fn component_name(&self) -> &'static str {
        match self {
            Self::Input => "input",
            Self::NumberInput(_) => "number_input",
            Self::Checkbox => "checkbox",
            Self::Switch => "switch",
            Self::Select(_) => "select",
            Self::InfiniteSelect(_) => "infinite_select",
            Self::Custom => "custom",
            Self::DatePicker => "date_picker",
        }
    }

    pub fn is_value_only_field(&self) -> bool {
        matches!(self, Self::Checkbox | Self::Switch)
    }

    pub fn needs_value_field(&self) -> bool {
        matches!(self, Self::NumberInput(_))
    }

    pub fn partial(&self) -> bool {
        match self {
            Self::Select(options) => options.partial,
            _ => false,
        }
    }

    pub fn subscribable(&self) -> bool {
        matches!(
            self,
            Self::Input
                | Self::NumberInput(_)
                | Self::Select(_)
                | Self::InfiniteSelect(_)
                | Self::DatePicker
        )
    }

    pub fn focusable(&self) -> bool {
        matches!(
            self,
            Self::Input | Self::NumberInput(_) | Self::Select(_) | Self::InfiniteSelect(_)
        )
    }
}
