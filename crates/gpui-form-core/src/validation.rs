//! Pure, GPUI-free generic contracts over per-field validation errors.
//!
//! The concrete, typed error map for a form is *generated* by the derive
//! (`<Name>FormFieldErrors`, re-keying koruma's typed per-field error structs
//! by `<Name>FormPath`). This module owns only the form-agnostic contract so
//! GPUI glue, tests, and application code can be generic over any form's
//! error map using the shared [`crate::FieldPath`] primitive.

use crate::FieldPath;

/// Form-agnostic view of a per-form field-error map.
///
/// Implemented by the generated `<Name>FormFieldErrors` for every koruma
/// form, so generic code (render all failing fields, attach errors to
/// path-keyed UI state) can consume any form's errors through the shared
/// [`FieldPath`] primitive without knowing the form's typed accessors.
///
/// `failing_paths` returns failing field paths in declaration order.
pub trait FormFieldErrors {
    /// Whether no field has errors (`true` == validation would succeed).
    fn is_empty(&self) -> bool;

    /// Failing field paths in declaration order.
    fn failing_paths(&self) -> Vec<FieldPath>;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct ManualErrors {
        failing: Vec<&'static str>,
    }

    impl FormFieldErrors for ManualErrors {
        fn is_empty(&self) -> bool {
            self.failing.is_empty()
        }

        fn failing_paths(&self) -> Vec<FieldPath> {
            self.failing
                .iter()
                .map(|name| FieldPath::new(&[name]))
                .collect()
        }
    }

    #[test]
    fn manual_impl_satisfies_the_trait() {
        let empty = ManualErrors { failing: vec![] };
        assert!(empty.is_empty());
        assert!(empty.failing_paths().is_empty());

        let failing = ManualErrors {
            failing: vec!["username", "age"],
        };
        assert!(!failing.is_empty());
        let paths = failing.failing_paths();
        assert_eq!(paths[0].segments(), &["username"]);
        assert_eq!(paths[1].to_string(), "age");

        fn assert_generic<E: FormFieldErrors>(errors: &E) -> bool {
            errors.is_empty()
        }
        assert!(!assert_generic(&failing));
    }
}
