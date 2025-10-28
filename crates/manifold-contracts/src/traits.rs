//! Core contract trait definitions

use crate::{ContractContext, ContractError, ContractResult};
use serde::{de::DeserializeOwned, Serialize};

/// Main contract trait that all contracts must implement
pub trait Contract: Sized {
    /// Contract initialization data type
    type InstantiateMsg: Serialize + DeserializeOwned;
    
    /// Execution message type
    type ExecuteMsg: Serialize + DeserializeOwned;
    
    /// Query message type
    type QueryMsg: Serialize + DeserializeOwned;
    
    /// Query response type
    type QueryResponse: Serialize + DeserializeOwned;
    
    /// Migration message type (for upgrades)
    type MigrateMsg: Serialize + DeserializeOwned;

    /// Deploy/instantiate a new contract instance
    fn instantiate(
        ctx: &mut ContractContext,
        msg: Self::InstantiateMsg,
    ) -> ContractResult<()>;

    /// Execute a state-changing operation
    fn execute(
        ctx: &mut ContractContext,
        msg: Self::ExecuteMsg,
    ) -> ContractResult<Vec<u8>>;

    /// Query contract state (read-only)
    fn query(
        ctx: &ContractContext,
        msg: Self::QueryMsg,
    ) -> ContractResult<Self::QueryResponse>;

    /// Migrate contract to new version
    fn migrate(
        ctx: &mut ContractContext,
        _msg: Self::MigrateMsg,
    ) -> ContractResult<()> {
        Err(ContractError::Custom(
            "Migration not supported".to_string(),
        ))
    }
}

/// Simplified deployment trait for contracts
pub trait ContractDeployment {
    /// Deploy a contract with given init data
    fn deploy(ctx: &mut ContractContext, init_data: &[u8]) -> ContractResult<String>;
}

/// Contract execution interface
pub trait ContractExecution {
    /// Execute a contract call
    fn execute(
        ctx: &mut ContractContext,
        contract_addr: &str,
        msg: &[u8],
    ) -> ContractResult<Vec<u8>>;
}

/// Contract query interface (read-only)
pub trait ContractQuery {
    /// Query contract state
    fn query(
        ctx: &ContractContext,
        contract_addr: &str,
        msg: &[u8],
    ) -> ContractResult<Vec<u8>>;
}
