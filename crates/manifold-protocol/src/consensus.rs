//! State consensus protocol for distributed simulation.
//!
//! Implements a simplified Byzantine Fault Tolerance (BFT) voting mechanism
//! to ensure all nodes agree on the simulation state at each tick.

use libp2p_identity::PeerId;
use serde::{Deserialize, Serialize};

/// Hash of the simulation state at a given tick.
pub type StateHash = [u8; 32];

/// Unique identifier for a consensus round.
pub type RoundId = u64;

/// Proposal from the leader for a new state.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StateProposal {
    /// The consensus round identifier
    pub round_id: RoundId,
    
    /// The simulation tick this proposal is for
    pub tick: u64,
    
    /// Hash of the proposed state
    pub state_hash: StateHash,
    
    /// The leader who created this proposal
    #[serde(with = "peer_id_serde")]
    pub leader: PeerId,
    
    /// Timestamp when proposal was created
    pub timestamp: u64,
}

/// Vote response from a peer on a state proposal.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StateVote {
    /// The round being voted on
    pub round_id: RoundId,
    
    /// The voter's peer ID
    #[serde(with = "peer_id_serde")]
    pub voter: PeerId,
    
    /// Whether the voter agrees with the proposal
    pub agree: bool,
    
    /// The hash the voter computed (for debugging)
    pub voter_hash: StateHash,
}

impl std::fmt::Display for StateVote {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "StateVote(round={}, voter={}, agree={})", self.round_id, self.voter, self.agree)
    }
}

/// Commit message broadcast after consensus is reached.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StateCommit {
    /// The round being committed
    pub round_id: RoundId,
    
    /// The tick being committed
    pub tick: u64,
    
    /// The agreed-upon state hash
    pub state_hash: StateHash,
    
    /// Number of votes received
    pub vote_count: usize,
}

/// Consensus vote tallying result.
#[derive(Debug, Clone, PartialEq)]
pub enum ConsensusResult {
    /// Not enough votes yet
    Pending,
    
    /// Consensus achieved with >2/3 majority
    Achieved {
        agree_count: usize,
        total_count: usize,
    },
    
    /// Failed to reach consensus
    Failed {
        agree_count: usize,
        total_count: usize,
    },
}

impl ConsensusResult {
    /// Check if we have achieved consensus with >2/3 majority.
    pub fn check(agree_count: usize, total_count: usize, required_peers: usize) -> Self {
        if total_count < required_peers {
            return Self::Pending;
        }

        // BFT requires >2/3 agreement
        let threshold = (required_peers * 2) / 3;
        
        if agree_count > threshold {
            Self::Achieved {
                agree_count,
                total_count,
            }
        } else {
            Self::Failed {
                agree_count,
                total_count,
            }
        }
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
    fn test_consensus_thresholds() {
        // 4 nodes: need >2 votes (3 or 4)
        assert_eq!(
            ConsensusResult::check(3, 4, 4),
            ConsensusResult::Achieved {
                agree_count: 3,
                total_count: 4
            }
        );

        assert_eq!(
            ConsensusResult::check(2, 4, 4),
            ConsensusResult::Failed {
                agree_count: 2,
                total_count: 4
            }
        );

        assert_eq!(
            ConsensusResult::check(2, 2, 4),
            ConsensusResult::Pending
        );
    }

    #[test]
    fn test_state_proposal_serialization() {
        let proposal = StateProposal {
            round_id: 1,
            tick: 100,
            state_hash: [0u8; 32],
            leader: PeerId::random(),
            timestamp: 12345,
        };

        let json = serde_json::to_string(&proposal).unwrap();
        let deserialized: StateProposal = serde_json::from_str(&json).unwrap();

        assert_eq!(proposal.round_id, deserialized.round_id);
        assert_eq!(proposal.tick, deserialized.tick);
    }
}
