//! Network initialization and event handling.

use crate::behaviour::{HandoffResponse, ManifoldBehaviour, ManifoldBehaviourEvent, SpawnRequest, SpawnResponse};
use crate::simulation::Simulation;
use anyhow::Result;
use futures::StreamExt;
use libp2p::{
    gossipsub, identify, identity, kad, noise, request_response, tcp, yamux, Multiaddr, PeerId, Swarm,
    SwarmBuilder,
};
use manifold_protocol::consensus::{ConsensusResult, StateCommit, StateProposal, StateVote};
use manifold_protocol::AgentHandoff;
use std::collections::HashMap;
use std::time::Duration;
use tracing::{error, info, warn};

/// Topic for broadcasting agent actions.
const ACTIONS_TOPIC: &str = "manifold-actions";

/// Main network coordinator managing libp2p swarm and simulation state.
pub struct Network {
    swarm: Swarm<ManifoldBehaviour>,
    simulation: Simulation,
    consensus_round: u64,
    pending_votes: HashMap<u64, Vec<StateVote>>,
    known_peers: Vec<PeerId>,
}

impl Network {
    /// Create a new network instance with initialized swarm and simulation.
    pub async fn new() -> Result<Self> {
        // Generate identity keypair
        let local_key = identity::Keypair::generate_ed25519();
        let local_peer_id = PeerId::from(local_key.public());

        info!("Local peer ID: {}", local_peer_id);

        // Create custom behaviour
        let behaviour = ManifoldBehaviour::new(local_peer_id, &local_key)
            .map_err(|e| anyhow::anyhow!("Failed to create behaviour: {}", e))?;

        // Build swarm with TCP transport, Noise encryption, and Yamux multiplexing
        let swarm = SwarmBuilder::with_existing_identity(local_key)
            .with_tokio()
            .with_tcp(
                tcp::Config::default(),
                noise::Config::new,
                yamux::Config::default,
            )?
            .with_behaviour(|_| behaviour)?
            .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
            .build();

        let simulation = Simulation::new(local_peer_id);

        Ok(Self {
            swarm,
            simulation,
            consensus_round: 0,
            pending_votes: HashMap::new(),
            known_peers: Vec::new(),
        })
    }

    /// Get the local peer ID.
    pub fn local_peer_id(&self) -> PeerId {
        *self.swarm.local_peer_id()
    }

    /// Subscribe to the actions topic for agent activity broadcasts.
    fn subscribe_topics(&mut self) -> Result<()> {
        let topic = gossipsub::IdentTopic::new(ACTIONS_TOPIC);
        self.swarm.behaviour_mut().gossipsub.subscribe(&topic)?;
        info!("Subscribed to topic: {}", ACTIONS_TOPIC);
        Ok(())
    }

    /// Start listening on the default multiaddr.
    pub async fn listen(&mut self) -> Result<()> {
        let listen_addr: Multiaddr = "/ip4/0.0.0.0/tcp/0".parse()?;
        self.swarm.listen_on(listen_addr)?;
        info!("Listening on all interfaces");
        Ok(())
    }

    /// Main event loop processing network events and simulation ticks.
    pub async fn run(mut self) -> Result<()> {
        self.subscribe_topics()?;
        self.listen().await?;

        // Simulation tick interval (100ms)
        let mut tick_interval = tokio::time::interval(Duration::from_millis(100));

        loop {
            tokio::select! {
                // Process network events
                event = self.swarm.select_next_some() => {
                    self.handle_event(event).await;
                }

                // Process simulation ticks
                _ = tick_interval.tick() => {
                    self.simulation.tick();
                    
                    // Process pending agent handoffs
                    self.process_agent_handoffs().await;
                    
                    // Initiate consensus round after each tick
                    if let Err(e) = self.initiate_consensus_round().await {
                        error!("Consensus round failed: {}", e);
                    }
                }
            }
        }
    }

    /// Handle incoming network events.
    async fn handle_event(&mut self, event: libp2p::swarm::SwarmEvent<ManifoldBehaviourEvent>) {
        match event {
            libp2p::swarm::SwarmEvent::NewListenAddr { address, .. } => {
                info!("🎧 Listening on {}", address);
            }

            libp2p::swarm::SwarmEvent::Behaviour(ManifoldBehaviourEvent::Gossipsub(
                gossipsub::Event::Message {
                    propagation_source,
                    message,
                    ..
                },
            )) => {
                info!(
                    "📨 Received message from {}: {:?}",
                    propagation_source,
                    String::from_utf8_lossy(&message.data)
                );
                // TODO: Decode and process Action events
            }

            libp2p::swarm::SwarmEvent::Behaviour(ManifoldBehaviourEvent::Kademlia(
                kad::Event::RoutingUpdated {
                    peer, addresses, ..
                },
            )) => {
                info!("🗺️  Routing updated for peer {}: {:?}", peer, addresses);
            }

            libp2p::swarm::SwarmEvent::Behaviour(ManifoldBehaviourEvent::Identify(
                identify::Event::Received { peer_id, info },
            )) => {
                info!("🔍 Identified peer {}: {:?}", peer_id, info.protocol_version);
                
                // Track known peers for consensus
                if !self.known_peers.contains(&peer_id) {
                    self.known_peers.push(peer_id);
                }
                
                // Add peer to Kademlia
                for addr in info.listen_addrs {
                    self.swarm
                        .behaviour_mut()
                        .kademlia
                        .add_address(&peer_id, addr);
                }
            }

            libp2p::swarm::SwarmEvent::Behaviour(ManifoldBehaviourEvent::RequestResponse(
                request_response::Event::Message { peer, message },
            )) => {
                self.handle_spawn_protocol(peer, message).await;
            }

            libp2p::swarm::SwarmEvent::Behaviour(ManifoldBehaviourEvent::Consensus(
                request_response::Event::Message { peer, message },
            )) => {
                self.handle_consensus_protocol(peer, message).await;
            }

            libp2p::swarm::SwarmEvent::Behaviour(ManifoldBehaviourEvent::Handoff(
                request_response::Event::Message { peer, message },
            )) => {
                self.handle_handoff_protocol(peer, message).await;
            }

            libp2p::swarm::SwarmEvent::ConnectionEstablished {
                peer_id, endpoint, ..
            } => {
                info!("🤝 Connection established with {}: {:?}", peer_id, endpoint);
            }

            libp2p::swarm::SwarmEvent::ConnectionClosed {
                peer_id, cause, ..
            } => {
                warn!("🔌 Connection closed with {}: {:?}", peer_id, cause);
            }

            _ => {}
        }
    }

    /// Handle spawn protocol requests.
    async fn handle_spawn_protocol(
        &mut self,
        peer: PeerId,
        message: request_response::Message<SpawnRequest, SpawnResponse>,
    ) {
        match message {
            request_response::Message::Request {
                request, channel, ..
            } => {
                info!("🐣 Spawn request from {}: {:?}", peer, request);

                // TODO: Validate CID and energy requirements
                let response = match self.simulation.spawn_agent(&request.cid, request.initial_energy) {
                    Ok(agent_id) => SpawnResponse {
                        success: true,
                        agent_id: Some(agent_id),
                        message: "Agent spawned successfully".to_string(),
                    },
                    Err(e) => SpawnResponse {
                        success: false,
                        agent_id: None,
                        message: format!("Spawn failed: {}", e),
                    },
                };

                if let Err(e) = self
                    .swarm
                    .behaviour_mut()
                    .request_response
                    .send_response(channel, response)
                {
                    error!("Failed to send spawn response: {}", e);
                }
            }

            request_response::Message::Response { response, .. } => {
                info!("🐣 Spawn response: {:?}", response);
            }
        }
    }

    /// Initiate a new consensus round by proposing the current state hash.
    async fn initiate_consensus_round(&mut self) -> Result<()> {
        // TODO: Implement proper leader election protocol (Raft/Paxos)
        // For now, use simple round-robin based on peer ID comparison
        
        if self.known_peers.is_empty() {
            // Single node, no need for consensus
            return Ok(());
        }

        let mut all_peers = self.known_peers.clone();
        all_peers.push(self.local_peer_id());
        all_peers.sort();

        let leader_index = (self.consensus_round as usize) % all_peers.len();
        let leader = all_peers[leader_index];

        if leader == self.local_peer_id() {
            // We are the leader, propose state
            let state_hash = self.simulation.calculate_state_hash();
            let proposal = StateProposal {
                round_id: self.consensus_round,
                tick: self.simulation.tick_count,
                state_hash,
                leader: self.local_peer_id(),
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)?
                    .as_secs(),
            };

            info!(
                "🎯 Leader for round {}: proposing state hash {:?}",
                self.consensus_round, proposal.state_hash
            );

            // Send proposal to all peers
            for peer in &self.known_peers {
                self.swarm
                    .behaviour_mut()
                    .consensus
                    .send_request(peer, proposal.clone());
            }
        }

        Ok(())
    }

    /// Handle consensus protocol messages (proposals and votes).
    async fn handle_consensus_protocol(
        &mut self,
        peer: PeerId,
        message: request_response::Message<StateProposal, StateVote>,
    ) {
        match message {
            request_response::Message::Request {
                request, channel, ..
            } => {
                info!(
                    "🗳️  Received state proposal for round {} from {}",
                    request.round_id, peer
                );

                // Compute our state hash
                let our_hash = self.simulation.calculate_state_hash();
                let agree = our_hash == request.state_hash;

                if !agree {
                    warn!(
                        "⚠️  State mismatch in round {}: expected {:?}, got {:?}",
                        request.round_id, our_hash, request.state_hash
                    );
                }

                // Send vote back to leader
                let vote = StateVote {
                    round_id: request.round_id,
                    voter: self.local_peer_id(),
                    agree,
                    voter_hash: our_hash,
                };

                if let Err(e) = self
                    .swarm
                    .behaviour_mut()
                    .consensus
                    .send_response(channel, vote.clone())
                {
                    error!("Failed to send consensus vote: {}", e);
                } else {
                    info!(
                        "✅ Voted {} for round {}",
                        if agree { "AGREE" } else { "DISAGREE" },
                        request.round_id
                    );
                }
            }

            request_response::Message::Response { response, .. } => {
                info!(
                    "📊 Received vote from {} for round {}: {}",
                    response.voter,
                    response.round_id,
                    if response.agree { "AGREE" } else { "DISAGREE" }
                );

                // Collect vote
                self.pending_votes
                    .entry(response.round_id)
                    .or_insert_with(Vec::new)
                    .push(response.clone());

                // Check if we have enough votes for consensus
                if let Some(votes) = self.pending_votes.get(&response.round_id) {
                    let total_nodes = self.known_peers.len() + 1; // +1 for self
                    let agree_count = votes.iter().filter(|v| v.agree).count();
                    let total_count = votes.len();
                    let consensus_result = ConsensusResult::check(agree_count, total_count, total_nodes);

                    match consensus_result {
                        ConsensusResult::Achieved { agree_count, total_count } => {
                            info!(
                                "✅ Consensus achieved for round {} with {}/{} votes",
                                response.round_id, agree_count, total_count
                            );

                            let commit = StateCommit {
                                round_id: response.round_id,
                                tick: self.simulation.tick_count,
                                state_hash: response.voter_hash,
                                vote_count: total_count,
                            };

                            // Broadcast commit over gossipsub
                            if let Ok(commit_bytes) = serde_json::to_vec(&commit) {
                                let topic = gossipsub::IdentTopic::new(ACTIONS_TOPIC);
                                if let Err(e) = self
                                    .swarm
                                    .behaviour_mut()
                                    .gossipsub
                                    .publish(topic, commit_bytes)
                                {
                                    error!("Failed to broadcast consensus commit: {}", e);
                                }
                            }

                            // Clean up votes and advance round
                            self.pending_votes.remove(&response.round_id);
                            self.consensus_round += 1;
                        }
                        ConsensusResult::Failed { .. } => {
                            error!(
                                "❌ Consensus failed for round {}: insufficient agreement",
                                response.round_id
                            );
                            // TODO: Implement state recovery or rollback mechanism
                            self.pending_votes.remove(&response.round_id);
                            self.consensus_round += 1;
                        }
                        ConsensusResult::Pending => {
                            // Still waiting for more votes
                        }
                    }
                }
            }
        }
    }

    /// Process pending agent handoffs by sending them to target nodes.
    async fn process_agent_handoffs(&mut self) {
        let handoffs = self.simulation.take_pending_handoffs();
        
        for handoff in handoffs {
            info!(
                "📤 Sending agent handoff from sector {} to sector {}",
                handoff.from_sector, handoff.to_sector
            );
            
            // TODO: Look up target node for sector via DHT
            // For now, broadcast to all known peers
            for peer in &self.known_peers {
                self.swarm
                    .behaviour_mut()
                    .handoff
                    .send_request(peer, handoff.clone());
            }
        }
    }

    /// Handle agent handoff protocol messages.
    async fn handle_handoff_protocol(
        &mut self,
        peer: PeerId,
        message: request_response::Message<AgentHandoff, HandoffResponse>,
    ) {
        match message {
            request_response::Message::Request {
                request, channel, ..
            } => {
                info!(
                    "📥 Received agent handoff from {} (sector {} -> {})",
                    peer, request.from_sector, request.to_sector
                );

                let response = match self.simulation.receive_agent_handoff(request) {
                    Ok(()) => HandoffResponse {
                        success: true,
                        message: "Agent accepted".to_string(),
                    },
                    Err(e) => HandoffResponse {
                        success: false,
                        message: format!("Handoff failed: {}", e),
                    },
                };

                if let Err(e) = self
                    .swarm
                    .behaviour_mut()
                    .handoff
                    .send_response(channel, response)
                {
                    error!("Failed to send handoff response: {}", e);
                }
            }

            request_response::Message::Response { response, .. } => {
                if response.success {
                    info!("✅ Agent handoff confirmed: {}", response.message);
                } else {
                    error!("❌ Agent handoff rejected: {}", response.message);
                }
            }
        }
    }
}

