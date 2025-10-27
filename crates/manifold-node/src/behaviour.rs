//! Custom libp2p NetworkBehaviour for The Manifold Web.
//!
//! Combines Kademlia (DHT), Gossipsub (pubsub), and custom protocols for
//! agent spawning and coordination.

use libp2p::{
    gossipsub, identify, kad,
    request_response::{self, ProtocolSupport},
    swarm::NetworkBehaviour,
    StreamProtocol,
};
use manifold_protocol::{consensus::{StateProposal, StateVote}, AgentHandoff};
use serde::{Deserialize, Serialize};

/// Request to spawn a new agent on the network.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnRequest {
    pub cid: String,
    pub initial_energy: u64,
}

/// Response from agent spawn operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpawnResponse {
    pub success: bool,
    pub agent_id: Option<String>,
    pub message: String,
}

/// Response to an agent handoff request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandoffResponse {
    pub success: bool,
    pub message: String,
}

/// The main network behavior combining DHT, pubsub, and custom protocols.
#[derive(NetworkBehaviour)]
pub struct ManifoldBehaviour {
    pub kademlia: kad::Behaviour<kad::store::MemoryStore>,
    pub gossipsub: gossipsub::Behaviour,
    pub identify: identify::Behaviour,
    pub request_response:
        request_response::cbor::Behaviour<SpawnRequest, SpawnResponse>,
    pub consensus:
        request_response::cbor::Behaviour<StateProposal, StateVote>,
    pub handoff:
        request_response::cbor::Behaviour<AgentHandoff, HandoffResponse>,
}

impl ManifoldBehaviour {
    pub fn new(
        local_peer_id: libp2p::PeerId,
        local_key: &libp2p::identity::Keypair,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // Configure Kademlia DHT
        let store = kad::store::MemoryStore::new(local_peer_id);
        let kademlia = kad::Behaviour::new(local_peer_id, store);

        // Configure Gossipsub
        let gossipsub_config = gossipsub::ConfigBuilder::default()
            .heartbeat_interval(std::time::Duration::from_secs(1))
            .validation_mode(gossipsub::ValidationMode::Strict)
            .build()
            .map_err(|e| format!("Gossipsub config error: {}", e))?;

        let gossipsub = gossipsub::Behaviour::new(
            gossipsub::MessageAuthenticity::Signed(local_key.clone()),
            gossipsub_config,
        )
        .map_err(|e| format!("Gossipsub initialization error: {}", e))?;

        // Configure Identify protocol
        let identify = identify::Behaviour::new(identify::Config::new(
            "/manifold/1.0.0".to_string(),
            local_key.public(),
        ));

        // Configure custom request/response protocol for agent spawning
        let request_response = request_response::cbor::Behaviour::new(
            [(
                StreamProtocol::new("/manifold/spawn/1.0.0"),
                ProtocolSupport::Full,
            )],
            request_response::Config::default(),
        );

        // Configure consensus protocol for state proposals and voting
        let consensus = request_response::cbor::Behaviour::new(
            [(
                StreamProtocol::new("/manifold/consensus/1.0.0"),
                ProtocolSupport::Full,
            )],
            request_response::Config::default(),
        );

        // Configure agent handoff protocol for cross-sector transfers
        let handoff = request_response::cbor::Behaviour::new(
            [(
                StreamProtocol::new("/manifold/handoff/1.0.0"),
                ProtocolSupport::Full,
            )],
            request_response::Config::default(),
        );

        Ok(Self {
            kademlia,
            gossipsub,
            identify,
            request_response,
            consensus,
            handoff,
        })
    }
}
