#![feature(prelude_import)]
#[macro_use]
extern crate std;
#[prelude_import]
use std::prelude::rust_2024::*;
pub mod i18n {
    #[folder = "/home/mark/Documents/GitHub/gpui-form/examples/some-lib/../i18n"]
    struct SomeLibI18nAssets;
    impl SomeLibI18nAssets {
        fn matcher() -> rust_embed::utils::PathMatcher {
            const INCLUDES: &[&str] = &[];
            const EXCLUDES: &[&str] = &[];
            static PATH_MATCHER: ::std::sync::OnceLock<rust_embed::utils::PathMatcher> = ::std::sync::OnceLock::new();
            PATH_MATCHER
                .get_or_init(|| rust_embed::utils::PathMatcher::new(INCLUDES, EXCLUDES))
                .clone()
        }
        /// Get an embedded file and its metadata.
        pub fn get(file_path: &str) -> ::std::option::Option<rust_embed::EmbeddedFile> {
            let rel_file_path = file_path.replace("\\", "/");
            let file_path = ::std::path::Path::new(
                    "/home/mark/Documents/GitHub/gpui-form/examples/some-lib/../i18n",
                )
                .join(&rel_file_path);
            let canonical_file_path = file_path.canonicalize().ok()?;
            if !canonical_file_path
                .starts_with("/home/mark/Documents/GitHub/gpui-form/examples/i18n")
            {
                let metadata = ::std::fs::symlink_metadata(&file_path).ok()?;
                if !metadata.is_symlink() {
                    return ::std::option::Option::None;
                }
            }
            let path_matcher = Self::matcher();
            if path_matcher.is_path_included(&rel_file_path) {
                rust_embed::utils::read_file_from_fs(&canonical_file_path).ok()
            } else {
                ::std::option::Option::None
            }
        }
        /// Iterates over the file paths in the folder.
        pub fn iter() -> impl ::std::iter::Iterator<
            Item = ::std::borrow::Cow<'static, str>,
        > {
            use ::std::path::Path;
            rust_embed::utils::get_files(
                    ::std::string::String::from(
                        "/home/mark/Documents/GitHub/gpui-form/examples/some-lib/../i18n",
                    ),
                    Self::matcher(),
                )
                .map(|e| ::std::borrow::Cow::from(e.rel_path))
        }
    }
    impl rust_embed::RustEmbed for SomeLibI18nAssets {
        fn get(file_path: &str) -> ::std::option::Option<rust_embed::EmbeddedFile> {
            SomeLibI18nAssets::get(file_path)
        }
        fn iter() -> rust_embed::Filenames {
            rust_embed::Filenames::Dynamic(
                ::std::boxed::Box::new(SomeLibI18nAssets::iter()),
            )
        }
    }
    impl es_fluent::__manager_core::EmbeddedAssets for SomeLibI18nAssets {
        fn domain() -> &'static str {
            "some-lib"
        }
    }
    static SOME_LIB_I18N_MODULE_DATA: es_fluent::__manager_core::EmbeddedModuleData = es_fluent::__manager_core::EmbeddedModuleData {
        name: "some-lib",
        domain: "some-lib",
        supported_languages: &[
            {
                #[allow(dead_code)]
                enum ProcMacroHack {
                    Value = ("\"en\"", 0).1,
                }
                unsafe {
                    ::unic_langid_macros::LanguageIdentifier::from_raw_parts_unchecked(
                        unsafe {
                            ::unic_langid_macros::subtags::Language::from_raw_unchecked(
                                28261u64,
                            )
                        },
                        None,
                        None,
                        None,
                    )
                }
            },
            {
                #[allow(dead_code)]
                enum ProcMacroHack {
                    Value = ("\"zh-CN\"", 0).1,
                }
                unsafe {
                    ::unic_langid_macros::LanguageIdentifier::from_raw_parts_unchecked(
                        unsafe {
                            ::unic_langid_macros::subtags::Language::from_raw_unchecked(
                                26746u64,
                            )
                        },
                        None,
                        Some(unsafe {
                            ::unic_langid_macros::subtags::Region::from_raw_unchecked(
                                20035u32,
                            )
                        }),
                        None,
                    )
                }
            },
            {
                #[allow(dead_code)]
                enum ProcMacroHack {
                    Value = ("\"fr\"", 0).1,
                }
                unsafe {
                    ::unic_langid_macros::LanguageIdentifier::from_raw_parts_unchecked(
                        unsafe {
                            ::unic_langid_macros::subtags::Language::from_raw_unchecked(
                                29286u64,
                            )
                        },
                        None,
                        None,
                        None,
                    )
                }
            },
        ],
    };
    #[allow(non_upper_case_globals)]
    const _: () = {
        static __INVENTORY: ::inventory::Node = ::inventory::Node {
            value: &{
                &es_fluent::__manager_core::EmbeddedI18nModule::<
                    SomeLibI18nAssets,
                >::new(&SOME_LIB_I18N_MODULE_DATA)
                    as &dyn es_fluent::__manager_core::I18nModule
            },
            next: ::inventory::core::cell::UnsafeCell::new(
                ::inventory::core::option::Option::None,
            ),
        };
        #[link_section = ".text.startup"]
        unsafe extern "C" fn __ctor() {
            unsafe { ::inventory::ErasedNode::submit(__INVENTORY.value, &__INVENTORY) }
        }
        #[used]
        #[link_section = ".init_array"]
        static __CTOR: unsafe extern "C" fn() = __ctor;
    };
}
pub mod structs {
    pub mod user {
        use es_fluent::{EsFluent, EsFluentKv};
        use garde::Validate;
        use gpui_form::{GpuiForm, SelectItem};
        use rust_decimal::Decimal;
        use strum::EnumIter;
        #[fluent(display = "std")]
        pub enum PreferedLanguage {
            #[default]
            English,
            French,
            Chinese,
        }
        #[automatically_derived]
        impl ::core::clone::Clone for PreferedLanguage {
            #[inline]
            fn clone(&self) -> PreferedLanguage {
                match self {
                    PreferedLanguage::English => PreferedLanguage::English,
                    PreferedLanguage::French => PreferedLanguage::French,
                    PreferedLanguage::Chinese => PreferedLanguage::Chinese,
                }
            }
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for PreferedLanguage {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::write_str(
                    f,
                    match self {
                        PreferedLanguage::English => "English",
                        PreferedLanguage::French => "French",
                        PreferedLanguage::Chinese => "Chinese",
                    },
                )
            }
        }
        #[automatically_derived]
        impl ::core::default::Default for PreferedLanguage {
            #[inline]
            fn default() -> PreferedLanguage {
                Self::English
            }
        }
        ///An iterator over the variants of [PreferedLanguage]
        #[allow(missing_copy_implementations)]
        pub struct PreferedLanguageIter {
            idx: usize,
            back_idx: usize,
            marker: ::core::marker::PhantomData<fn() -> ()>,
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for PreferedLanguageIter {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("PreferedLanguageIter").field("len", &self.len()).finish()
            }
        }
        #[automatically_derived]
        impl PreferedLanguageIter {
            fn get(&self, idx: usize) -> ::core::option::Option<PreferedLanguage> {
                match idx {
                    0usize => ::core::option::Option::Some(PreferedLanguage::English),
                    1usize => ::core::option::Option::Some(PreferedLanguage::French),
                    2usize => ::core::option::Option::Some(PreferedLanguage::Chinese),
                    _ => ::core::option::Option::None,
                }
            }
        }
        #[automatically_derived]
        impl ::strum::IntoEnumIterator for PreferedLanguage {
            type Iterator = PreferedLanguageIter;
            #[inline]
            fn iter() -> PreferedLanguageIter {
                PreferedLanguageIter {
                    idx: 0,
                    back_idx: 0,
                    marker: ::core::marker::PhantomData,
                }
            }
        }
        #[automatically_derived]
        impl Iterator for PreferedLanguageIter {
            type Item = PreferedLanguage;
            #[inline]
            fn next(&mut self) -> ::core::option::Option<<Self as Iterator>::Item> {
                self.nth(0)
            }
            #[inline]
            fn size_hint(&self) -> (usize, ::core::option::Option<usize>) {
                let t = if self.idx + self.back_idx >= 3usize {
                    0
                } else {
                    3usize - self.idx - self.back_idx
                };
                (t, Some(t))
            }
            #[inline]
            fn nth(
                &mut self,
                n: usize,
            ) -> ::core::option::Option<<Self as Iterator>::Item> {
                let idx = self.idx + n + 1;
                if idx + self.back_idx > 3usize {
                    self.idx = 3usize;
                    ::core::option::Option::None
                } else {
                    self.idx = idx;
                    PreferedLanguageIter::get(self, idx - 1)
                }
            }
        }
        #[automatically_derived]
        impl ExactSizeIterator for PreferedLanguageIter {
            #[inline]
            fn len(&self) -> usize {
                self.size_hint().0
            }
        }
        #[automatically_derived]
        impl DoubleEndedIterator for PreferedLanguageIter {
            #[inline]
            fn next_back(&mut self) -> ::core::option::Option<<Self as Iterator>::Item> {
                let back_idx = self.back_idx + 1;
                if self.idx + back_idx > 3usize {
                    self.back_idx = 3usize;
                    ::core::option::Option::None
                } else {
                    self.back_idx = back_idx;
                    PreferedLanguageIter::get(self, 3usize - self.back_idx)
                }
            }
        }
        #[automatically_derived]
        impl ::core::iter::FusedIterator for PreferedLanguageIter {}
        #[automatically_derived]
        impl Clone for PreferedLanguageIter {
            #[inline]
            fn clone(&self) -> PreferedLanguageIter {
                PreferedLanguageIter {
                    idx: self.idx,
                    back_idx: self.back_idx,
                    marker: self.marker.clone(),
                }
            }
        }
        impl ::std::fmt::Display for PreferedLanguage {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                match self {
                    Self::English => {
                        f.write_fmt(
                            format_args!(
                                "{0}",
                                ::es_fluent::localize("prefered_language-English", None),
                            ),
                        )
                    }
                    Self::French => {
                        f.write_fmt(
                            format_args!(
                                "{0}",
                                ::es_fluent::localize("prefered_language-French", None),
                            ),
                        )
                    }
                    Self::Chinese => {
                        f.write_fmt(
                            format_args!(
                                "{0}",
                                ::es_fluent::localize("prefered_language-Chinese", None),
                            ),
                        )
                    }
                }
            }
        }
        impl From<&PreferedLanguage> for ::es_fluent::FluentValue<'_> {
            fn from(value: &PreferedLanguage) -> Self {
                value.to_string().into()
            }
        }
        impl From<PreferedLanguage> for ::es_fluent::FluentValue<'_> {
            fn from(value: PreferedLanguage) -> Self {
                (&value).into()
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for PreferedLanguage {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for PreferedLanguage {
            #[inline]
            fn eq(&self, other: &PreferedLanguage) -> bool {
                let __self_discr = ::core::intrinsics::discriminant_value(self);
                let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                __self_discr == __arg1_discr
            }
        }
        impl gpui_component::select::SelectItem for PreferedLanguage {
            type Value = Self;
            fn title(&self) -> gpui::SharedString {
                self.to_string().into()
            }
            fn value(&self) -> &Self::Value {
                self
            }
        }
        #[fluent(display = "std")]
        pub enum EnumCountry {
            #[default]
            UnitedStates,
            France,
            China,
        }
        #[automatically_derived]
        impl ::core::clone::Clone for EnumCountry {
            #[inline]
            fn clone(&self) -> EnumCountry {
                match self {
                    EnumCountry::UnitedStates => EnumCountry::UnitedStates,
                    EnumCountry::France => EnumCountry::France,
                    EnumCountry::China => EnumCountry::China,
                }
            }
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for EnumCountry {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::fmt::Formatter::write_str(
                    f,
                    match self {
                        EnumCountry::UnitedStates => "UnitedStates",
                        EnumCountry::France => "France",
                        EnumCountry::China => "China",
                    },
                )
            }
        }
        #[automatically_derived]
        impl ::core::default::Default for EnumCountry {
            #[inline]
            fn default() -> EnumCountry {
                Self::UnitedStates
            }
        }
        ///An iterator over the variants of [EnumCountry]
        #[allow(missing_copy_implementations)]
        pub struct EnumCountryIter {
            idx: usize,
            back_idx: usize,
            marker: ::core::marker::PhantomData<fn() -> ()>,
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for EnumCountryIter {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.debug_struct("EnumCountryIter").field("len", &self.len()).finish()
            }
        }
        #[automatically_derived]
        impl EnumCountryIter {
            fn get(&self, idx: usize) -> ::core::option::Option<EnumCountry> {
                match idx {
                    0usize => ::core::option::Option::Some(EnumCountry::UnitedStates),
                    1usize => ::core::option::Option::Some(EnumCountry::France),
                    2usize => ::core::option::Option::Some(EnumCountry::China),
                    _ => ::core::option::Option::None,
                }
            }
        }
        #[automatically_derived]
        impl ::strum::IntoEnumIterator for EnumCountry {
            type Iterator = EnumCountryIter;
            #[inline]
            fn iter() -> EnumCountryIter {
                EnumCountryIter {
                    idx: 0,
                    back_idx: 0,
                    marker: ::core::marker::PhantomData,
                }
            }
        }
        #[automatically_derived]
        impl Iterator for EnumCountryIter {
            type Item = EnumCountry;
            #[inline]
            fn next(&mut self) -> ::core::option::Option<<Self as Iterator>::Item> {
                self.nth(0)
            }
            #[inline]
            fn size_hint(&self) -> (usize, ::core::option::Option<usize>) {
                let t = if self.idx + self.back_idx >= 3usize {
                    0
                } else {
                    3usize - self.idx - self.back_idx
                };
                (t, Some(t))
            }
            #[inline]
            fn nth(
                &mut self,
                n: usize,
            ) -> ::core::option::Option<<Self as Iterator>::Item> {
                let idx = self.idx + n + 1;
                if idx + self.back_idx > 3usize {
                    self.idx = 3usize;
                    ::core::option::Option::None
                } else {
                    self.idx = idx;
                    EnumCountryIter::get(self, idx - 1)
                }
            }
        }
        #[automatically_derived]
        impl ExactSizeIterator for EnumCountryIter {
            #[inline]
            fn len(&self) -> usize {
                self.size_hint().0
            }
        }
        #[automatically_derived]
        impl DoubleEndedIterator for EnumCountryIter {
            #[inline]
            fn next_back(&mut self) -> ::core::option::Option<<Self as Iterator>::Item> {
                let back_idx = self.back_idx + 1;
                if self.idx + back_idx > 3usize {
                    self.back_idx = 3usize;
                    ::core::option::Option::None
                } else {
                    self.back_idx = back_idx;
                    EnumCountryIter::get(self, 3usize - self.back_idx)
                }
            }
        }
        #[automatically_derived]
        impl ::core::iter::FusedIterator for EnumCountryIter {}
        #[automatically_derived]
        impl Clone for EnumCountryIter {
            #[inline]
            fn clone(&self) -> EnumCountryIter {
                EnumCountryIter {
                    idx: self.idx,
                    back_idx: self.back_idx,
                    marker: self.marker.clone(),
                }
            }
        }
        impl ::std::fmt::Display for EnumCountry {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                match self {
                    Self::UnitedStates => {
                        f.write_fmt(
                            format_args!(
                                "{0}",
                                ::es_fluent::localize("enum_country-UnitedStates", None),
                            ),
                        )
                    }
                    Self::France => {
                        f.write_fmt(
                            format_args!(
                                "{0}",
                                ::es_fluent::localize("enum_country-France", None),
                            ),
                        )
                    }
                    Self::China => {
                        f.write_fmt(
                            format_args!(
                                "{0}",
                                ::es_fluent::localize("enum_country-China", None),
                            ),
                        )
                    }
                }
            }
        }
        impl From<&EnumCountry> for ::es_fluent::FluentValue<'_> {
            fn from(value: &EnumCountry) -> Self {
                value.to_string().into()
            }
        }
        impl From<EnumCountry> for ::es_fluent::FluentValue<'_> {
            fn from(value: EnumCountry) -> Self {
                (&value).into()
            }
        }
        #[automatically_derived]
        impl ::core::marker::StructuralPartialEq for EnumCountry {}
        #[automatically_derived]
        impl ::core::cmp::PartialEq for EnumCountry {
            #[inline]
            fn eq(&self, other: &EnumCountry) -> bool {
                let __self_discr = ::core::intrinsics::discriminant_value(self);
                let __arg1_discr = ::core::intrinsics::discriminant_value(other);
                __self_discr == __arg1_discr
            }
        }
        impl gpui_component::select::SelectItem for EnumCountry {
            type Value = Self;
            fn title(&self) -> gpui::SharedString {
                self.to_string().into()
            }
            fn value(&self) -> &Self::Value {
                self
            }
        }
        #[fluent_kv(display = "std")]
        #[fluent_kv(this, keys = ["Description", "Label"])]
        pub struct User {
            #[gpui_form(component(input))]
            #[garde(length(min = 3, max = 50))]
            pub username: Option<String>,
            #[gpui_form(component(input))]
            #[garde(email)]
            pub email: String,
            #[gpui_form(component(number_input))]
            #[garde(range(min = 0, max = 150))]
            pub age: Option<u32>,
            #[gpui_form(component(number_input))]
            #[garde(range(min = Decimal::ZERO))]
            pub balance: Decimal,
            #[gpui_form(component(checkbox))]
            #[garde(skip)]
            pub subscribe_newsletter: bool,
            #[gpui_form(component(switch))]
            #[garde(skip)]
            pub enable_notifications: bool,
            #[gpui_form(component(select(default)))]
            #[garde(skip)]
            pub preferred: PreferedLanguage,
            #[gpui_form(component(select(searchable, index = EnumCountry::France)))]
            #[garde(skip)]
            pub country: Option<EnumCountry>,
            #[gpui_form(component(date_picker))]
            #[garde(skip)]
            pub birth_date: Option<chrono::NaiveDate>,
            #[gpui_form(skip)]
            #[garde(skip)]
            #[fluent_kv(skip)]
            pub skip_me: bool,
        }
        #[automatically_derived]
        impl ::core::clone::Clone for User {
            #[inline]
            fn clone(&self) -> User {
                User {
                    username: ::core::clone::Clone::clone(&self.username),
                    email: ::core::clone::Clone::clone(&self.email),
                    age: ::core::clone::Clone::clone(&self.age),
                    balance: ::core::clone::Clone::clone(&self.balance),
                    subscribe_newsletter: ::core::clone::Clone::clone(
                        &self.subscribe_newsletter,
                    ),
                    enable_notifications: ::core::clone::Clone::clone(
                        &self.enable_notifications,
                    ),
                    preferred: ::core::clone::Clone::clone(&self.preferred),
                    country: ::core::clone::Clone::clone(&self.country),
                    birth_date: ::core::clone::Clone::clone(&self.birth_date),
                    skip_me: ::core::clone::Clone::clone(&self.skip_me),
                }
            }
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for User {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                let names: &'static _ = &[
                    "username",
                    "email",
                    "age",
                    "balance",
                    "subscribe_newsletter",
                    "enable_notifications",
                    "preferred",
                    "country",
                    "birth_date",
                    "skip_me",
                ];
                let values: &[&dyn ::core::fmt::Debug] = &[
                    &self.username,
                    &self.email,
                    &self.age,
                    &self.balance,
                    &self.subscribe_newsletter,
                    &self.enable_notifications,
                    &self.preferred,
                    &self.country,
                    &self.birth_date,
                    &&self.skip_me,
                ];
                ::core::fmt::Formatter::debug_struct_fields_finish(
                    f,
                    "User",
                    names,
                    values,
                )
            }
        }
        #[automatically_derived]
        impl ::core::default::Default for User {
            #[inline]
            fn default() -> User {
                User {
                    username: ::core::default::Default::default(),
                    email: ::core::default::Default::default(),
                    age: ::core::default::Default::default(),
                    balance: ::core::default::Default::default(),
                    subscribe_newsletter: ::core::default::Default::default(),
                    enable_notifications: ::core::default::Default::default(),
                    preferred: ::core::default::Default::default(),
                    country: ::core::default::Default::default(),
                    birth_date: ::core::default::Default::default(),
                    skip_me: ::core::default::Default::default(),
                }
            }
        }
        pub enum UserDescriptionFtl {
            Username,
            Email,
            Age,
            Balance,
            SubscribeNewsletter,
            EnableNotifications,
            Preferred,
            Country,
            BirthDate,
        }
        impl ::std::fmt::Display for UserDescriptionFtl {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                match self {
                    Self::Username => {
                        f.write_fmt(
                            format_args!(
                                "{0}",
                                ::es_fluent::localize("user_description_ftl-username", None),
                            ),
                        )
                    }
                    Self::Email => {
                        f.write_fmt(
                            format_args!(
                                "{0}",
                                ::es_fluent::localize("user_description_ftl-email", None),
                            ),
                        )
                    }
                    Self::Age => {
                        f.write_fmt(
                            format_args!(
                                "{0}",
                                ::es_fluent::localize("user_description_ftl-age", None),
                            ),
                        )
                    }
                    Self::Balance => {
                        f.write_fmt(
                            format_args!(
                                "{0}",
                                ::es_fluent::localize("user_description_ftl-balance", None),
                            ),
                        )
                    }
                    Self::SubscribeNewsletter => {
                        f.write_fmt(
                            format_args!(
                                "{0}",
                                ::es_fluent::localize(
                                    "user_description_ftl-subscribe_newsletter",
                                    None,
                                ),
                            ),
                        )
                    }
                    Self::EnableNotifications => {
                        f.write_fmt(
                            format_args!(
                                "{0}",
                                ::es_fluent::localize(
                                    "user_description_ftl-enable_notifications",
                                    None,
                                ),
                            ),
                        )
                    }
                    Self::Preferred => {
                        f.write_fmt(
                            format_args!(
                                "{0}",
                                ::es_fluent::localize(
                                    "user_description_ftl-preferred",
                                    None,
                                ),
                            ),
                        )
                    }
                    Self::Country => {
                        f.write_fmt(
                            format_args!(
                                "{0}",
                                ::es_fluent::localize("user_description_ftl-country", None),
                            ),
                        )
                    }
                    Self::BirthDate => {
                        f.write_fmt(
                            format_args!(
                                "{0}",
                                ::es_fluent::localize(
                                    "user_description_ftl-birth_date",
                                    None,
                                ),
                            ),
                        )
                    }
                }
            }
        }
        impl UserDescriptionFtl {
            pub fn this_ftl() -> String {
                ::es_fluent::localize("user_description_ftl", None)
            }
        }
        impl From<&UserDescriptionFtl> for ::es_fluent::FluentValue<'_> {
            fn from(value: &UserDescriptionFtl) -> Self {
                value.to_string().into()
            }
        }
        impl From<UserDescriptionFtl> for ::es_fluent::FluentValue<'_> {
            fn from(value: UserDescriptionFtl) -> Self {
                (&value).into()
            }
        }
        pub enum UserLabelFtl {
            Username,
            Email,
            Age,
            Balance,
            SubscribeNewsletter,
            EnableNotifications,
            Preferred,
            Country,
            BirthDate,
        }
        impl ::std::fmt::Display for UserLabelFtl {
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                match self {
                    Self::Username => {
                        f.write_fmt(
                            format_args!(
                                "{0}",
                                ::es_fluent::localize("user_label_ftl-username", None),
                            ),
                        )
                    }
                    Self::Email => {
                        f.write_fmt(
                            format_args!(
                                "{0}",
                                ::es_fluent::localize("user_label_ftl-email", None),
                            ),
                        )
                    }
                    Self::Age => {
                        f.write_fmt(
                            format_args!(
                                "{0}",
                                ::es_fluent::localize("user_label_ftl-age", None),
                            ),
                        )
                    }
                    Self::Balance => {
                        f.write_fmt(
                            format_args!(
                                "{0}",
                                ::es_fluent::localize("user_label_ftl-balance", None),
                            ),
                        )
                    }
                    Self::SubscribeNewsletter => {
                        f.write_fmt(
                            format_args!(
                                "{0}",
                                ::es_fluent::localize(
                                    "user_label_ftl-subscribe_newsletter",
                                    None,
                                ),
                            ),
                        )
                    }
                    Self::EnableNotifications => {
                        f.write_fmt(
                            format_args!(
                                "{0}",
                                ::es_fluent::localize(
                                    "user_label_ftl-enable_notifications",
                                    None,
                                ),
                            ),
                        )
                    }
                    Self::Preferred => {
                        f.write_fmt(
                            format_args!(
                                "{0}",
                                ::es_fluent::localize("user_label_ftl-preferred", None),
                            ),
                        )
                    }
                    Self::Country => {
                        f.write_fmt(
                            format_args!(
                                "{0}",
                                ::es_fluent::localize("user_label_ftl-country", None),
                            ),
                        )
                    }
                    Self::BirthDate => {
                        f.write_fmt(
                            format_args!(
                                "{0}",
                                ::es_fluent::localize("user_label_ftl-birth_date", None),
                            ),
                        )
                    }
                }
            }
        }
        impl UserLabelFtl {
            pub fn this_ftl() -> String {
                ::es_fluent::localize("user_label_ftl", None)
            }
        }
        impl From<&UserLabelFtl> for ::es_fluent::FluentValue<'_> {
            fn from(value: &UserLabelFtl) -> Self {
                value.to_string().into()
            }
        }
        impl From<UserLabelFtl> for ::es_fluent::FluentValue<'_> {
            fn from(value: UserLabelFtl) -> Self {
                (&value).into()
            }
        }
        impl User {
            pub fn this_ftl() -> String {
                ::es_fluent::localize("user", None)
            }
        }
        pub struct UserFormValueHolder {
            pub username: String,
            pub email: String,
            pub age: u32,
            pub balance: Decimal,
            pub subscribe_newsletter: bool,
            pub enable_notifications: bool,
            pub preferred: PreferedLanguage,
            pub country: EnumCountry,
            pub birth_date: Option<chrono::NaiveDate>,
            pub skip_me: bool,
        }
        #[automatically_derived]
        impl ::core::clone::Clone for UserFormValueHolder {
            #[inline]
            fn clone(&self) -> UserFormValueHolder {
                UserFormValueHolder {
                    username: ::core::clone::Clone::clone(&self.username),
                    email: ::core::clone::Clone::clone(&self.email),
                    age: ::core::clone::Clone::clone(&self.age),
                    balance: ::core::clone::Clone::clone(&self.balance),
                    subscribe_newsletter: ::core::clone::Clone::clone(
                        &self.subscribe_newsletter,
                    ),
                    enable_notifications: ::core::clone::Clone::clone(
                        &self.enable_notifications,
                    ),
                    preferred: ::core::clone::Clone::clone(&self.preferred),
                    country: ::core::clone::Clone::clone(&self.country),
                    birth_date: ::core::clone::Clone::clone(&self.birth_date),
                    skip_me: ::core::clone::Clone::clone(&self.skip_me),
                }
            }
        }
        #[automatically_derived]
        impl ::core::fmt::Debug for UserFormValueHolder {
            #[inline]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                let names: &'static _ = &[
                    "username",
                    "email",
                    "age",
                    "balance",
                    "subscribe_newsletter",
                    "enable_notifications",
                    "preferred",
                    "country",
                    "birth_date",
                    "skip_me",
                ];
                let values: &[&dyn ::core::fmt::Debug] = &[
                    &self.username,
                    &self.email,
                    &self.age,
                    &self.balance,
                    &self.subscribe_newsletter,
                    &self.enable_notifications,
                    &self.preferred,
                    &self.country,
                    &self.birth_date,
                    &&self.skip_me,
                ];
                ::core::fmt::Formatter::debug_struct_fields_finish(
                    f,
                    "UserFormValueHolder",
                    names,
                    values,
                )
            }
        }
        #[automatically_derived]
        impl ::core::default::Default for UserFormValueHolder {
            #[inline]
            fn default() -> UserFormValueHolder {
                UserFormValueHolder {
                    username: ::core::default::Default::default(),
                    email: ::core::default::Default::default(),
                    age: ::core::default::Default::default(),
                    balance: ::core::default::Default::default(),
                    subscribe_newsletter: ::core::default::Default::default(),
                    enable_notifications: ::core::default::Default::default(),
                    preferred: ::core::default::Default::default(),
                    country: ::core::default::Default::default(),
                    birth_date: ::core::default::Default::default(),
                    skip_me: ::core::default::Default::default(),
                }
            }
        }
        impl From<User> for UserFormValueHolder
        where
            String: ::core::default::Default,
            u32: ::core::default::Default,
            EnumCountry: ::core::default::Default,
            chrono::NaiveDate: ::core::default::Default,
        {
            fn from(from: User) -> Self {
                Self {
                    username: from.username.unwrap_or_default(),
                    email: from.email,
                    age: from.age.unwrap_or_default(),
                    balance: from.balance,
                    subscribe_newsletter: from.subscribe_newsletter,
                    enable_notifications: from.enable_notifications,
                    preferred: from.preferred,
                    country: from.country.unwrap_or_default(),
                    birth_date: from.birth_date,
                    skip_me: from.skip_me,
                }
            }
        }
        impl From<UserFormValueHolder> for User {
            fn from(from: UserFormValueHolder) -> Self {
                Self {
                    username: Some(from.username),
                    email: from.email,
                    age: Some(from.age),
                    balance: from.balance,
                    subscribe_newsletter: from.subscribe_newsletter,
                    enable_notifications: from.enable_notifications,
                    preferred: from.preferred,
                    country: Some(from.country),
                    birth_date: from.birth_date,
                    skip_me: from.skip_me,
                }
            }
        }
        impl ::gpui_form::unwrapped::Unwrapped for User {
            type Unwrapped = UserFormValueHolder;
        }
        impl UserFormValueHolder {
            pub fn try_from(
                from: User,
            ) -> Result<Self, ::gpui_form::unwrapped::UnwrappedError> {
                Ok(Self {
                    username: from
                        .username
                        .ok_or(::gpui_form::unwrapped::UnwrappedError {
                            field_name: "username",
                        })?,
                    email: from.email,
                    age: from
                        .age
                        .ok_or(::gpui_form::unwrapped::UnwrappedError {
                            field_name: "age",
                        })?,
                    balance: from.balance,
                    subscribe_newsletter: from.subscribe_newsletter,
                    enable_notifications: from.enable_notifications,
                    preferred: from.preferred,
                    country: from
                        .country
                        .ok_or(::gpui_form::unwrapped::UnwrappedError {
                            field_name: "country",
                        })?,
                    birth_date: from.birth_date,
                    skip_me: from.skip_me,
                })
            }
        }
        pub struct UserFormFields {
            pub username_input: gpui::Entity<gpui_component::input::InputState>,
            pub email_input: gpui::Entity<gpui_component::input::InputState>,
            pub age_number_input: gpui::Entity<gpui_component::input::InputState>,
            pub balance_number_input: gpui::Entity<gpui_component::input::InputState>,
            pub preferred_select: gpui::Entity<
                gpui_component::select::SelectState<Vec<PreferedLanguage>>,
            >,
            pub country_select: gpui::Entity<
                gpui_component::select::SelectState<
                    gpui_component::select::SearchableVec<EnumCountry>,
                >,
            >,
            pub birth_date_date_picker: gpui::Entity<
                gpui_component::date_picker::DatePickerState,
            >,
        }
        pub struct UserFormComponents;
        impl UserFormComponents {
            pub fn username_input(
                window: &mut gpui::Window,
                cx: &mut gpui::Context<'_, gpui_component::input::InputState>,
            ) -> gpui_component::input::InputState {
                gpui_component::input::InputState::new(window, cx)
            }
            pub fn email_input(
                window: &mut gpui::Window,
                cx: &mut gpui::Context<'_, gpui_component::input::InputState>,
            ) -> gpui_component::input::InputState {
                gpui_component::input::InputState::new(window, cx)
            }
            pub fn age_number_input(
                window: &mut gpui::Window,
                cx: &mut gpui::Context<'_, gpui_component::input::InputState>,
            ) -> gpui_component::input::InputState {
                use ::gpui_form::NumRegex;
                gpui_component::input::InputState::new(window, cx)
                    .pattern(u32::validation_regex().clone())
            }
            pub fn balance_number_input(
                window: &mut gpui::Window,
                cx: &mut gpui::Context<'_, gpui_component::input::InputState>,
            ) -> gpui_component::input::InputState {
                use ::gpui_form::NumRegex;
                gpui_component::input::InputState::new(window, cx)
                    .pattern(Decimal::validation_regex().clone())
            }
            pub fn preferred_select(
                window: &mut gpui::Window,
                cx: &mut gpui::Context<
                    '_,
                    gpui_component::select::SelectState<Vec<PreferedLanguage>>,
                >,
            ) -> gpui_component::select::SelectState<Vec<PreferedLanguage>> {
                use strum::IntoEnumIterator as _;
                gpui_component::select::SelectState::new(
                    PreferedLanguage::iter().collect::<Vec<PreferedLanguage>>().into(),
                    Some(
                        gpui_component::IndexPath::new(
                            PreferedLanguage::iter()
                                .position(|x| x == PreferedLanguage::default())
                                .unwrap(),
                        ),
                    ),
                    window,
                    cx,
                )
            }
            pub fn country_select(
                window: &mut gpui::Window,
                cx: &mut gpui::Context<
                    '_,
                    gpui_component::select::SelectState<
                        gpui_component::select::SearchableVec<EnumCountry>,
                    >,
                >,
            ) -> gpui_component::select::SelectState<
                gpui_component::select::SearchableVec<EnumCountry>,
            > {
                use strum::IntoEnumIterator as _;
                gpui_component::select::SelectState::new(
                    EnumCountry::iter().collect::<Vec<EnumCountry>>().into(),
                    Some(
                        gpui_component::IndexPath::new(
                            EnumCountry::iter()
                                .position(|x| x == EnumCountry::France)
                                .unwrap(),
                        ),
                    ),
                    window,
                    cx,
                )
            }
            pub fn birth_date_date_picker(
                window: &mut gpui::Window,
                cx: &mut gpui::Context<'_, gpui_component::date_picker::DatePickerState>,
            ) -> gpui_component::date_picker::DatePickerState {
                gpui_component::date_picker::DatePickerState::new(window, cx)
            }
        }
        impl ::garde::Validate for User {
            type Context = ();
            #[allow(clippy::needless_borrow)]
            fn validate_into(
                &self,
                __garde_user_ctx: &Self::Context,
                mut __garde_path: &mut dyn FnMut() -> ::garde::Path,
                __garde_report: &mut ::garde::error::Report,
            ) {
                let __garde_user_ctx = &__garde_user_ctx;
                {
                    let Self { age, balance, email, username, .. } = self;
                    {
                        {
                            let mut __garde_path = ::garde::util::__make_nested_path(
                                &mut __garde_path,
                                &"age",
                            );
                            let __garde_binding = &*age;
                            {
                                if let Err(__garde_error) = (::garde::rules::range::apply)(
                                    &*__garde_binding,
                                    (Some(0), Some(150)),
                                ) {
                                    __garde_report.append(__garde_path(), __garde_error);
                                }
                            }
                        }
                        {
                            let mut __garde_path = ::garde::util::__make_nested_path(
                                &mut __garde_path,
                                &"balance",
                            );
                            let __garde_binding = &*balance;
                            {
                                if let Err(__garde_error) = (::garde::rules::range::apply)(
                                    &*__garde_binding,
                                    (Some(Decimal::ZERO), None),
                                ) {
                                    __garde_report.append(__garde_path(), __garde_error);
                                }
                            }
                        }
                        {
                            let mut __garde_path = ::garde::util::__make_nested_path(
                                &mut __garde_path,
                                &"email",
                            );
                            let __garde_binding = &*email;
                            {
                                if let Err(__garde_error) = (::garde::rules::email::apply)(
                                    &*__garde_binding,
                                    (),
                                ) {
                                    __garde_report.append(__garde_path(), __garde_error);
                                }
                            }
                        }
                        {
                            let mut __garde_path = ::garde::util::__make_nested_path(
                                &mut __garde_path,
                                &"username",
                            );
                            let __garde_binding = &*username;
                            {
                                if let Err(__garde_error) = (::garde::rules::length::simple::apply)(
                                    &*__garde_binding,
                                    (3usize, 50usize),
                                ) {
                                    __garde_report.append(__garde_path(), __garde_error);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
