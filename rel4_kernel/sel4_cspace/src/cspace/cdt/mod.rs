pub mod proof;
pub mod spec;
pub mod state;

#[cfg(verus_keep_ghost)]
pub use state::depth_witness_valid_for;
pub use state::{CdtDepthWitness, CdtState};
