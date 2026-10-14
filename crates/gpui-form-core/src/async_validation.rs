//! Executor-free async per-field validation contract, path-keyed by
//! [`crate::FieldPath`]. Nothing here runs, times, or cancels a future —
//! the driving layer owns it.

use crate::FieldPath;

/// Async per-field validation over a live form holder. Return `Ok(())`
/// for fields the validator does not handle so a single impl can serve
/// a whole form.
#[allow(async_fn_in_trait)]
pub trait AsyncFieldValidation<H> {
    /// The async failure type for this validator.
    type Error;

    /// Validate one field of the live holder; return `Ok(())` for fields
    /// this validator does not handle.
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
