// Error types are self-explanatory
#![allow(missing_docs)]

use super::app_validation_workflow::AppValidationError;
use super::produce_dht_ops_workflow::dht_op_light::error::DhtOpConvertError;
use crate::conductor::api::error::ConductorApiError;
use crate::conductor::CellError;
use crate::core::queue_consumer::QueueTriggerClosedError;
use crate::core::ribosome::error::RibosomeError;
use crate::core::SysValidationError;
use holochain_cascade::error::CascadeError;
use holochain_lmdb::error::DatabaseError;
use holochain_p2p::HolochainP2pError;
use holochain_state::source_chain::SourceChainError;
use holochain_state::workspace::WorkspaceError;
use holochain_types::prelude::*;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WorkflowError {
    #[error(
        "You authored an invalid element! This either means:\n
    - a DNA you're using has some faulty validation logic,\n
    - Holochain has a serious bug,\n
    - or you are using a modified conductor\n
    We have no graceful way to deal with this for now,
    so this app is now in an invalid state and will be disabled.\n

    Rejected DhtOp: {0:?}"
    )]
    AuthoredGenesisValidationRejection(DhtOpLight),

    #[error(transparent)]
    AppValidationError(#[from] AppValidationError),

    #[error("Agent is invalid: {0:?}")]
    AgentInvalid(AgentPubKey),

    #[error("Conductor API error: {0}")]
    ConductorApi(#[from] Box<ConductorApiError>),

    #[error(transparent)]
    CascadeError(#[from] CascadeError),

    #[error("Workspace error: {0}")]
    WorkspaceError(#[from] WorkspaceError),

    #[error("Database error: {0}")]
    DatabaseError(#[from] DatabaseError),

    #[error(transparent)]
    RibosomeError(#[from] RibosomeError),

    #[error("Source chain error: {0}")]
    SourceChainError(#[from] SourceChainError),

    #[error("Capability token missing")]
    CapabilityMissing,

    #[error(transparent)]
    SerializedBytesError(#[from] SerializedBytesError),

    #[error(transparent)]
    DhtOpConvertError(#[from] DhtOpConvertError),

    #[error(transparent)]
    CellError(#[from] CellError),

    #[error(transparent)]
    JoinError(#[from] tokio::task::JoinError),

    #[error(transparent)]
    QueueTriggerClosedError(#[from] QueueTriggerClosedError),

    #[error(transparent)]
    HolochainP2pError(#[from] HolochainP2pError),

    #[error(transparent)]
    HoloHashError(#[from] holo_hash::error::HoloHashError),

    #[error(transparent)]
    DhtOpError(#[from] DhtOpError),

    #[error(transparent)]
    SysValidationError(#[from] SysValidationError),
}

/// Internal type to handle running workflows
pub type WorkflowResult<T> = Result<T, WorkflowError>;
