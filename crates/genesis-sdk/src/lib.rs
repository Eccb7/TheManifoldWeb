//! # Genesis SDK
//!
//! Software Development Kit for creating and deploying agents to The Manifold
//! Web.
//!
//! Provides utilities for:
//! - Publishing genomes (WASM modules) to IPFS
//! - Spawning agents on network nodes via libp2p
//! - Managing agent lifecycle

pub mod ipfs;
pub mod spawn;

pub use ipfs::*;
pub use spawn::*;
