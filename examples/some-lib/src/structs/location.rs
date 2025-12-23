//! Example of nested tuple enums for location selection.
//!
//! This demonstrates a 3-level hierarchy: Country -> State/Province -> City
//! using the TupleEnumInner derive macro.

use es_fluent::{EsFluent, EsFluentKv};
use gpui_form::TupleEnumInner;
use strum::EnumIter;

// ============================================================================
// Level 3: Cities (leaf nodes - no inner values)
// ============================================================================

#[derive(Clone, Debug, Default, EnumIter, EsFluent, PartialEq, TupleEnumInner)]
#[fluent(this)]
pub enum CaliforniaCity {
    #[default]
    LosAngeles,
    SanFrancisco,
    SanDiego,
    SanJose,
    Sacramento,
}

#[derive(Clone, Debug, Default, EnumIter, EsFluent, PartialEq, TupleEnumInner)]
#[fluent(this)]
pub enum TexasCity {
    #[default]
    Houston,
    Dallas,
    Austin,
    SanAntonio,
    FortWorth,
}

#[derive(Clone, Debug, Default, EnumIter, EsFluent, PartialEq, TupleEnumInner)]
#[fluent(this)]
pub enum NewYorkCity {
    #[default]
    NewYorkCity,
    Buffalo,
    Rochester,
    Albany,
    Syracuse,
}

#[derive(Clone, Debug, Default, EnumIter, EsFluent, PartialEq, TupleEnumInner)]
#[fluent(this)]
pub enum OntarioCity {
    #[default]
    Toronto,
    Ottawa,
    Mississauga,
    Hamilton,
    London,
}

#[derive(Clone, Debug, Default, EnumIter, EsFluent, PartialEq, TupleEnumInner)]
#[fluent(this)]
pub enum QuebecCity {
    #[default]
    Montreal,
    QuebecCity,
    Laval,
    Gatineau,
    Longueuil,
}

#[derive(Clone, Debug, Default, EnumIter, EsFluent, PartialEq, TupleEnumInner)]
#[fluent(this)]
pub enum BritishColumbiaCity {
    #[default]
    Vancouver,
    Victoria,
    Surrey,
    Burnaby,
    Richmond,
}

// ============================================================================
// Level 2: States/Provinces (contain cities)
// ============================================================================

#[derive(Clone, Debug, EnumIter, EsFluent, EsFluentKv, PartialEq, TupleEnumInner)]
#[fluent(this)]
#[fluent_kv(keys = ["description", "label"], keys_this)]
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

#[derive(Clone, Debug, EnumIter, EsFluent, EsFluentKv, PartialEq, TupleEnumInner)]
#[fluent(this)]
#[fluent_kv(keys = ["description", "label"], keys_this)]
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

// ============================================================================
// Level 1: Countries (contain states/provinces)
// ============================================================================

#[derive(Clone, Debug, EnumIter, EsFluent, EsFluentKv, PartialEq, TupleEnumInner)]
#[fluent(this)]
#[fluent_kv(keys = ["description", "label"], keys_this)]
pub enum Country {
    USA(USAState),
    Canada(CanadaProvince),
}

impl Default for Country {
    fn default() -> Self {
        Self::USA(USAState::default())
    }
}

// ============================================================================
// Form struct using the nested tuple enum
// ============================================================================

use garde::Validate;
use gpui_form::GpuiForm;

/// A form that demonstrates tuple select with nested enums.
#[derive(Clone, Debug, Default, EsFluentKv, GpuiForm, Validate)]
#[fluent_kv(keys = ["description", "label"], this)] // rn "this" will also do "keys_this", new update of es-fluent will fix this
pub struct LocationForm {
    /// User's name
    #[gpui_form(component(input))]
    #[garde(length(min = 1, max = 100))]
    pub name: String,

    /// Location selection using cascading selects
    #[gpui_form(component(tuple_select(default)))]
    #[garde(skip)]
    pub location: Country,
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_form_component::{TupleEnumInner, TupleSelectPath, build_from_path, get_max_depth};

    #[test]
    fn test_depth_calculation() {
        // Cities are at depth 1 (leaf)
        assert_eq!(CaliforniaCity::depth(), 1);
        assert_eq!(TexasCity::depth(), 1);

        // States are at depth 2 (contain cities)
        assert_eq!(USAState::depth(), 2);
        assert_eq!(CanadaProvince::depth(), 2);

        // Countries are at depth 3 (contain states which contain cities)
        assert_eq!(Country::depth(), 3);
    }

    #[test]
    fn test_variant_names() {
        let usa = Country::USA(USAState::default());
        assert_eq!(usa.variant_name(), "USA");

        let california = USAState::California(CaliforniaCity::default());
        assert_eq!(california.variant_name(), "California");

        let la = CaliforniaCity::LosAngeles;
        assert_eq!(la.variant_name(), "LosAngeles");
    }

    #[test]
    fn test_has_inner() {
        let usa = Country::USA(USAState::default());
        assert!(usa.has_inner());

        let la = CaliforniaCity::LosAngeles;
        assert!(!la.has_inner());
    }

    #[test]
    fn test_child_variant_names() {
        let usa = Country::USA(USAState::default());
        let child_names = usa.child_variant_names();
        assert_eq!(child_names.len(), 3);
        assert_eq!(child_names[0], "California");
        assert_eq!(child_names[1], "Texas");
        assert_eq!(child_names[2], "NewYork");

        let canada = Country::Canada(CanadaProvince::default());
        let child_names = canada.child_variant_names();
        assert_eq!(child_names.len(), 3);
        assert_eq!(child_names[0], "Ontario");
        assert_eq!(child_names[1], "Quebec");
        assert_eq!(child_names[2], "BritishColumbia");
    }

    #[test]
    fn test_set_child_by_index() {
        let usa = Country::USA(USAState::default());

        // Change to Texas
        let updated = usa.set_child_by_index(1).unwrap();
        match updated {
            Country::USA(state) => assert_eq!(state.variant_name(), "Texas"),
            _ => panic!("Expected USA variant"),
        }
    }

    #[test]
    fn test_variants() {
        let countries = Country::variants();
        assert_eq!(countries.len(), 2);
        assert_eq!(countries[0].variant_name(), "USA");
        assert_eq!(countries[1].variant_name(), "Canada");

        let states = USAState::variants();
        assert_eq!(states.len(), 3);
        assert_eq!(states[0].variant_name(), "California");
        assert_eq!(states[1].variant_name(), "Texas");
        assert_eq!(states[2].variant_name(), "NewYork");
    }

    #[test]
    fn test_path_building() {
        // Build USA -> Texas -> Austin
        let mut path = TupleSelectPath::new();
        path.set(0, 0); // USA
        path.set(1, 1); // Texas
        path.set(2, 2); // Austin

        let country: Country = build_from_path(&path).unwrap();
        assert_eq!(country.variant_name(), "USA");

        // Verify the nested structure
        match country {
            Country::USA(state) => {
                assert_eq!(state.variant_name(), "Texas");
                match state {
                    USAState::Texas(city) => {
                        assert_eq!(city.variant_name(), "Austin");
                    },
                    _ => panic!("Expected Texas"),
                }
            },
            _ => panic!("Expected USA"),
        }
    }

    #[test]
    fn test_max_depth() {
        assert_eq!(get_max_depth::<Country>(), 3);
        assert_eq!(get_max_depth::<USAState>(), 2);
        assert_eq!(get_max_depth::<CaliforniaCity>(), 1);
    }

    #[test]
    fn test_child_depth() {
        let usa = Country::USA(USAState::default());
        assert_eq!(usa.child_depth(), 2); // USAState has depth 2

        let california = USAState::California(CaliforniaCity::default());
        assert_eq!(california.child_depth(), 1); // CaliforniaCity has depth 1

        let la = CaliforniaCity::LosAngeles;
        assert_eq!(la.child_depth(), 0); // Leaf node
    }
}
