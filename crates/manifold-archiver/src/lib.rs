//! Permanent archival layer for The Manifold Web.
//!
//! Integrates with Arweave for permanent, immutable storage of:
//! - Genesis blocks (initial simulation state)
//! - Consensus checkpoints
//! - Historical simulation snapshots
//!
//! Arweave provides permanent storage with a one-time payment model,
//! ensuring critical simulation data persists indefinitely.

use anyhow::Result;
use arweave_rs::{Arweave, Transaction};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;
use tracing::{info, warn};

/// A genesis block representing the initial state of the simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisBlock {
    /// Unique identifier for this genesis
    pub id: String,
    
    /// Timestamp of creation (Unix epoch seconds)
    pub timestamp: u64,
    
    /// Initial agent genome CIDs
    pub genesis_genomes: Vec<String>,
    
    /// Initial energy allocation per agent
    pub initial_energy: u64,
    
    /// World parameters
    pub world_config: WorldConfig,
    
    /// Creator's signature or identifier
    pub creator: String,
    
    /// SHA-256 hash of the block contents
    pub hash: [u8; 32],
}

/// World configuration parameters for the simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldConfig {
    /// Size of each sector (for distributed simulation)
    pub sector_size: f32,
    
    /// Maximum agents per sector
    pub max_agents_per_sector: u64,
    
    /// Energy decay rate per tick
    pub energy_decay_rate: f64,
    
    /// Mutation rate for genetic algorithm
    pub mutation_rate: f64,
    
    /// Replication energy threshold
    pub replication_threshold: u64,
}

impl GenesisBlock {
    /// Create a new genesis block with the given parameters.
    pub fn new(
        genesis_genomes: Vec<String>,
        initial_energy: u64,
        world_config: WorldConfig,
        creator: String,
    ) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let id = format!("genesis-{}", timestamp);
        
        let mut block = Self {
            id,
            timestamp,
            genesis_genomes,
            initial_energy,
            world_config,
            creator,
            hash: [0u8; 32],
        };
        
        // Calculate and set hash
        block.hash = block.calculate_hash();
        block
    }
    
    /// Calculate the SHA-256 hash of the genesis block.
    fn calculate_hash(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        
        hasher.update(self.id.as_bytes());
        hasher.update(self.timestamp.to_le_bytes());
        hasher.update(self.initial_energy.to_le_bytes());
        hasher.update(self.creator.as_bytes());
        
        for genome_cid in &self.genesis_genomes {
            hasher.update(genome_cid.as_bytes());
        }
        
        // Hash world config
        hasher.update(self.world_config.sector_size.to_le_bytes());
        hasher.update(self.world_config.max_agents_per_sector.to_le_bytes());
        hasher.update(self.world_config.energy_decay_rate.to_le_bytes());
        hasher.update(self.world_config.mutation_rate.to_le_bytes());
        hasher.update(self.world_config.replication_threshold.to_le_bytes());
        
        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        hash
    }
    
    /// Serialize to JSON.
    pub fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }
    
    /// Deserialize from JSON.
    pub fn from_json(json: &str) -> Result<Self> {
        Ok(serde_json::from_str(json)?)
    }
    
    /// Verify the integrity of the genesis block.
    pub fn verify(&self) -> bool {
        let computed_hash = self.calculate_hash();
        computed_hash == self.hash
    }
}

/// Archiver for uploading data to Arweave.
pub struct Archiver {
    arweave: Arweave,
    wallet_path: String,
}

impl Archiver {
    /// Create a new archiver instance.
    ///
    /// # Arguments
    /// * `arweave_node_url` - URL of the Arweave gateway (e.g., "https://arweave.net")
    /// * `wallet_path` - Path to the Arweave wallet JSON file
    pub async fn new(arweave_node_url: &str, wallet_path: &str) -> Result<Self> {
        let arweave = Arweave::new(arweave_node_url.parse()?);
        
        // Verify wallet exists
        if !Path::new(wallet_path).exists() {
            warn!(
                "Arweave wallet not found at {}. You'll need a wallet to upload data.",
                wallet_path
            );
        }
        
        Ok(Self {
            arweave,
            wallet_path: wallet_path.to_string(),
        })
    }
    
    /// Archive a genesis block to Arweave.
    ///
    /// Returns the Arweave transaction ID which can be used to retrieve the data.
    ///
    /// # Example
    /// ```no_run
    /// # use manifold_archiver::{Archiver, GenesisBlock, WorldConfig};
    /// # tokio_test::block_on(async {
    /// let archiver = Archiver::new("https://arweave.net", "wallet.json").await?;
    /// let genesis = GenesisBlock::new(
    ///     vec!["QmExample".to_string()],
    ///     1000,
    ///     WorldConfig {
    ///         sector_size: 100.0,
    ///         max_agents_per_sector: 1000,
    ///         energy_decay_rate: 0.01,
    ///         mutation_rate: 0.01,
    ///         replication_threshold: 500,
    ///     },
    ///     "creator-id".to_string(),
    /// );
    /// let tx_id = archiver.archive_genesis_block(&genesis).await?;
    /// println!("Genesis block archived: https://arweave.net/{}", tx_id);
    /// # Ok::<(), anyhow::Error>(())
    /// # });
    /// ```
    pub async fn archive_genesis_block(&self, genesis: &GenesisBlock) -> Result<String> {
        // Verify genesis block integrity
        if !genesis.verify() {
            anyhow::bail!("Genesis block hash verification failed");
        }
        
        // Serialize to JSON
        let json_data = genesis.to_json()?;
        let data_bytes = json_data.as_bytes();
        
        info!(
            "📦 Preparing to archive genesis block '{}' ({} bytes)",
            genesis.id,
            data_bytes.len()
        );
        
        // TODO: Implement proper Arweave transaction creation and signing
        // This is a placeholder showing the intended workflow:
        //
        // 1. Load wallet from wallet_path
        // 2. Create transaction with data and appropriate tags
        // 3. Sign transaction with wallet
        // 4. Submit to Arweave network
        // 5. Wait for confirmation
        //
        // let wallet = load_wallet(&self.wallet_path)?;
        // let tx = Transaction::new(&self.arweave, data_bytes, &wallet).await?;
        // tx.add_tag("Content-Type", "application/json");
        // tx.add_tag("Application", "TheManifoldWeb");
        // tx.add_tag("Type", "GenesisBlock");
        // tx.add_tag("Genesis-ID", &genesis.id);
        // let tx_id = tx.sign_and_submit(&wallet).await?;
        
        warn!("TODO: Implement Arweave transaction signing and submission");
        warn!("Genesis block prepared but not uploaded (wallet integration needed)");
        
        // For now, return a mock transaction ID
        let mock_tx_id = format!("mock-tx-{}", genesis.id);
        
        info!("✅ Genesis block would be available at: https://arweave.net/{}", mock_tx_id);
        
        Ok(mock_tx_id)
    }
    
    /// Retrieve a genesis block from Arweave by transaction ID.
    pub async fn retrieve_genesis_block(&self, tx_id: &str) -> Result<GenesisBlock> {
        info!("📥 Retrieving genesis block from Arweave: {}", tx_id);
        
        // TODO: Implement Arweave data retrieval
        // let data = self.arweave.get_transaction_data(tx_id).await?;
        // let json_str = String::from_utf8(data)?;
        // let genesis = GenesisBlock::from_json(&json_str)?;
        
        warn!("TODO: Implement Arweave data retrieval");
        anyhow::bail!("Retrieval not yet implemented - use Arweave gateway directly")
    }
    
    /// Archive a simulation checkpoint (state snapshot at a specific tick).
    ///
    /// Returns the Arweave transaction ID.
    pub async fn archive_checkpoint(
        &self,
        tick: u64,
        state_hash: [u8; 32],
        agent_count: usize,
        metadata: serde_json::Value,
    ) -> Result<String> {
        let checkpoint = serde_json::json!({
            "type": "checkpoint",
            "tick": tick,
            "state_hash": hex::encode(state_hash),
            "agent_count": agent_count,
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            "metadata": metadata,
        });
        
        info!("📸 Archiving checkpoint at tick {}", tick);
        
        // TODO: Implement checkpoint archival similar to genesis_block
        warn!("TODO: Implement checkpoint archival");
        
        Ok(format!("mock-checkpoint-{}", tick))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_genesis_block_creation() {
        let genesis = GenesisBlock::new(
            vec!["QmTest1".to_string(), "QmTest2".to_string()],
            1000,
            WorldConfig {
                sector_size: 100.0,
                max_agents_per_sector: 1000,
                energy_decay_rate: 0.01,
                mutation_rate: 0.01,
                replication_threshold: 500,
            },
            "test-creator".to_string(),
        );
        
        assert!(genesis.verify());
        assert_eq!(genesis.genesis_genomes.len(), 2);
        assert_eq!(genesis.initial_energy, 1000);
    }
    
    #[test]
    fn test_genesis_block_serialization() {
        let genesis = GenesisBlock::new(
            vec!["QmTest".to_string()],
            1000,
            WorldConfig {
                sector_size: 100.0,
                max_agents_per_sector: 1000,
                energy_decay_rate: 0.01,
                mutation_rate: 0.01,
                replication_threshold: 500,
            },
            "test-creator".to_string(),
        );
        
        let json = genesis.to_json().unwrap();
        let deserialized = GenesisBlock::from_json(&json).unwrap();
        
        assert_eq!(genesis.id, deserialized.id);
        assert_eq!(genesis.hash, deserialized.hash);
        assert!(deserialized.verify());
    }
    
    #[test]
    fn test_genesis_block_tampering_detection() {
        let mut genesis = GenesisBlock::new(
            vec!["QmTest".to_string()],
            1000,
            WorldConfig {
                sector_size: 100.0,
                max_agents_per_sector: 1000,
                energy_decay_rate: 0.01,
                mutation_rate: 0.01,
                replication_threshold: 500,
            },
            "test-creator".to_string(),
        );
        
        // Tamper with the genesis block
        genesis.initial_energy = 9999;
        
        // Verification should fail
        assert!(!genesis.verify());
    }
}
