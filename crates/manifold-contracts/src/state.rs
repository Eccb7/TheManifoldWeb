//! State management for contracts

use crate::{ContractError, ContractResult, ContractStorage, StorageKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// State transition record for audit and rollback
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateTransition {
    /// Transition ID (hash of changes)
    pub id: String,
    
    /// Contract address
    pub contract: String,
    
    /// Transaction hash that caused this transition
    pub tx_hash: String,
    
    /// Timestamp
    pub timestamp: u64,
    
    /// State changes (key -> (old_value, new_value))
    pub changes: Vec<StateChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateChange {
    pub key: Vec<u8>,
    pub old_value: Option<Vec<u8>>,
    pub new_value: Option<Vec<u8>>,
}

impl StateTransition {
    pub fn new(contract: String, tx_hash: String) -> Self {
        Self {
            id: String::new(), // Will be computed after adding changes
            contract,
            tx_hash,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            changes: Vec::new(),
        }
    }

    pub fn add_change(&mut self, key: Vec<u8>, old_value: Option<Vec<u8>>, new_value: Option<Vec<u8>>) {
        self.changes.push(StateChange {
            key,
            old_value,
            new_value,
        });
    }

    /// Compute and set the transition ID based on changes
    pub fn finalize(&mut self) {
        let mut hasher = Sha256::new();
        hasher.update(self.contract.as_bytes());
        hasher.update(self.tx_hash.as_bytes());
        
        for change in &self.changes {
            hasher.update(&change.key);
            if let Some(ref val) = change.new_value {
                hasher.update(val);
            }
        }
        
        self.id = hex::encode(hasher.finalize());
    }
}

/// State manager that tracks changes and supports rollback
pub struct StateManager {
    storage: Box<dyn ContractStorage>,
    transitions: Vec<StateTransition>,
    max_history: usize,
}

impl StateManager {
    pub fn new(storage: Box<dyn ContractStorage>) -> Self {
        Self {
            storage,
            transitions: Vec::new(),
            max_history: 1000, // Keep last 1000 transitions
        }
    }

    pub fn with_history_limit(storage: Box<dyn ContractStorage>, max_history: usize) -> Self {
        Self {
            storage,
            transitions: Vec::new(),
            max_history,
        }
    }

    /// Begin a new state transition
    pub fn begin_transition(&mut self, contract: String, tx_hash: String) -> StateTransition {
        StateTransition::new(contract, tx_hash)
    }

    /// Apply a state transition
    pub fn apply_transition(&mut self, mut transition: StateTransition) -> ContractResult<()> {
        // Apply all changes
        for change in &transition.changes {
            match &change.new_value {
                Some(value) => {
                    self.storage.set(change.key.clone(), value.clone())?;
                }
                None => {
                    self.storage.remove(&change.key)?;
                }
            }
        }

        // Finalize and store transition
        transition.finalize();
        self.transitions.push(transition);

        // Trim history if needed
        if self.transitions.len() > self.max_history {
            self.transitions.remove(0);
        }

        Ok(())
    }

    /// Get storage reference
    pub fn storage(&self) -> &dyn ContractStorage {
        &*self.storage
    }

    /// Get mutable storage reference
    pub fn storage_mut(&mut self) -> &mut dyn ContractStorage {
        &mut *self.storage
    }

    /// Get transition history
    pub fn history(&self) -> &[StateTransition] {
        &self.transitions
    }

    /// Get specific transition by ID
    pub fn get_transition(&self, id: &str) -> Option<&StateTransition> {
        self.transitions.iter().find(|t| t.id == id)
    }

    /// Rollback to a specific transition (experimental)
    pub fn rollback_to(&mut self, transition_id: &str) -> ContractResult<()> {
        // Find the transition
        let pos = self
            .transitions
            .iter()
            .position(|t| t.id == transition_id)
            .ok_or_else(|| ContractError::InvalidState("Transition not found".to_string()))?;

        // Rollback all transitions after this one in reverse order
        for transition in self.transitions.iter().skip(pos + 1).rev() {
            for change in transition.changes.iter().rev() {
                match &change.old_value {
                    Some(value) => {
                        self.storage.set(change.key.clone(), value.clone())?;
                    }
                    None => {
                        self.storage.remove(&change.key)?;
                    }
                }
            }
        }

        // Truncate history
        self.transitions.truncate(pos + 1);

        Ok(())
    }
}

/// Snapshot of contract state at a specific point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateSnapshot {
    pub contract: String,
    pub timestamp: u64,
    pub block_height: u64,
    pub state_root: String, // Merkle root of state
    pub transition_count: usize,
}

impl StateSnapshot {
    pub fn from_manager(
        manager: &StateManager,
        contract: String,
        block_height: u64,
    ) -> Self {
        // TODO: Implement actual Merkle tree state root computation
        let state_root = format!("snapshot_{}", block_height);

        Self {
            contract,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            block_height,
            state_root,
            transition_count: manager.transitions.len(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::MemoryStorage;

    #[test]
    fn test_state_transition() {
        let mut transition = StateTransition::new(
            "contract_123".to_string(),
            "tx_456".to_string(),
        );

        transition.add_change(
            b"key1".to_vec(),
            None,
            Some(b"value1".to_vec()),
        );

        transition.finalize();
        assert!(!transition.id.is_empty());
    }

    #[test]
    fn test_state_manager() {
        let storage = Box::new(MemoryStorage::new());
        let mut manager = StateManager::new(storage);

        let mut transition = manager.begin_transition(
            "contract_123".to_string(),
            "tx_456".to_string(),
        );

        transition.add_change(
            b"key1".to_vec(),
            None,
            Some(b"value1".to_vec()),
        );

        manager.apply_transition(transition).unwrap();

        assert_eq!(manager.history().len(), 1);
    }
}
