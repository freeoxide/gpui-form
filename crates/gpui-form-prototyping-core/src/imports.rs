//! Per-item import tracking for generated prototyping output: each field
//! generator declares required items as [`ImportItem`] slices, merged into an
//! [`ImportSet`] that emits minimal, deduplicated `use` statements.
use std::collections::BTreeMap;

use proc_macro2::TokenStream;
use quote::quote;

use crate::error::{PrototypingError, PrototypingResult};

/// The alias applied to an imported item.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Alias {
    /// Wildcard discard: `use X as _`, for trait imports that only need to be
    /// in scope.
    Anonymous,
    /// Named rename: `use X as Foo`.
    Rename(&'static str),
}

/// A single item to be imported into a generated file. Paths are `&'static
/// str` (built-in literals or custom-component metadata); [`ImportItem::owned_path`]
/// composes a path at generation time (the nested-form scaffold import).
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ImportItem {
    /// Full path to the imported item, e.g. `"::gpui_kit::component::checkbox::Checkbox"`.
    pub path: &'static str,
    /// Optional alias applied to the import.
    pub alias: Option<Alias>,
}

impl ImportItem {
    /// An import with no alias.
    pub const fn path(path: &'static str) -> Self {
        Self { path, alias: None }
    }

    /// An import with an alias (e.g. [`Alias::Anonymous`] for trait imports).
    pub const fn aliased(path: &'static str, alias: Alias) -> Self {
        Self {
            path,
            alias: Some(alias),
        }
    }

    /// An import whose path is composed at generation time (the nested
    /// form's scaffold import `crate::<snake>::<Nested>Form`). The path is
    /// leaked into `'static` storage and reclaimed at process exit.
    pub fn owned_path(path: String) -> Self {
        Self {
            path: Box::leak(path.into_boxed_str()),
            alias: None,
        }
    }
}

/// A deduplicated, ordered collection of [`ImportItem`]s that can be rendered
/// into grouped `use` statements.
#[derive(Default)]
pub struct ImportSet(std::collections::BTreeSet<ImportItem>);

impl ImportSet {
    /// Insert a single item.
    pub fn add(&mut self, item: ImportItem) {
        self.0.insert(item);
    }

    /// Insert a slice of items (convenience for per-generator const arrays).
    pub fn extend_items(&mut self, items: &[ImportItem]) {
        self.0.extend(items.iter().cloned());
    }

    /// Insert an iterator of items.
    pub fn extend(&mut self, items: impl IntoIterator<Item = ImportItem>) {
        for item in items {
            self.0.insert(item);
        }
    }

    /// Render all imports as grouped, sorted `use parent::{a, b as c};` token
    /// streams. Every entry is parsed before any token is emitted; all malformed
    /// entries are collected into one [`PrototypingError::InvalidImports`].
    pub fn to_token_stream(&self) -> PrototypingResult<TokenStream> {
        let mut grouped: BTreeMap<&str, Vec<(&'static str, Option<&Alias>)>> = BTreeMap::new();

        for item in &self.0 {
            let (parent, name) = item.path.rsplit_once("::").unwrap_or(("", item.path));
            grouped
                .entry(parent)
                .or_default()
                .push((name, item.alias.as_ref()));
        }

        let mut errors: Vec<PrototypingError> = Vec::new();
        let mut parsed_groups: Vec<(syn::Path, Vec<TokenStream>)> = Vec::new();
        for (parent, items) in &grouped {
            if parent.is_empty() {
                continue;
            }

            let parent_path: syn::Path = match syn::parse_str(parent) {
                Ok(path) => path,
                Err(error) => {
                    errors.push(PrototypingError::InvalidPath {
                        kind: "import parent path",
                        value: (*parent).to_string(),
                        error: error.to_string(),
                    });
                    continue;
                },
            };

            let mut item_tokens: Vec<TokenStream> = Vec::with_capacity(items.len());
            for (name, alias) in items {
                let name_ident: syn::Ident = match syn::parse_str(name) {
                    Ok(ident) => ident,
                    Err(error) => {
                        errors.push(PrototypingError::InvalidPath {
                            kind: "import item name",
                            value: (*name).to_string(),
                            error: error.to_string(),
                        });
                        continue;
                    },
                };
                enum RenderedAlias {
                    Anonymous,
                    Named(syn::Ident),
                    None,
                }
                let rendered_alias = match alias {
                    Some(Alias::Anonymous) => RenderedAlias::Anonymous,
                    Some(Alias::Rename(alias)) => match syn::parse_str::<syn::Ident>(alias) {
                        Ok(ident) => RenderedAlias::Named(ident),
                        Err(error) => {
                            errors.push(PrototypingError::InvalidPath {
                                kind: "import alias",
                                value: (*alias).to_string(),
                                error: error.to_string(),
                            });
                            continue;
                        },
                    },
                    None => RenderedAlias::None,
                };
                item_tokens.push(match rendered_alias {
                    RenderedAlias::Anonymous => quote! { #name_ident as _ },
                    RenderedAlias::Named(alias) => quote! { #name_ident as #alias },
                    RenderedAlias::None => quote! { #name_ident },
                });
            }
            parsed_groups.push((parent_path, item_tokens));
        }

        if !errors.is_empty() {
            return Err(PrototypingError::InvalidImports { errors });
        }

        let mut tokens = TokenStream::new();
        for (parent_path, item_tokens) in &parsed_groups {
            if item_tokens.len() == 1 {
                let single = &item_tokens[0];
                tokens.extend(quote! { use #parent_path::#single; });
            } else {
                tokens.extend(quote! { use #parent_path::{#(#item_tokens),*}; });
            }
        }

        Ok(tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::{Alias, ImportItem, ImportSet};

    fn compact(input: &str) -> String {
        input.chars().filter(|c| !c.is_whitespace()).collect()
    }

    #[test]
    fn malformed_parent_paths_return_a_typed_error_instead_of_panicking() {
        let mut set = ImportSet::default();
        set.add(ImportItem::owned_path(
            "crate::address book::Address BookForm".to_string(),
        ));

        let error = set
            .to_token_stream()
            .expect_err("malformed import parents must return a typed error, not panic");
        match &error {
            crate::error::PrototypingError::InvalidImports { errors } => {
                assert!(
                    errors.iter().any(|error| matches!(
                        error,
                        crate::error::PrototypingError::InvalidPath { kind, value, .. }
                            if *kind == "import parent path" && value == "crate::address book"
                    )),
                    "the offending parent path must be reported: {error}"
                );
            },
            other => panic!("unexpected error: {other}"),
        }
    }

    #[test]
    fn malformed_item_names_and_aliases_return_typed_errors() {
        let mut set = ImportSet::default();
        set.add(ImportItem::path("::gpui_kit::component::button::"));
        set.add(ImportItem::aliased(
            "::gpui::prelude::FluentBuilder",
            Alias::Rename("not an ident"),
        ));

        let error = set
            .to_token_stream()
            .expect_err("malformed item names/aliases must return a typed error, not panic");

        let crate::error::PrototypingError::InvalidImports { errors } = error else {
            panic!("expected InvalidImports");
        };
        assert!(
            errors.iter().any(|error| matches!(
                error,
                crate::error::PrototypingError::InvalidPath { kind, value, .. }
                    if *kind == "import item name" && value.is_empty()
            )),
            "the empty item name must be reported: {errors:?}"
        );
        assert!(
            errors.iter().any(|error| matches!(
                error,
                crate::error::PrototypingError::InvalidPath { kind, value, .. }
                    if *kind == "import alias" && *value == "not an ident"
            )),
            "the malformed alias must be reported: {errors:?}"
        );
    }

    #[test]
    fn all_malformed_entries_are_collected_not_just_the_first() {
        let mut set = ImportSet::default();
        set.add(ImportItem::path(
            "::gpui_kit::component::bad button::Button",
        ));
        set.add(ImportItem::path(
            "::gpui_kit::component::other widget::Switch",
        ));

        let error = set
            .to_token_stream()
            .expect_err("malformed imports must return a typed error, not panic");

        let crate::error::PrototypingError::InvalidImports { errors } = error else {
            panic!("expected InvalidImports");
        };
        assert_eq!(
            errors.len(),
            2,
            "every offender must be reported in one pass: {errors:?}"
        );
    }

    #[test]
    fn valid_import_sets_tokenize_as_before() {
        let mut set = ImportSet::default();
        set.add(ImportItem::path("::gpui_kit::component::button::Button"));
        set.add(ImportItem::path("::gpui_kit::component::h_flex"));
        set.add(ImportItem::aliased(
            "::gpui::prelude::FluentBuilder",
            Alias::Anonymous,
        ));

        let tokens = set.to_token_stream().expect("valid imports must tokenize");
        let compact = compact(&tokens.to_string());

        assert!(
            compact.contains("use::gpui_kit::component::button::Button;")
                && compact.contains("use::gpui_kit::component::h_flex;"),
            "grouped use statements keep their sorted shape: {compact}"
        );
        assert!(
            compact.contains("use::gpui::prelude::FluentBuilderas_;"),
            "anonymous aliases keep `as _`: {compact}"
        );
    }
}
