//! Platform-owned raw_rand transport and one in-flight source capability.
//! No seed pool, timer, fresh-request retry or consumer admission policy.

#[cfg(test)]
mod tests;

use crate::Error;
#[cfg(feature = "metrics")]
use crate::metrics;
use ic_platform_runtime::raw_rand;
use std::{cell::RefCell, future::Future, rc::Rc};

///
/// Source
///
/// Lifecycle generation and exclusive custody for one outstanding acquisition.
///

#[derive(Default)]
struct Source {
    generation: u64,
    pending: bool,
    suspended: bool,
}

impl Source {
    const fn invalidate(&mut self, suspended: bool) {
        self.generation = self
            .generation
            .checked_add(1)
            .expect("entropy generation exhausted");
        self.pending = false;
        self.suspended = suspended;
    }
}

thread_local! {
    static SOURCE: Rc<RefCell<Source>> = Rc::new(RefCell::new(Source::default()));
}

///
/// Request
///
/// A source claim whose drop releases custody only for its own generation.
///

struct Request {
    source: Rc<RefCell<Source>>,
    generation: u64,
}

impl Request {
    fn claim(source: Rc<RefCell<Source>>) -> Result<Self, Error> {
        let generation = {
            let mut state = source.borrow_mut();
            if state.suspended {
                return Err(Error::Disabled);
            }

            if state.pending {
                return Err(Error::Busy);
            }
            state.pending = true;

            state.generation
        };

        Ok(Self { source, generation })
    }

    fn is_current(&self) -> bool {
        let state = self.source.borrow();

        state.generation == self.generation && !state.suspended
    }
}

impl Drop for Request {
    fn drop(&mut self) {
        let mut state = self.source.borrow_mut();
        if state.generation == self.generation {
            state.pending = false;
        }
    }
}

/// Make one new bounded-wait, zero-attached-cycle management request.
///
/// Never consumes buffered or seeded bytes. Call after the relevant commitment.
/// Concurrent requests refuse; failures return without automatic retries.
///
/// # Errors
/// Refuses non-replicated execution, suspension or an outstanding request. Returns
/// a transport or framing error on failure, or [`Error::Stale`] after reconstruction.
///
/// # Panics
/// With `logging`, propagates a panic from the configured host sink.
pub async fn fresh_seed() -> Result<[u8; 32], Error> {
    seed(
        #[cfg(feature = "metrics")]
        metrics::Kind::Fresh,
    )
    .await
}

pub(super) async fn seed(
    #[cfg(feature = "metrics")] kind: metrics::Kind,
) -> Result<[u8; 32], Error> {
    require_replicated()?;
    #[cfg(feature = "metrics")]
    let observation = metrics::Acquisition::start(kind);
    let result = request_with(SOURCE.with(Rc::clone), raw_rand::request(300)).await;
    #[cfg(feature = "metrics")]
    observation.finish(result.as_ref().map(|_| ()).map_err(|error| *error));
    #[cfg(feature = "logging")]
    if let Err(error) = result {
        crate::logging::source_error(error);
    }

    result
}

async fn request_with(
    source: Rc<RefCell<Source>>,
    operation: impl Future<Output = Result<[u8; 32], raw_rand::Error>>,
) -> Result<[u8; 32], Error> {
    let request = Request::claim(source)?;
    let result = operation.await;
    if !request.is_current() {
        return Err(Error::Stale);
    }

    result.map_err(|error| match error {
        raw_rand::Error::Frame(_) => Error::MalformedReply,
        raw_rand::Error::Submission(_)
        | raw_rand::Error::Rejected(_)
        | raw_rand::Error::OutcomeUnknown(_) => Error::Transport,
    })
}

pub(super) fn require_replicated() -> Result<(), Error> {
    if ic_platform_runtime::raw::in_replicated_execution() {
        return Ok(());
    }

    Err(Error::NonReplicated)
}

pub(super) fn invalidate(suspended: bool) {
    SOURCE.with(|source| source.borrow_mut().invalidate(suspended));
}
