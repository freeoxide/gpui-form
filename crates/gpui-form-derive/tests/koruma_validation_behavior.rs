//! Runtime behavior of koruma validators re-emitted onto the generated
//! `...FormValueHolder`: builder-chain arguments (bounds) survive codegen, the
//! synthetic `RequiredValidation` fires for absent values, and koruma
//! attributes wrapped in `cfg_attr` reach the holder through the derive's
//! cfg_attr flattening.

use gpui_form_derive::GpuiForm;

#[derive(GpuiForm, koruma::Koruma)]
struct SeatsForm {
    #[gpui_form(component(number_input))]
    #[koruma(koruma_collection::numeric::RangeValidation::<_>::builder().min(1).max(12))]
    seats: u32,
}

#[derive(GpuiForm, koruma::Koruma)]
struct QuantityForm {
    #[gpui_form(component(number_input))]
    #[koruma(koruma_collection::numeric::PositiveValidation::<_>::builder())]
    quantity: u32,
}

#[derive(GpuiForm, koruma::Koruma)]
#[gpui_form(koruma)]
struct NameForm {
    #[gpui_form(component(input))]
    name: String,
}

#[derive(GpuiForm, koruma::Koruma)]
struct CfgAttrForm {
    #[cfg_attr(all(), gpui_form(component(number_input)))]
    #[cfg_attr(all(), koruma(koruma_collection::numeric::PositiveValidation::<_>::builder()))]
    amount: u32,
}

#[derive(GpuiForm, koruma::Koruma)]
#[gpui_form(koruma)]
struct OptionalOnlyForm {
    note: Option<String>,
    kind: Option<u8>,
}

#[test]
fn range_validation_bounds_survive_codegen() {
    assert!(
        SeatsFormFormValueHolder::default().validate().is_err(),
        "absent default must fail the synthetic required validation"
    );

    let bounded = |seats| SeatsFormFormValueHolder { seats: Some(seats) };
    assert!(bounded(1).validate().is_ok(), "min bound is inclusive");
    assert!(bounded(12).validate().is_ok(), "max bound is inclusive");
    assert!(bounded(5).validate().is_ok());
    assert!(
        bounded(13).validate().is_err(),
        "value above max must fail range validation"
    );
    assert!(
        bounded(0).validate().is_err(),
        "value below min must fail range validation"
    );
}

#[test]
fn zero_setter_builder_chain_is_honored() {
    let at_zero = QuantityFormFormValueHolder { quantity: Some(0) };
    assert!(
        at_zero.validate().is_err(),
        "zero must fail positive validation"
    );

    let at_five = QuantityFormFormValueHolder { quantity: Some(5) };
    assert!(at_five.validate().is_ok());
}

#[test]
fn non_optional_fields_get_synthetic_required_validation() {
    let absent = NameFormFormValueHolder::default();
    assert!(
        absent.validate().is_err(),
        "None must fail the synthetic required validation"
    );

    let present = NameFormFormValueHolder {
        name: Some("ada".to_string()),
    };
    assert!(present.validate().is_ok());
}

#[test]
fn cfg_attr_wrapped_attributes_reach_the_holder() {
    let at_zero = CfgAttrFormFormValueHolder { amount: Some(0) };
    assert!(
        at_zero.validate().is_err(),
        "validator inside cfg_attr must be re-emitted onto the holder"
    );

    let at_three = CfgAttrFormFormValueHolder { amount: Some(3) };
    assert!(at_three.validate().is_ok());
}

#[test]
fn koruma_without_validators_still_validates() {
    let holder = OptionalOnlyFormFormValueHolder::default();
    assert!(
        holder.validate().is_ok(),
        "koruma-enabled forms must expose validate() even with no validators"
    );
}
