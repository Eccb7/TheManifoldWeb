//! Contract execution context

use crate::{ContractStorage, GasMeter};
use libp2p_identity::PeerId;
use std::time::{SystemTime, UNIX_EPOCH};

/// Information about the current contract execution
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    /// Contract address being executed
    pub contract_address: String,
    
    /// Caller/sender of the message
    pub sender: PeerId,
    
    /// Current block/tick number
    pub block_height: u64,
    
    /// Current block timestamp (Unix epoch milliseconds)
    pub block_time: u64,
    
    /// Transaction hash
    pub tx_hash: String,
}

impl ExecutionContext {
    pub fn new(contract_address: String, sender: PeerId, block_height: u64, tx_hash: String) -> Self {
        Self {
            contract_address,
            sender,
            block_height,
            block_time: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            tx_hash,
        }
    }
}

/// Full contract execution context with storage and gas metering
pub struct ContractContext {
    /// Execution metadata
    pub info: ExecutionContext,
    
    /// Storage interface
    pub storage: Box<dyn ContractStorage>,
    
    /// Gas meter for resource tracking
    pub gas_meter: GasMeter,
}

impl ContractContext {
    pub fn new(
        info: ExecutionContext,
        storage: Box<dyn ContractStorage>,
        gas_limit: u64,
    ) -> Self {
        Self {
            info,
            storage,
            gas_meter: GasMeter::new(gas_limit),
        }
    }

    /// Consume gas for an operation
    pub fn consume_gas(&mut self, amount: u64) -> crate::ContractResult<()> {
        self.gas_meter.consume(amount)
    }

    /// Get remaining gas
    pub fn remaining_gas(&self) -> u64 {
        self.gas_meter.remaining()
    }
}
