//! Contract runtime and gas metering

use crate::{ContractError, ContractResult};
use std::collections::HashMap;

/// Gas costs for different operations
#[derive(Debug, Clone)]
pub struct GasCosts {
    pub read_storage_byte: u64,
    pub write_storage_byte: u64,
    pub delete_storage: u64,
    pub compute_base: u64,
    pub compute_per_byte: u64,
    pub contract_call: u64,
}

impl Default for GasCosts {
    fn default() -> Self {
        Self {
            read_storage_byte: 3,
            write_storage_byte: 10,
            delete_storage: 5,
            compute_base: 100,
            compute_per_byte: 2,
            contract_call: 1000,
        }
    }
}

/// Gas meter for tracking resource consumption
#[derive(Debug, Clone)]
pub struct GasMeter {
    limit: u64,
    consumed: u64,
    costs: GasCosts,
}

impl GasMeter {
    pub fn new(limit: u64) -> Self {
        Self {
            limit,
            consumed: 0,
            costs: GasCosts::default(),
        }
    }

    pub fn with_costs(limit: u64, costs: GasCosts) -> Self {
        Self {
            limit,
            consumed: 0,
            costs,
        }
    }

    /// Consume gas, returning error if limit exceeded
    pub fn consume(&mut self, amount: u64) -> ContractResult<()> {
        let new_total = self.consumed.checked_add(amount)
            .ok_or(ContractError::OutOfGas {
                required: amount,
                available: self.remaining(),
            })?;

        if new_total > self.limit {
            return Err(ContractError::OutOfGas {
                required: amount,
                available: self.remaining(),
            });
        }

        self.consumed = new_total;
        Ok(())
    }

    /// Get remaining gas
    pub fn remaining(&self) -> u64 {
        self.limit.saturating_sub(self.consumed)
    }

    /// Get consumed gas
    pub fn consumed(&self) -> u64 {
        self.consumed
    }

    /// Calculate gas for storage read
    pub fn gas_for_read(&self, bytes: usize) -> u64 {
        self.costs.read_storage_byte * (bytes as u64)
    }

    /// Calculate gas for storage write
    pub fn gas_for_write(&self, bytes: usize) -> u64 {
        self.costs.write_storage_byte * (bytes as u64)
    }

    /// Calculate gas for computation
    pub fn gas_for_compute(&self, bytes: usize) -> u64 {
        self.costs.compute_base + self.costs.compute_per_byte * (bytes as u64)
    }
}

/// Contract runtime that manages contract execution
pub struct ContractRuntime {
    /// Registry of deployed contracts
    contracts: HashMap<String, DeployedContract>,
    
    /// Gas costs configuration
    gas_costs: GasCosts,
}

/// A deployed contract instance
#[derive(Debug, Clone)]
pub struct DeployedContract {
    /// Contract address
    pub address: String,
    
    /// Contract code hash (for verification)
    pub code_hash: String,
    
    /// Contract deployer
    pub creator: String,
    
    /// Deployment timestamp
    pub created_at: u64,
    
    /// Contract version (for migrations)
    pub version: u32,
}

impl ContractRuntime {
    pub fn new() -> Self {
        Self {
            contracts: HashMap::new(),
            gas_costs: GasCosts::default(),
        }
    }

    pub fn with_gas_costs(gas_costs: GasCosts) -> Self {
        Self {
            contracts: HashMap::new(),
            gas_costs,
        }
    }

    /// Register a deployed contract
    pub fn register_contract(
        &mut self,
        address: String,
        code_hash: String,
        creator: String,
    ) -> ContractResult<()> {
        if self.contracts.contains_key(&address) {
            return Err(ContractError::AlreadyExists(address));
        }

        let contract = DeployedContract {
            address: address.clone(),
            code_hash,
            creator,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            version: 1,
        };

        self.contracts.insert(address, contract);
        Ok(())
    }

    /// Get contract info
    pub fn get_contract(&self, address: &str) -> ContractResult<&DeployedContract> {
        self.contracts
            .get(address)
            .ok_or_else(|| ContractError::NotFound(address.to_string()))
    }

    /// Check if contract exists
    pub fn has_contract(&self, address: &str) -> bool {
        self.contracts.contains_key(address)
    }

    /// List all contracts
    pub fn list_contracts(&self) -> Vec<&DeployedContract> {
        self.contracts.values().collect()
    }

    /// Generate a unique contract address
    pub fn generate_address(&self, deployer: &str, nonce: u64) -> String {
        use sha2::{Digest, Sha256};
        
        let mut hasher = Sha256::new();
        hasher.update(deployer.as_bytes());
        hasher.update(nonce.to_le_bytes());
        
        let hash = hasher.finalize();
        format!("contract_{}", hex::encode(&hash[..20]))
    }
}

impl Default for ContractRuntime {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gas_meter_basic() {
        let mut meter = GasMeter::new(1000);
        assert_eq!(meter.remaining(), 1000);

        meter.consume(100).unwrap();
        assert_eq!(meter.consumed(), 100);
        assert_eq!(meter.remaining(), 900);
    }

    #[test]
    fn test_gas_meter_overflow() {
        let mut meter = GasMeter::new(100);
        let result = meter.consume(150);
        assert!(result.is_err());
    }

    #[test]
    fn test_contract_registration() {
        let mut runtime = ContractRuntime::new();
        let addr = runtime.generate_address("deployer", 0);
        
        runtime
            .register_contract(addr.clone(), "hash123".to_string(), "deployer".to_string())
            .unwrap();

        assert!(runtime.has_contract(&addr));
        let contract = runtime.get_contract(&addr).unwrap();
        assert_eq!(contract.version, 1);
    }
}
