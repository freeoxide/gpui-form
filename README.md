[![Build Status](https://github.com/stayhydated/gpui-form/actions/workflows/ci.yml/badge.svg)](https://github.com/stayhydated/gpui-form/actions/workflows/ci.yml)
[![Docs](https://docs.rs/gpui-form/badge.svg)](https://docs.rs/gpui-form/)
[![Crates.io](https://img.shields.io/crates/v/gpui-form.svg)](https://crates.io/crates/gpui-form)

A struct derive macro for deriving [gpui-component](https://github.com/longbridge/gpui-component)... components on fields.

## Currently Supported components

- [Checkbox](https://longbridge.github.io/gpui-component/docs/components/checkbox)
- [Date Picker](https://longbridge.github.io/gpui-component/docs/components/date-picker)
- [Select](https://longbridge.github.io/gpui-component/docs/components/select)
- [Input](https://longbridge.github.io/gpui-component/docs/components/input)
- [Number Input](https://longbridge.github.io/gpui-component/docs/components/number-input)
- [Switch](https://longbridge.github.io/gpui-component/docs/components/switch)

## Custom components

- [Infinite Select](<>)

Compatibility of `gpui-form` versions:

| `gpui-form` | `gpui-component` |
| :------------ | :--------------- |
| **git** | |
| `master` | `main` |
| **crates.io** | |
| `0.5.x` | `0.5.x` |

## Showcase

```rs
use es_fluent::{EsFluent, EsFluentVariants, EsFluentThis};
use gpui_form::{GpuiForm, SelectItem};
use koruma::{Koruma, KorumaAllFluent};
use koruma_collection::{
    collection::NonEmptyValidation,
    format::EmailValidation,
    general::RequiredValidation,
    numeric::{PositiveValidation, RangeValidation},
    string::{PrefixValidation, SuffixValidation},
};
use strum::EnumIter;

#[derive(Clone, Debug, Default, EnumIter, EsFluent, PartialEq, SelectItem)]
#[select_item(fluent)]
pub enum PreferedLanguage {
    #[default]
    English,
    French,
    Chinese,
}

#[derive(Clone, Debug, Default, EnumIter, EsFluent, PartialEq, SelectItem)]
#[select_item(fluent)]
pub enum EnumCountry {
    #[default]
    UnitedStates,
    France,
    China,
}

#[derive(Clone, Debug, Default, EsFluentVariants, EsFluentThis, GpuiForm, Koruma, KorumaAllFluent)]
#[fluent_this(origin, members)]
#[fluent_variants(keys = ["description", "label"])]
#[gpui_form(koruma(fluent))]
pub struct User {
    #[gpui_form(component(input))]
    #[koruma(NonEmptyValidation::<_>, RequiredValidation::<Option<_>>, PrefixValidation::<_>(prefix = "Xx"), SuffixValidation::<_>(suffix = "xX"))]
    pub username: Option<String>,

    #[gpui_form(component(input))]
    #[koruma(EmailValidation::<_>)]
    pub email: String,

    #[gpui_form(component(number_input))]
    #[koruma(RangeValidation::<_>(min = 18, max = 167))]
    pub age: Option<u32>,

    #[gpui_form(component(number_input))]
    #[koruma(PositiveValidation::<_>)]
    pub balance: f64,

    #[gpui_form(component(checkbox))]
    pub subscribe_newsletter: bool,

    #[gpui_form(component(switch))]
    pub enable_notifications: bool,

    #[gpui_form(component(select(default)))]
    pub preferred: PreferedLanguage,

    #[gpui_form(component(select(searchable, index = EnumCountry::France)))]
    pub country: Option<EnumCountry>,

    #[gpui_form(component(date_picker))]
    pub birth_date: Option<chrono::NaiveDate>,

    #[gpui_form(skip)]
    #[fluent_variants(skip)]
    pub skip_me: bool,
}
```

## Prototyping

There's also a prototyping tool which you can customize to your needs (except the [gpui-form-prototyping-core](crates/gpui-form-prototyping-core), which you could fork)

see examples's [README.md](examples/README.md)
