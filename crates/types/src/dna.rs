//! dna is a library for working with holochain dna files/entries.
//!
//! It includes utilities for representing dna structures in memory,
//! as well as serializing and deserializing dna, mainly to json format.

// pub mod bridges;
// pub mod capabilities;
// pub mod entry_types;
pub mod error;
// pub mod fn_declarations;
// pub mod traits;
pub mod wasm;
pub mod zome;
use crate::prelude::*;
pub use holo_hash::*;
use std::hash::{Hash, Hasher};

/// Represents the top-level holochain dna object.
#[derive(Serialize, Deserialize, Clone, Debug, Eq, SerializedBytes)]
pub struct Dna {}

impl Dna {
    /// Gets DnaHash from Dna
    pub fn dna_hash(&self) -> DnaHash {
        let sb: SerializedBytes = self.try_into().expect("TODO: can this fail?");
        DnaHash::with_data_sync(&sb.bytes())
    }
}

impl Hash for Dna {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let s: Vec<u8> =
            UnsafeBytes::from(SerializedBytes::try_from(self).expect("TODO: can this fail?"))
                .into();
        s.hash(state);
    }
}

impl PartialEq for Dna {
    fn eq(&self, other: &Dna) -> bool {
        // need to guarantee that PartialEq and Hash always agree
        let (this, that) = (
            SerializedBytes::try_from(self),
            SerializedBytes::try_from(other),
        );
        this.is_ok() && that.is_ok() && this == that
    }
}
