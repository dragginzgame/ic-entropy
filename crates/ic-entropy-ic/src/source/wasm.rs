//! IC transport and execution admission; source custody stays in the parent.

use crate::Error;
use ic_platform_protocol::raw_rand;
use ic_platform_runtime::call::{Call, Outcome, Wait};
use ic_platform_types::Principal;

pub(super) async fn transport() -> Result<Vec<u8>, Error> {
    let completion = Call::new(
        Principal::management_canister(),
        raw_rand::METHOD,
        Wait::Bounded { seconds: 300 },
    )
    .args(raw_rand::encode(6).expect("fixed raw_rand request"))
    .execute()
    .await
    .map_err(|_| Error::Transport)?;
    match completion.into_parts().0 {
        Outcome::Reply(bytes) => Ok(bytes),
        Outcome::Rejected(_) | Outcome::OutcomeUnknown(_) => Err(Error::Transport),
    }
}

pub(crate) fn require_replicated() -> Result<(), Error> {
    if ic_platform_runtime::raw::in_replicated_execution() {
        return Ok(());
    }
    Err(Error::NonReplicated)
}
