//! Headless submit-lifecycle state and the submit hook contract.
//!
//! Backlog feature #7 ("Submit lifecycle helpers") ships this as *optional
//! runtime glue, not required generated state*: [`SubmitTracker<E>`] is a
//! plain-data state machine the application owns, and the value holder stays
//! plain data ([D2] of the feature-parity plan — `E` is the app's
//! submit-failure type and never appears in generated form state).
//!
//! The derive exposes it per form through the opt-in
//! `#[gpui_form(submit)]` attribute, which emits a
//! `pub type <Name>FormSubmission = SubmitTracker<...>` alias ([D7]).
//!
//! # Driving a submit
//!
//! `SubmitTracker` stores state; it does not run futures. The intended
//! protocol (mirrored by [`SubmitAction`]) is:
//!
//! ```ignore
//! if tracker.begin() {
//!     let result = action.submit(&holder).await; // app drives the future
//!     tracker.finish(result.map(|_: ()| ()));
//! }
//! ```
//!
//! Driving (GPUI `cx.spawn`, tokio) is glue that stays app- or
//! component-authored ([D5]); nothing here needs an executor, which keeps
//! the crate dependency-free.

/// Default submit-failure type for forms without a koruma error type:
/// a boxed [`std::error::Error`].
///
/// The opt-in `#[gpui_form(submit)]` alias uses the holder's koruma error
/// type when the form has one, and this alias otherwise.
pub type BoxError = ::std::boxed::Box<dyn ::std::error::Error>;

/// State machine for one form's submit lifecycle.
///
/// Tracks whether a submit is in flight, how many attempts started, the last
/// failure, and whether the last attempt succeeded — the submit-half of the
/// parity surface (`isSubmitting`/`isSubmitted`/`isSubmitSuccessful`/
/// `submitCount`). A generation/epoch counter guards against stale async
/// completions ([D7]/R12): a `finish` that arrives after [`reset`] (or with
/// no attempt in flight) is ignored, so a slow submit can never write its
/// outcome into a tracker that has already moved on.
///
/// # Trait bounds
///
/// `E` needs no bounds at all for construction and queries; only the stored
/// `last_error` slot requires `E` to be owned.
///
/// # Example
///
/// ```
/// use gpui_form_core::submit::SubmitTracker;
///
/// let mut tracker: SubmitTracker<&'static str> = SubmitTracker::new();
/// assert!(tracker.can_submit());
///
/// assert!(tracker.begin());          // attempt 1 starts
/// assert!(!tracker.can_submit());
/// assert!(!tracker.begin());         // re-entry guard: double-click protection
///
/// tracker.finish(Err("server said no"));
/// assert!(tracker.is_submitted());
/// assert!(!tracker.is_submit_successful());
/// assert_eq!(tracker.last_error(), Some(&"server said no"));
///
/// tracker.begin();
/// tracker.finish(Ok(()));
/// assert!(tracker.is_submit_successful());
/// assert_eq!(tracker.submit_count(), 2);
///
/// tracker.reset();                   // fresh tracker, new epoch
/// tracker.finish(Err("stale"));      // completion from before the reset
/// assert!(!tracker.is_submitted(), "stale completions are ignored");
/// ```
#[derive(Clone, Debug)]
pub struct SubmitTracker<E> {
    submitting: bool,
    count: u32,
    last_error: Option<E>,
    /// Monotonic epoch. Bumped by every `begin` and every `reset` so an
    /// in-flight completion from an earlier epoch is detectable as stale
    /// ([D7]/R12).
    generation: u64,
    /// The generation of the attempt currently in flight, if any. `finish`
    /// is honored only while this matches — the pinned representation of
    /// "this completion belongs to the current tracker epoch".
    active_generation: Option<u64>,
    /// `Some` once at least one attempt reached a terminal state; the flag
    /// records whether the *last* terminal state was success.
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

    /// Whether the last terminal state was success.
    ///
    /// Manual [`set_error`](Self::set_error)/[`clear_error`](Self::clear_error)
    /// do not affect this flag: a server error injected after a successful
    /// save does not rewrite history.
    pub fn is_submit_successful(&self) -> bool {
        self.last_terminal_success == Some(true)
    }

    /// Submit-button gating: `true` unless a submit is currently in flight.
    ///
    /// Callers layer their own policy on top — form validity (the generated
    /// facade's `is_valid()`), dirty state, permission checks.
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

    /// Marks an attempt as started and returns `true`.
    ///
    /// Returns `false` — changing nothing — when a submit is already in
    /// flight: the re-entry data for double-click protection lives here,
    /// while the decision to use it stays with the caller. A successful
    /// `begin` increments [`submit_count`](Self::submit_count) and bumps the
    /// generation, pinning this attempt as the active epoch.
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
    /// `Err(e)` records it.
    ///
    /// Stale completions are ignored: a `finish` with no attempt in flight
    /// — the state left behind by [`reset`](Self::reset) or
    /// [`abandon`](Self::abandon), i.e. a generation mismatch — changes
    /// nothing.
    pub fn finish(&mut self, result: Result<(), E>) {
        let Some(active) = self.active_generation else {
            return; // stale completion: no in-flight attempt from this epoch
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

    /// Manually inject a submit failure — the hook for server-side errors
    /// that no local validation could produce (parity gap: `setError`).
    ///
    /// Does not start, complete, or abort an attempt; pair with
    /// [`clear_error`](Self::clear_error).
    pub fn set_error(&mut self, error: E) {
        self.last_error = Some(error);
    }

    /// Manually clear an injected or recorded submit failure.
    pub fn clear_error(&mut self) {
        self.last_error = None;
    }

    /// Clear every piece of submit state: attempts, in-flight submit, error,
    /// and terminal verdicts. The generation still moves forward, so
    /// completions from before the reset stay stale forever.
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
///
/// Executor-free by design ([D5]): the implementor returns a future and the
/// *caller* drives it — GPUI glue through `cx.spawn`, server-side callers
/// through their own runtime. There is deliberately no `Send` bound (GPUI
/// futures are typically non-Send; tokio users who need to `spawn` pay for
/// a `Send` wrapper themselves — open question OQ 3 of the parity plan).
///
/// `submit` receives the holder by shared reference: submitting never
/// mutates form data. The save protocol on success — typically
/// `sync_baseline()` on the facade — is the application's to run.
// No `Send` bound by design (see OQ 3 above), hence the plain async fn.
#[allow(async_fn_in_trait)]
pub trait SubmitAction<H> {
    /// What a successful submit produces (a saved-record id, a unit, ...).
    type Output;
    /// The submit failure type; becomes the tracker's `E`.
    type Error;

    /// Perform the submit for a form value snapshot.
    async fn submit(&mut self, holder: &H) -> Result<Self::Output, Self::Error>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::future::{Future, poll_fn};
    use std::task::Poll;

    /// Minimal std-only executor: our test futures are deterministic
    /// state machines (pending a fixed number of polls, then ready), so
    /// spin-polling a noop-waker context is sufficient.
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

        // Double-click protection: the second begin is refused and changes
        // nothing — the count does not move.
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

        // Server error injected after a successful save: visible as
        // last_error, but the last terminal state is still the success.
        tracker.set_error("row changed upstream".to_string());
        assert!(tracker.is_submit_successful());
        assert_eq!(
            tracker.last_error().map(String::as_str),
            Some("row changed upstream")
        );

        tracker.clear_error();
        assert!(tracker.last_error().is_none());
        // Clearing an injected error also does not fabricate success.
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

        // Reset mid-flight (navigation, form reload): the epoch moves and
        // every piece of state clears.
        tracker.reset();
        assert!(!tracker.is_submitting());
        assert_eq!(tracker.submit_count(), 0);
        assert!(tracker.last_error().is_none());
        assert!(!tracker.is_submitted());
        assert!(!tracker.is_submit_successful());

        // The in-flight completion arrives late: generation mismatch -> the
        // entire result is ignored, not partially applied.
        tracker.finish(Ok(()));
        assert!(!tracker.is_submitting(), "stale finish must not flip state");
        assert!(!tracker.is_submitted());
        assert!(!tracker.is_submit_successful());
        assert_eq!(tracker.submit_count(), 0);

        // A stale Err is equally ignored.
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
        // The intended protocol: begin -> drive SubmitAction -> finish.
        // The holder is passed by reference and never mutated.
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

        // Submitting never mutated the form data.
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
                // One `Poll::Pending` first, so the future genuinely pends
                // and only completes because the driver awaited it.
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

        // An app-style gate: the submit may begin only once async
        // validation for the field has completed successfully.
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

        // Invalid value: validation completes with an error, the gate never
        // lets the submit begin.
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

        // Valid value: validation passes and the submit starts.
        let valid = Holder {
            username: "ada".to_string(),
        };
        assert!(gated_begin(&mut tracker, &validator, &valid));
        assert!(tracker.is_submitting());
        tracker.finish(Ok(()));
        assert!(tracker.is_submit_successful());
    }
}
