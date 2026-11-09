//! Platform-owned raw_rand transport and one in-flight source capability.
//! No seed pool, timer, fresh-request retry or consumer admission policy.

use crate::Error;
use std::{cell::RefCell, future::Future, rc::Rc};

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
#[expect(
    clippy::future_not_send,
    reason = "IC source custody is local to one executor thread"
)]
pub async fn fresh_seed() -> Result<[u8; 32], Error> {
    require_replicated()?;
    request_with(SOURCE.with(Rc::clone), transport()).await
}

#[expect(
    clippy::future_not_send,
    reason = "IC source custody is local to one executor thread"
)]
async fn request_with(
    source: Rc<RefCell<Source>>,
    transport: impl Future<Output = Result<Vec<u8>, Error>>,
) -> Result<[u8; 32], Error> {
    let request = Request::claim(source)?;
    let reply = transport.await;
    if !request.is_current() {
        return Err(Error::Stale);
    }
    ic_platform::raw_rand::decode(&reply?).map_err(|_| Error::MalformedReply)
}

#[cfg(target_arch = "wasm32")]
async fn transport() -> Result<Vec<u8>, Error> {
    use ic_platform::raw_rand;
    use ic_platform_runtime::{Principal, cdk::call::Call};
    Call::bounded_wait(Principal::management_canister(), raw_rand::METHOD)
        .with_raw_args(raw_rand::encode())
        .await
        .map(ic_platform_runtime::call::Response::into_bytes)
        .map_err(|_| Error::Transport)
}

#[cfg(not(target_arch = "wasm32"))]
fn transport() -> std::future::Ready<Result<Vec<u8>, Error>> {
    std::future::ready(Err(Error::NonReplicated))
}

#[cfg(target_arch = "wasm32")]
pub fn require_replicated() -> Result<(), Error> {
    #[cfg(target_arch = "wasm32")]
    if ic_platform_runtime::ic0::in_replicated_execution() != 0 {
        return Ok(());
    }
    Err(Error::NonReplicated)
}

#[cfg(not(target_arch = "wasm32"))]
pub const fn require_replicated() -> Result<(), Error> {
    Err(Error::NonReplicated)
}

pub fn invalidate(suspended: bool) {
    SOURCE.with(|source| source.borrow_mut().invalidate(suspended));
}

#[cfg(test)]
mod tests;
