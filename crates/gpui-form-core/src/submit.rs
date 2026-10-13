//! Headless submit-lifecycle state: a plain-data [`SubmitTracker`] state
//! machine and the executor-free [`SubmitAction`] hook contract, exposed
//! per form through the opt-in `#[gpui_form(submit)]` derive attribute.

/// Default submit-failure type for forms without a koruma error type: a
/// boxed [`std::error::Error`].
pub type BoxError = ::std::boxed::Box<dyn ::std::error::Error>;

/// State machine for one form's submit lifecycle: in-flight flag, attempt
/// count, last failure, and last terminal verdict. A generation counter
/// makes a `finish` arriving after [`SubmitTracker::reset`] a no-op.
#[derive(Clone, Debug)]
pub struct SubmitTracker<E> {
    submitting: bool,
    count: u32,
    last_error: Option<E>,
    generation: u64,
    active_generation: Option<u64>,
    last_terminal_success: Option<bool>,
}

impl<E> SubmitTracker<E> {
    /// Create a tracker with no attempts, no in-flight submit, and no error.
    pub const fn new() -> Self {
        Self {
            submitting: false,
            count: 0,
            last_error: None,
            generation: 0,
            active_generation: None,
            last_terminal_success: None,
        }
    }

    /// Whether a submit attempt is currently in flight.
    pub fn is_submitting(&self) -> bool {
        self.submitting
    }

    /// How many attempts have started (`begin` returned `true`).
    pub fn submit_count(&self) -> u32 {
        self.count
    }

    /// Whether at least one attempt has reached a terminal state
    /// (a `finish` that was honored, not a stale one).
    pub fn is_submitted(&self) -> bool {
        self.last_terminal_success.is_some()
    }

    /// Whether the last terminal state was success; manual
    /// [`set_error`](Self::set_error)/[`clear_error`](Self::clear_error)
    /// do not rewrite this verdict.
    pub fn is_submit_successful(&self) -> bool {
        self.last_terminal_success == Some(true)
    }

    /// Submit-button gating: `true` unless a submit is currently in flight.
    pub fn can_submit(&self) -> bool {
        !self.submitting
    }

    /// The last submit failure, if any.
    pub fn last_error(&self) -> Option<&E> {
        self.last_error.as_ref()
    }

    /// Take the last submit failure, clearing the slot (one-shot banners).
    pub fn take_last_error(&mut self) -> Option<E> {
        self.last_error.take()
    }

    /// Marks an attempt as started; returns `false` and changes nothing when
    /// a submit is already in flight. A successful `begin` increments
    /// [`submit_count`](Self::submit_count).
    pub fn begin(&mut self) -> bool {
        if self.active_generation.is_some() {
            return false;
        }
        self.generation += 1;
        self.active_generation = Some(self.generation);
        self.submitting = true;
        self.count += 1;
        true
    }

    /// Completes the in-flight attempt: `Ok(())` clears `last_error`,
    /// `Err(e)` records it. Stale completions with no in-flight attempt
    /// change nothing.
    pub fn finish(&mut self, result: Result<(), E>) {
        let Some(active) = self.active_generation else {
            return;
        };
        debug_assert_eq!(
            active, self.generation,
            "the active attempt always carries the current generation"
        );
        self.active_generation = None;
        self.submitting = false;
        match result {
            Ok(()) => {
                self.last_error = None;
                self.last_terminal_success = Some(true);
            },
            Err(error) => {
                self.last_error = Some(error);
                self.last_terminal_success = Some(false);
            },
        }
    }

    /// Clears in-flight state with no outcome (an abandoned or cancelled
    /// submit). A late `finish` for the abandoned attempt is ignored.
    pub fn abandon(&mut self) {
        self.active_generation = None;
        self.submitting = false;
    }

    /// Manually inject a submit failure without starting or completing an
    /// attempt; pair with [`clear_error`](Self::clear_error).
    pub fn set_error(&mut self, error: E) {
        self.last_error = Some(error);
    }

    /// Manually clear an injected or recorded submit failure.
    pub fn clear_error(&mut self) {
        self.last_error = None;
    }

    /// Clear every piece of submit state; the generation still moves
    /// forward, so completions from before the reset stay stale forever.
    pub fn reset(&mut self) {
        self.submitting = false;
        self.count = 0;
        self.last_error = None;
        self.active_generation = None;
        self.last_terminal_success = None;
        self.generation += 1;
    }
}

impl<E> Default for SubmitTracker<E> {
    fn default() -> Self {
        Self::new()
    }
}

/// The submit hook contract: perform the submit for a form value snapshot.
/// Executor-free — the implementor returns a future and the caller drives
/// it; there is deliberately no `Send` bound.
#[allow(async_fn_in_trait)]
pub trait SubmitAction<H> {
    /// What a successful submit produces (a saved-record id, a unit, ...).
    type Output;
    /// The submit failure type; becomes the tracker's `E`.
    type Error;

    async fn submit(&mut self, holder: &H) -> Result<Self::Output, Self::Error>;
}

#[cfg(test)]
mod tests {
    use super::*;
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

    #[derive(Clone, Debug, PartialEq, Default)]
    struct Holder {
        username: String,
    }

    struct SaveAction;

    impl SubmitAction<Holder> for SaveAction {
        type Output = u32;
        type Error = &'static str;

        async fn submit(&mut self, holder: &Holder) -> Result<u32, &'static str> {
            if holder.username == "conflict" {
                Err("username already taken")
            } else {
                Ok(holder.username.len() as u32)
            }
        }
    }

    #[test]
    fn new_tracker_is_idle() {
        let tracker: SubmitTracker<String> = SubmitTracker::new();
        let default: SubmitTracker<String> = SubmitTracker::default();
        assert!(!tracker.is_submitting());
        assert!(tracker.can_submit());
        assert_eq!(tracker.submit_count(), 0);
        assert!(!tracker.is_submitted());
        assert!(!tracker.is_submit_successful());
        assert!(tracker.last_error().is_none());
        assert!(!default.is_submitted());
    }

    #[test]
    fn begin_starts_an_attempt_and_guards_reentry() {
        let mut tracker: SubmitTracker<()> = SubmitTracker::new();
        assert!(tracker.begin());
        assert!(tracker.is_submitting());
        assert!(!tracker.can_submit());
        assert_eq!(tracker.submit_count(), 1);

        assert!(!tracker.begin());
        assert_eq!(tracker.submit_count(), 1);
        assert!(tracker.is_submitting());
        assert!(!tracker.is_submitted(), "in-flight is not terminal yet");
    }

    #[test]
    fn finish_err_then_finish_ok_carry_state() {
        let mut tracker: SubmitTracker<&'static str> = SubmitTracker::new();

        tracker.begin();
        tracker.finish(Err("boom"));
        assert!(!tracker.is_submitting());
        assert!(tracker.can_submit());
        assert!(tracker.is_submitted());
        assert!(!tracker.is_submit_successful());
        assert_eq!(tracker.last_error(), Some(&"boom"));

        tracker.begin();
        tracker.finish(Ok(()));
        assert!(tracker.is_submitted());
        assert!(tracker.is_submit_successful());
        assert_eq!(tracker.last_error(), None, "success clears the error");
        assert_eq!(tracker.submit_count(), 2);
    }

    #[test]
    fn take_last_error_is_one_shot() {
        let mut tracker: SubmitTracker<String> = SubmitTracker::new();
        tracker.set_error("late failure".to_string());
        assert_eq!(tracker.take_last_error(), Some("late failure".to_string()));
        assert_eq!(tracker.take_last_error(), None);
        assert!(tracker.last_error().is_none());
    }

    #[test]
    fn manual_error_injection_does_not_rewrite_terminal_verdicts() {
        let mut tracker: SubmitTracker<String> = SubmitTracker::new();
        tracker.begin();
        tracker.finish(Ok(()));
        assert!(tracker.is_submit_successful());

        tracker.set_error("row changed upstream".to_string());
        assert!(tracker.is_submit_successful());
        assert_eq!(
            tracker.last_error().map(String::as_str),
            Some("row changed upstream")
        );

        tracker.clear_error();
        assert!(tracker.last_error().is_none());
        tracker.begin();
        tracker.finish(Err("real failure".to_string()));
        tracker.clear_error();
        assert!(!tracker.is_submit_successful(), "cleared error != success");
        assert!(tracker.is_submitted());
    }

    #[test]
    fn stale_completions_after_reset_are_ignored() {
        let mut tracker: SubmitTracker<&'static str> = SubmitTracker::new();
        tracker.begin();
        tracker.finish(Err("first attempt failed"));
        assert!(tracker.is_submitted());
        assert_eq!(tracker.submit_count(), 1);

        tracker.begin();
        assert!(tracker.is_submitting());

        tracker.reset();
        assert!(!tracker.is_submitting());
        assert_eq!(tracker.submit_count(), 0);
        assert!(tracker.last_error().is_none());
        assert!(!tracker.is_submitted());
        assert!(!tracker.is_submit_successful());

        tracker.finish(Ok(()));
        assert!(!tracker.is_submitting(), "stale finish must not flip state");
        assert!(!tracker.is_submitted());
        assert!(!tracker.is_submit_successful());
        assert_eq!(tracker.submit_count(), 0);

        tracker.finish(Err("also stale"));
        assert!(tracker.last_error().is_none());
    }

    #[test]
    fn late_completions_after_abandon_are_ignored() {
        let mut tracker: SubmitTracker<&'static str> = SubmitTracker::new();
        tracker.begin();
        tracker.abandon();
        assert!(!tracker.is_submitting());
        assert!(tracker.can_submit());
        assert!(
            !tracker.is_submitted(),
            "abandoned attempts have no outcome"
        );
        assert_eq!(tracker.submit_count(), 1, "the attempt did start");

        tracker.finish(Err("late"));
        assert!(!tracker.is_submitted());
        assert!(tracker.last_error().is_none());
    }

    #[test]
    fn reset_allows_a_fresh_lifecycle() {
        let mut tracker: SubmitTracker<&'static str> = SubmitTracker::new();
        tracker.begin();
        tracker.finish(Err("nope"));
        tracker.reset();

        assert!(tracker.begin());
        tracker.finish(Ok(()));
        assert!(tracker.is_submit_successful());
        assert_eq!(tracker.submit_count(), 1, "count restarted at zero");
    }

    #[test]
    fn driving_submit_action_through_the_tracker() {
        let mut holder = Holder::default();
        let mut action = SaveAction;
        let mut tracker: SubmitTracker<&'static str> = SubmitTracker::new();

        tracker.begin();
        let result = block_on(action.submit(&holder));
        tracker.finish(result.map(|_| ()));
        assert!(tracker.is_submit_successful());

        holder.username = "conflict".to_string();
        let snapshot = holder.clone();
        tracker.begin();
        let result = block_on(action.submit(&snapshot));
        tracker.finish(result.map(|_| ()));
        assert!(!tracker.is_submit_successful());
        assert_eq!(tracker.last_error(), Some(&"username already taken"));

        assert_eq!(
            holder,
            Holder {
                username: "conflict".to_string(),
            }
        );
    }

    #[test]
    fn submit_gating_waits_on_pending_async_validation() {
        use crate::async_validation::AsyncFieldValidation;
        use crate::path::FieldPath;
        use std::cell::Cell;

        struct SlowEmailCheck {
            polled: Cell<usize>,
        }

        impl AsyncFieldValidation<Holder> for SlowEmailCheck {
            type Error = &'static str;

            async fn validate_field(
                &self,
                holder: &Holder,
                _field: &FieldPath,
            ) -> Result<(), Self::Error> {
                let observed = &self.polled;
                let mut polls = 0;
                poll_fn(move |_cx| {
                    polls += 1;
                    observed.set(polls);
                    if polls < 2 {
                        Poll::Pending
                    } else if holder.username.contains(' ') {
                        Poll::Ready(Err("username contains whitespace"))
                    } else {
                        Poll::Ready(Ok(()))
                    }
                })
                .await
            }
        }

        fn gated_begin<V>(
            tracker: &mut SubmitTracker<V::Error>,
            validator: &V,
            holder: &Holder,
        ) -> bool
        where
            V: AsyncFieldValidation<Holder, Error = &'static str>,
        {
            let field = FieldPath::new(&["username"]);
            matches!(block_on(validator.validate_field(holder, &field)), Ok(())) && tracker.begin()
        }

        let validator = SlowEmailCheck {
            polled: Cell::new(0),
        };
        let mut tracker: SubmitTracker<&'static str> = SubmitTracker::new();

        let invalid = Holder {
            username: "a b".to_string(),
        };
        assert!(!gated_begin(&mut tracker, &validator, &invalid));
        assert!(!tracker.is_submitting());
        assert_eq!(tracker.submit_count(), 0);
        assert!(
            validator.polled.get() >= 2,
            "the gate awaited the future to completion"
        );

        let valid = Holder {
            username: "ada".to_string(),
        };
        assert!(gated_begin(&mut tracker, &validator, &valid));
        assert!(tracker.is_submitting());
        tracker.finish(Ok(()));
        assert!(tracker.is_submit_successful());
    }
}
