//! Contract manager integration for manifold-node

use manifold_contracts::{
    governance::{GovernanceToken, ManifoldDAO, DAOConfig, TokenConfig},
    storage::MemoryStorage,
    Contract, ContractContext, ContractResult, ContractRuntime, ExecutionContext,
};
use libp2p::PeerId;
use std::collections::HashMap;
use tracing::{info, warn};

/// Contract manager for the node
pub struct ContractManager {
    runtime: ContractRuntime,
    deployed_contracts: HashMap<String, DeployedContractInfo>,
    current_block: u64,
}

#[derive(Debug, Clone)]
pub struct DeployedContractInfo {
    pub address: String,
    pub contract_type: ContractType,
    pub deployer: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContractType {
    GovernanceToken,
    DAO,
}

impl ContractManager {
    pub fn new() -> Self {
        Self {
            runtime: ContractRuntime::new(),
            deployed_contracts: HashMap::new(),
            current_block: 0,
        }
    }

    /// Deploy the governance token contract
    pub fn deploy_governance_token(
        &mut self,
        deployer: PeerId,
        config: TokenConfig,
    ) -> ContractResult<String> {
        let address = self.runtime.generate_address(&deployer.to_string(), 0);
        
        info!("🪙 Deploying Governance Token at {}", address);

        // Create execution context
        let info = ExecutionContext::new(
            address.clone(),
            deployer.clone(),
            self.current_block,
            format!("deploy_{}", address),
        );
        let storage = Box::new(MemoryStorage::new());
        let mut ctx = ContractContext::new(info, storage, 10_000_000);

        // Instantiate contract
        GovernanceToken::instantiate(&mut ctx, config)?;

        // Register in runtime
        self.runtime.register_contract(
            address.clone(),
            "governance_token".to_string(),
            deployer.to_string(),
        )?;

        // Track deployment
        self.deployed_contracts.insert(
            address.clone(),
            DeployedContractInfo {
                address: address.clone(),
                contract_type: ContractType::GovernanceToken,
                deployer: deployer.to_string(),
            },
        );

        Ok(address)
    }

    /// Deploy the DAO contract
    pub fn deploy_dao(
        &mut self,
        deployer: PeerId,
        config: DAOConfig,
    ) -> ContractResult<String> {
        let address = self.runtime.generate_address(&deployer.to_string(), 1);
        
        info!("🏛️ Deploying ManifoldDAO at {}", address);

        let info = ExecutionContext::new(
            address.clone(),
            deployer.clone(),
            self.current_block,
            format!("deploy_{}", address),
        );
        let storage = Box::new(MemoryStorage::new());
        let mut ctx = ContractContext::new(info, storage, 10_000_000);

        ManifoldDAO::instantiate(&mut ctx, config)?;

        self.runtime.register_contract(
            address.clone(),
            "manifold_dao".to_string(),
            deployer.to_string(),
        )?;

        self.deployed_contracts.insert(
            address.clone(),
            DeployedContractInfo {
                address: address.clone(),
                contract_type: ContractType::DAO,
                deployer: deployer.to_string(),
            },
        );

        Ok(address)
    }

    /// Update current block height
    pub fn update_block_height(&mut self, block: u64) {
        self.current_block = block;
    }

    /// Get deployed contract info
    pub fn get_contract(&self, address: &str) -> Option<&DeployedContractInfo> {
        self.deployed_contracts.get(address)
    }

    /// List all deployed contracts
    pub fn list_contracts(&self) -> Vec<&DeployedContractInfo> {
        self.deployed_contracts.values().collect()
    }

    /// Check if contract exists
    pub fn has_contract(&self, address: &str) -> bool {
        self.deployed_contracts.contains_key(address)
    }
}

impl Default for ContractManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deploy_governance_token() {
        let mut manager = ContractManager::new();
        let deployer = PeerId::random();
        
        let address = manager
            .deploy_governance_token(deployer, TokenConfig::default())
            .unwrap();

        assert!(manager.has_contract(&address));
        
        let info = manager.get_contract(&address).unwrap();
        assert_eq!(info.contract_type, ContractType::GovernanceToken);
    }

    #[test]
    fn test_deploy_dao() {
        let mut manager = ContractManager::new();
        let deployer = PeerId::random();
        
        let address = manager
            .deploy_dao(deployer, DAOConfig::default())
            .unwrap();

        assert!(manager.has_contract(&address));
        
        let info = manager.get_contract(&address).unwrap();
        assert_eq!(info.contract_type, ContractType::DAO);
    }
}
