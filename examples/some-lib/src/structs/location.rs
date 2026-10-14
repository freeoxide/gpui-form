//! Nested infinite-select enums: a 3-level Country -> State/Province ->
//! City hierarchy for the `InfiniteSelect` derive.

use gpui_form::InfiniteSelect;
use strum::EnumIter;

#[derive(Clone, Debug, Default, EnumIter, InfiniteSelect, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CaliforniaCity {
    #[default]
    LosAngeles,
    SanFrancisco,
    SanDiego,
    SanJose,
    Sacramento,
}

#[derive(Clone, Debug, Default, EnumIter, InfiniteSelect, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TexasCity {
    #[default]
    Houston,
    Dallas,
    Austin,
    SanAntonio,
    FortWorth,
}

#[derive(Clone, Debug, Default, EnumIter, InfiniteSelect, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum NewYorkCity {
    #[default]
    NewYorkCity,
    Buffalo,
    Rochester,
    Albany,
    Syracuse,
}

#[derive(Clone, Debug, Default, EnumIter, InfiniteSelect, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum OntarioCity {
    #[default]
    Toronto,
    Ottawa,
    Mississauga,
    Hamilton,
    London,
}

#[derive(Clone, Debug, Default, EnumIter, InfiniteSelect, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum QuebecCity {
    #[default]
    Montreal,
    QuebecCity,
    Laval,
    Gatineau,
    Longueuil,
}

#[derive(Clone, Debug, Default, EnumIter, InfiniteSelect, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum BritishColumbiaCity {
    #[default]
    Vancouver,
    Victoria,
    Surrey,
    Burnaby,
    Richmond,
}

#[derive(Clone, Debug, EnumIter, InfiniteSelect, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum USAState {
    California(CaliforniaCity),
    Texas(TexasCity),
    NewYork(NewYorkCity),
}

impl Default for USAState {
    fn default() -> Self {
        Self::California(CaliforniaCity::default())
    }
}

#[derive(Clone, Debug, EnumIter, InfiniteSelect, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CanadaProvince {
    Ontario(OntarioCity),
    Quebec(QuebecCity),
    BritishColumbia(BritishColumbiaCity),
}

impl Default for CanadaProvince {
    fn default() -> Self {
        Self::Ontario(OntarioCity::default())
    }
}

#[derive(Clone, Debug, EnumIter, InfiniteSelect, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Country {
    USA(USAState),
    Canada { province: CanadaProvince },
}

impl Default for Country {
    fn default() -> Self {
        Self::USA(USAState::default())
    }
}

use gpui_form::GpuiForm;

/// A form that demonstrates tuple select with nested enums.
#[derive(Clone, Debug, Default, GpuiForm)]
pub struct LocationForm {
    #[gpui_form(component(input))]
    pub name: String,

    #[gpui_form(component(infinite_select))]
    pub location: Country,
}
