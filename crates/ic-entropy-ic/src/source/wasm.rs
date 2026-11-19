//! IC execution admission; the parent retains source custody.

use crate::Error;

pub(crate) fn require_replicated() -> Result<(), Error> {
    if ic_platform_runtime::raw::in_replicated_execution() {
        return Ok(());
    }
    Err(Error::NonReplicated)
}
