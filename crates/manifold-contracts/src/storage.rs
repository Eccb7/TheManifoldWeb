//! Contract storage abstraction

use crate::{ContractError, ContractResult};
use std::collections::HashMap;

/// Storage key type
pub type StorageKey = Vec<u8>;

/// Storage value type
pub type StorageValue = Vec<u8>;

/// Contract storage interface
pub trait ContractStorage: Send + Sync {
    /// Get a value from storage
    fn get(&self, key: &StorageKey) -> ContractResult<Option<StorageValue>>;

    /// Set a value in storage
    fn set(&mut self, key: StorageKey, value: StorageValue) -> ContractResult<()>;

    /// Remove a value from storage
    fn remove(&mut self, key: &StorageKey) -> ContractResult<()>;

    /// Check if a key exists
    fn has(&self, key: &StorageKey) -> bool;

    /// Get all keys with a given prefix
    fn keys_with_prefix(&self, prefix: &[u8]) -> ContractResult<Vec<StorageKey>>;
}

/// In-memory storage implementation for testing and fast execution
#[derive(Debug, Clone, Default)]
pub struct MemoryStorage {
    data: HashMap<StorageKey, StorageValue>,
}

impl MemoryStorage {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }
}

impl ContractStorage for MemoryStorage {
    fn get(&self, key: &StorageKey) -> ContractResult<Option<StorageValue>> {
        Ok(self.data.get(key).cloned())
    }

    fn set(&mut self, key: StorageKey, value: StorageValue) -> ContractResult<()> {
        self.data.insert(key, value);
        Ok(())
    }

    fn remove(&mut self, key: &StorageKey) -> ContractResult<()> {
        self.data.remove(key);
        Ok(())
    }

    fn has(&self, key: &StorageKey) -> bool {
        self.data.contains_key(key)
    }

    fn keys_with_prefix(&self, prefix: &[u8]) -> ContractResult<Vec<StorageKey>> {
        Ok(self
            .data
            .keys()
            .filter(|k| k.starts_with(prefix))
            .cloned()
            .collect())
    }
}

/// Persistent storage backed by a key-value store (e.g., RocksDB, sled)
/// TODO: Implement actual persistent backend
#[derive(Debug, Clone)]
pub struct PersistentStorage {
    memory: MemoryStorage, // Temporary implementation
}

impl PersistentStorage {
    pub fn new() -> Self {
        Self {
            memory: MemoryStorage::new(),
        }
    }
}

impl Default for PersistentStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl ContractStorage for PersistentStorage {
    fn get(&self, key: &StorageKey) -> ContractResult<Option<StorageValue>> {
        self.memory.get(key)
    }

    fn set(&mut self, key: StorageKey, value: StorageValue) -> ContractResult<()> {
        self.memory.set(key, value)
    }

    fn remove(&mut self, key: &StorageKey) -> ContractResult<()> {
        self.memory.remove(key)
    }

    fn has(&self, key: &StorageKey) -> bool {
        self.memory.has(key)
    }

    fn keys_with_prefix(&self, prefix: &[u8]) -> ContractResult<Vec<StorageKey>> {
        self.memory.keys_with_prefix(prefix)
    }
}

/// Helper functions for storage operations
pub mod helpers {
    use super::*;
    use serde::{Deserialize, Serialize};

    /// Serialize and store a value
    pub fn set_json<T: Serialize>(
        storage: &mut dyn ContractStorage,
        key: &[u8],
        value: &T,
    ) -> ContractResult<()> {
        let serialized = serde_json::to_vec(value)
            .map_err(|e| ContractError::SerializationError(e.to_string()))?;
        storage.set(key.to_vec(), serialized)
    }

    /// Load and deserialize a value
    pub fn get_json<T: for<'de> Deserialize<'de>>(
        storage: &dyn ContractStorage,
        key: &[u8],
    ) -> ContractResult<Option<T>> {
        match storage.get(&key.to_vec())? {
            Some(bytes) => {
                let value = serde_json::from_slice(&bytes)
                    .map_err(|e| ContractError::SerializationError(e.to_string()))?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }
}
