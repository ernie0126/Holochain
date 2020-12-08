//! Signals which can be emitted from within Holochain, out across an interface.
//! There are two main kinds of Signal: system-defined, and app-defined:
//! - App-defined signals are produced via the `emit_signal` host function.
//! - System-defined signals are produced in various places in the system

use holochain_serialized_bytes::prelude::*;
use holochain_types::cell::CellId;
use holochain_types::impl_from;
use holochain_zome_types::signal::AppSignal;

/// A Signal is some information emitted from within Holochain out through
/// an Interface
#[derive(Clone, Debug, Serialize, Deserialize, SerializedBytes, PartialEq, Eq)]
pub enum Signal {
    /// Signal from a Cell, generated by `emit_signal`
    App(CellId, AppSignal),
    /// System-defined signals
    System(SystemSignal),
}

/// A Signal which originates from within the Holochain system, as opposed to
/// from within a Cell
///
/// TODO, decide what these will be. For instance, maybe there is a
/// DataAvailable signal for doing async network requests
#[derive(Clone, Debug, Serialize, Deserialize, SerializedBytes, PartialEq, Eq)]
pub enum SystemSignal {
    /// Since we have no real system signals, we use a test signal for testing
    /// TODO: replace instances of this with something real
    Test(String),
}

pub fn test_signal(s: &str) -> Signal {
    SystemSignal::Test(s.to_string()).into()
}

impl_from! {
    SystemSignal => Signal, |s| { Self::System(s) },
}
