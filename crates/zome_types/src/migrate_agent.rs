use crate::zome::ZomeName;
use crate::zome_io::GuestOutput;
use holochain_serialized_bytes::prelude::*;

#[derive(Clone, Serialize, Deserialize, SerializedBytes)]
pub enum MigrateAgent {
    Open,
    Close,
}

#[derive(PartialEq, Serialize, Deserialize, SerializedBytes)]
pub enum MigrateAgentCallbackResult {
    Pass,
    Fail(ZomeName, String),
}

impl From<GuestOutput> for MigrateAgentCallbackResult {
    fn from(guest_output: GuestOutput) -> Self {
        match guest_output.into_inner().try_into() {
            Ok(v) => v,
            Err(e) => Self::Fail(ZomeName::unknown(), format!("{:?}", e)),
        }
    }
}
