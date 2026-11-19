//! Platform-owned raw_rand transport and one in-flight source capability.
//! No seed pool, timer, fresh-request retry or consumer admission policy.

#[cfg(test)]
mod tests;
// Runtime exposes raw IC admission only on Wasm; host tests
// require fresh requests to refuse before claiming source custody.
#[cfg(target_arch = "wasm32")]
mod wasm;

use crate::Error;
use ic_platform_runtime::raw_rand;
use std::{cell::RefCell, future::Future, rc::Rc};
#[cfg(target_arch = "wasm32")]
pub(super) use wasm::require_replicated;

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

#[cfg_attr(
    all(not(target_arch = "wasm32"), not(test)),
    expect(
        dead_code,
        reason = "native callers refuse; custody is exercised by native tests"
    )
)]
struct Request {
    source: Rc<RefCell<Source>>,
    generation: u64,
}

#[cfg_attr(
    all(not(target_arch = "wasm32"), not(test)),
    expect(
        dead_code,
        reason = "native callers refuse; custody is exercised by native tests"
    )
)]
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
pub async fn fresh_seed() -> Result<[u8; 32], Error> {
    // Runtime 0.23.20 exports this IC operation only on Wasm. Native callers
    // refuse directly; custody tests supply controlled Platform outcomes.
    #[cfg(target_arch = "wasm32")]
    {
        require_replicated()?;
        request_with(SOURCE.with(Rc::clone), raw_rand::request(300)).await
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        Err(Error::NonReplicated)
    }
}

#[cfg_attr(
    all(not(target_arch = "wasm32"), not(test)),
    expect(
        dead_code,
        reason = "native callers refuse; custody is exercised by native tests"
    )
)]
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

#[cfg(not(target_arch = "wasm32"))]
pub(super) const fn require_replicated() -> Result<(), Error> {
    Err(Error::NonReplicated)
}

pub(super) fn invalidate(suspended: bool) {
    SOURCE.with(|source| source.borrow_mut().invalidate(suspended));
}
