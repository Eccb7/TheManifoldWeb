//! Core data models for The Manifold Web protocol.
//!
//! These structures define agents, genomes, resources, actions, and governance
//! primitives that form the basis of the decentralized agent ecosystem.

use glam::Vec3;
use libp2p_identity::PeerId;
use serde::{Deserialize, Serialize};

/// A content-addressed genome defining an agent's behavior.
///
/// The genome is stored on IPFS and contains executable WASM code that
/// determines how the agent perceives, decides, and acts in the manifold.
///
/// # Fields
/// * `cid` - IPFS Content Identifier pointing to the WASM module
/// * `parameters` - Byte array of evolvable parameters (genetic material)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Genome {
    pub cid: String,
    pub parameters: Vec<u8>,
}

impl Genome {
    /// Create a new genome with the given CID and parameters.
    pub fn new(cid: String, parameters: Vec<u8>) -> Self {
        Self { cid, parameters }
    }

    /// Serialize genome to JSON string.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Deserialize genome from JSON string.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Validate the genome structure.
    pub fn validate(&self) -> Result<(), crate::ProtocolError> {
        if self.cid.is_empty() {
            return Err(crate::ProtocolError::InvalidGenome(
                "CID cannot be empty".to_string(),
            ));
        }
        // TODO: Add proper CID format validation using cid crate
        Ok(())
    }
}

/// An autonomous agent in the manifold.
///
/// Agents are the primary actors in the system, executing their genome's
/// instructions, consuming resources, and potentially creating offspring.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    /// Unique peer identifier for this agent
    #[serde(with = "peer_id_serde")]
    pub id: PeerId,

    /// The agent's genome (behavior definition)
    pub genome: Genome,

    /// Current energy level (consumed by actions, gained from resources)
    pub energy: u64,

    /// 3D position in the manifold space
    pub position: Vec3,

    /// Creation timestamp (Unix epoch milliseconds)
    pub created_at: u64,

    /// Generation number (0 for genesis agents)
    pub generation: u32,
}

impl Agent {
    pub fn new(id: PeerId, genome: Genome, initial_energy: u64, position: Vec3) -> Self {
        Self {
            id,
            genome,
            energy: initial_energy,
            position,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            generation: 0,
        }
    }
}

/// Resources available in the manifold for agents to consume.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Resource {
    /// Energy crystal providing raw energy
    Energy { amount: u64, position: Vec3 },

    /// Information packet containing data
    Information { data: Vec<u8>, position: Vec3 },

    /// Computational cycles for genome execution
    Compute { cycles: u64, position: Vec3 },
}

impl Resource {
    pub fn position(&self) -> Vec3 {
        match self {
            Resource::Energy { position, .. } => *position,
            Resource::Information { position, .. } => *position,
            Resource::Compute { position, .. } => *position,
        }
    }
}

/// Actions that agents can perform in the manifold.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Action {
    /// Move to a new position
    Move { target: Vec3 },

    /// Consume a resource at the current location
    Consume { resource_id: String },

    /// Replicate, creating offspring (requires sufficient energy)
    Replicate { partner_id: Option<String> },

    /// Broadcast a message to nearby agents
    Broadcast { message: Vec<u8> },

    /// Submit a governance proposal
    Propose { proposal: Proposal },

    /// Vote on a governance proposal
    Vote { proposal_id: String, support: bool },
}

/// Identity type for agents participating in governance.
pub type IdentityId = String;

/// Types of governance proposals.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProposalType {
    /// Modify network parameters
    ParameterChange { key: String, value: String },

    /// Allocate resources to a specific agent or location
    ResourceAllocation { amount: u64, recipient: String },

    /// Update the protocol rules
    ProtocolUpgrade { version: String, cid: String },

    /// Custom proposal with arbitrary data
    Custom { data: Vec<u8> },
}

/// A governance proposal for network-wide decisions.
///
/// Proposals are voted on by agents weighted by their contribution metrics.
/// // TODO: Implement quadratic voting and conviction voting (Section 4.2)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Proposal {
    pub id: String,
    pub proposer: IdentityId,
    pub proposal_type: ProposalType,
    pub description: String,
    pub created_at: u64,
    pub voting_ends_at: u64,
    pub votes_for: u64,
    pub votes_against: u64,
}

impl Proposal {
    pub fn new(
        id: String,
        proposer: IdentityId,
        proposal_type: ProposalType,
        description: String,
        voting_period_ms: u64,
    ) -> Self {
        let created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        Self {
            id,
            proposer,
            proposal_type,
            description,
            created_at,
            voting_ends_at: created_at + voting_period_ms,
            votes_for: 0,
            votes_against: 0,
        }
    }

    pub fn is_active(&self) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        now < self.voting_ends_at
    }
}

// Custom serialization for PeerId
mod peer_id_serde {
    use libp2p_identity::PeerId;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(peer_id: &PeerId, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&peer_id.to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<PeerId, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_genome_serialization() {
        let genome = Genome::new(
            "QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG".to_string(),
            vec![1, 2, 3, 4, 5],
        );

        let json = genome.to_json().expect("Serialization failed");
        let deserialized = Genome::from_json(&json).expect("Deserialization failed");

        assert_eq!(genome, deserialized);
    }

    #[test]
    fn test_genome_validation() {
        let valid_genome = Genome::new("QmValid".to_string(), vec![1, 2, 3]);
        assert!(valid_genome.validate().is_ok());

        let invalid_genome = Genome::new(String::new(), vec![1, 2, 3]);
        assert!(invalid_genome.validate().is_err());
    }

    #[test]
    fn test_proposal_lifecycle() {
        let proposal = Proposal::new(
            "prop-001".to_string(),
            "agent-123".to_string(),
            ProposalType::ParameterChange {
                key: "max_energy".to_string(),
                value: "1000".to_string(),
            },
            "Increase maximum energy cap".to_string(),
            86400000, // 24 hours
        );

        assert!(proposal.is_active());
        assert_eq!(proposal.votes_for, 0);
        assert_eq!(proposal.votes_against, 0);
    }
}
