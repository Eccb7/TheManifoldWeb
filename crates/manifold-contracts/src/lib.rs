//! # Manifold Contracts
//!
//! A custom Rust-native smart contract framework for The Manifold Web.
//!
//! Unlike traditional blockchain smart contracts that rely on external VMs,
//! Manifold Contracts are native Rust code compiled to WASM, providing:
//!
//! - **Type Safety**: Full Rust type system with compile-time guarantees
//! - **Performance**: Native execution speed with optional WASM sandboxing
//! - **Composability**: Direct function calls between contracts
//! - **State Management**: Built-in storage abstraction with versioning
//! - **Gas Metering**: Resource accounting for fair computation
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │                  Contract Interface                      │
//! │  (deploy, execute, query, migrate)                      │
//! └─────────────────────────────────────────────────────────┘
//!                          │
//!         ┌────────────────┼────────────────┐
//!         │                │                │
//!    ┌────▼────┐      ┌────▼────┐     ┌────▼────┐
//!    │ Storage │      │  Gas    │     │ Context │
//!    │ Manager │      │ Meter   │     │  Info   │
//!    └─────────┘      └─────────┘     └─────────┘
//! ```

pub mod context;
pub mod errors;
pub mod governance;
pub mod runtime;
pub mod state;
pub mod storage;
pub mod traits;

// Re-exports for convenience
pub use context::{ContractContext, ExecutionContext};
pub use errors::{ContractError, ContractResult};
pub use runtime::{ContractRuntime, GasMeter};
pub use state::{StateManager, StateTransition};
pub use storage::{ContractStorage, StorageKey, StorageValue};
pub use traits::{Contract, ContractDeployment, ContractExecution, ContractQuery};

/// Standard response type for contract execution
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ContractResponse<T> {
    /// Execution result data
    pub data: T,
    /// Events emitted during execution
    pub events: Vec<ContractEvent>,
    /// Gas consumed
    pub gas_used: u64,
}

/// Events emitted by contracts for external observation
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ContractEvent {
    /// Event type identifier
    pub event_type: String,
    /// Event attributes
    pub attributes: Vec<(String, String)>,
}

impl ContractEvent {
    pub fn new(event_type: impl Into<String>) -> Self {
        Self {
            event_type: event_type.into(),
            attributes: Vec::new(),
        }
    }

    pub fn add_attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.push((key.into(), value.into()));
        self
    }
}
