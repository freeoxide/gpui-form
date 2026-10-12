//! Executor-free async per-field validation contract.
//!
//! Async validation (the "username taken?" class of checks that need a
//! server round-trip) is specified here as a *contract only*: an async fn in
//! a trait, path-keyed by the shared [`crate::FieldPath`] primitive. The
//! trait adds no dependency and no executor — `async fn` in traits is stable
//! at this crate's MSRV — so the contract belongs with the other headless
//! primitives and is implementable by server-side revalidation (tokio),
//! tests, and GPUI glue alike.
//!
//! # What is deliberately NOT here
//!
//! Anything that *runs*, times, or cancels a future: no clock, no debounce
//! timer, no cancellation token. Ownership of those stays with the driving
//! layer — the GPUI glue phase in `gpui-form-component` (which already hosts
//! all GPUI-facing runtime state) or the application. Natural drop-based
//! cancellation (dropping the in-flight future when a newer edit arrives)
//! needs no trait support; debounce is a driver policy. Storing this state
//! in core would violate its purity contract and be dead weight for
//! headless consumers.
//!
//! # No `Send` bound
//!
//! GPUI futures are typically non-`Send`, so the trait does not require
//! `Send` (parity-plan open question OQ 3). Tokio users who need to `spawn`
//! the future wrap it in a `Send` adapter of their own.
//!
//! # Relation to the derive surface
//!
//! The derive emits nothing for async validation: `AsyncFieldValidation`
//! impls are application-owned (they reference app services). The
//! `#[gpui_form(validate(async_fn = ...))]` attribute parses and records
//! *metadata only* in this milestone; wiring that metadata to execution,
//! merging async outcomes with the sync `<Name>FormFieldErrors` map, and
//! pending-state semantics for `is_valid` are deferred to the GPUI-glue
//! phase (OQ 5).

use crate::FieldPath;

/// Async per-field validation over a live form holder.
///
/// Implement this once per form (or per service) and return `Ok(())` for
/// fields the validator does not handle, so a single impl can serve a whole
/// form:
///
/// ```
/// use gpui_form_core::AsyncFieldValidation;
/// use gpui_form_core::{FieldPath, PathSegment};
///
/// struct UsernameTaken;
///
/// impl AsyncFieldValidation<UsernameTakenFormValueHolder> for UsernameTaken {
///     type Error = &'static str;
///
///     async fn validate_field(
///         &self,
///         holder: &UsernameTakenFormValueHolder,
///         field: &FieldPath,
///     ) -> Result<(), Self::Error> {
///         if field.segments() != [PathSegment::Name("username")] {
///             return Ok(()); // not my field
///         }
///         match holder.username.as_deref() {
///             Some("ada") => Err("username already taken"),
///             _ => Ok(()),
///         }
///     }
/// }
/// # #[derive(Default)]
/// # struct UsernameTakenFormValueHolder { username: Option<String> }
/// ```
///
/// The contract is stateless per call and fire-and-forget: the caller owns
/// the future, and driving/cancelling it is glue (see the module docs).
// No `Send` bound by design (GPUI futures are typically non-Send; parity
// plan OQ 3 defers any Send-bound twin), hence the plain async fn.
#[allow(async_fn_in_trait)]
pub trait AsyncFieldValidation<H> {
    /// The async failure type for this validator.
    type Error;

    /// Validate one field of the live holder.
    ///
    /// Return `Ok(())` for fields this validator does not handle. Paths are
    /// the shared [`FieldPath`] (flat single-segment field names, plus the
    /// nested/list composition segments from milestone M3 — see
    /// [`crate::path::PathSegment`]).
    async fn validate_field(&self, holder: &H, field: &FieldPath) -> Result<(), Self::Error>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::PathSegment;
    use std::future::{Future, poll_fn};
    use std::task::Poll;

    fn block_on<F: Future>(future: F) -> F::Output {
        let mut future = std::pin::pin!(future);
        let waker = std::task::Waker::noop();
        let mut cx = std::task::Context::from_waker(waker);
        loop {
            match future.as_mut().poll(&mut cx) {
                Poll::Ready(output) => return output,
                Poll::Pending => continue,
            }
        }
    }

    #[derive(Default)]
    struct Holder {
        username: String,
        #[allow(dead_code)]
        email: String,
    }

    struct TakenUsernames(&'static [&'static str]);

    impl AsyncFieldValidation<Holder> for TakenUsernames {
        type Error = &'static str;

        async fn validate_field(
            &self,
            holder: &Holder,
            field: &FieldPath,
        ) -> Result<(), Self::Error> {
            // One impl serves the whole form: unhandled fields are Ok.
            if field.segments() != [PathSegment::Name("username")] {
                return Ok(());
            }
            if self.0.contains(&holder.username.as_str()) {
                Err("username already taken")
            } else {
                Ok(())
            }
        }
    }

    #[test]
    fn one_impl_serves_the_whole_form() {
        let validator = TakenUsernames(&["ada", "grace"]);
        let holder = Holder {
            username: "ada".to_string(),
            email: String::new(),
        };

        assert_eq!(
            block_on(validator.validate_field(&holder, &FieldPath::new(&["username"]))),
            Err("username already taken")
        );
        // Other fields fall through as Ok — no per-field impls needed.
        assert_eq!(
            block_on(validator.validate_field(&holder, &FieldPath::new(&["email"]))),
            Ok(())
        );

        let free = Holder {
            username: "linus".to_string(),
            email: String::new(),
        };
        assert_eq!(
            block_on(validator.validate_field(&free, &FieldPath::new(&["username"]))),
            Ok(())
        );
    }

    #[test]
    fn the_contract_is_genuinely_async() {
        // The future must be awaited to produce a verdict: it returns
        // Pending at least once before completing. Drivers (debounce,
        // latest-wins cancellation) rely on the future making no progress
        // until polled.
        struct PendOnce;
        impl AsyncFieldValidation<Holder> for PendOnce {
            type Error = &'static str;
            async fn validate_field(
                &self,
                _holder: &Holder,
                _field: &FieldPath,
            ) -> Result<(), Self::Error> {
                let mut polls = 0;
                poll_fn(move |_cx| {
                    polls += 1;
                    if polls < 3 {
                        Poll::Pending
                    } else {
                        Poll::Ready(Err("settled on the third poll"))
                    }
                })
                .await
            }
        }

        let holder = Holder::default();
        let path = FieldPath::new(&["a"]);
        let mut future = std::pin::pin!(PendOnce.validate_field(&holder, &path));
        let waker = std::task::Waker::noop();
        let mut cx = std::task::Context::from_waker(waker);
        assert_eq!(future.as_mut().poll(&mut cx), Poll::Pending);
        assert_eq!(future.as_mut().poll(&mut cx), Poll::Pending);
        assert_eq!(
            future.as_mut().poll(&mut cx),
            Poll::Ready(Err("settled on the third poll"))
        );
    }
}
