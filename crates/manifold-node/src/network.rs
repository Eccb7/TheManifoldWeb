//! Network initialization and event handling.

use crate::behaviour::{ManifoldBehaviour, ManifoldBehaviourEvent, SpawnRequest, SpawnResponse};
use crate::simulation::Simulation;
use anyhow::Result;
use libp2p::{
    gossipsub, identity, noise, tcp, yamux, Multiaddr, PeerId, Swarm, SwarmBuilder,
};
use std::time::Duration;
use tracing::{error, info, warn};

/// Topic for broadcasting agent actions.
const ACTIONS_TOPIC: &str = "manifold-actions";

/// Main network coordinator managing libp2p swarm and simulation state.
pub struct Network {
    swarm: Swarm<ManifoldBehaviour>,
    simulation: Simulation,
}

impl Network {
    /// Create a new network instance with initialized swarm and simulation.
    pub async fn new() -> Result<Self> {
        // Generate identity keypair
        let local_key = identity::Keypair::generate_ed25519();
        let local_peer_id = PeerId::from(local_key.public());

        info!("Local peer ID: {}", local_peer_id);

        // Create custom behaviour
        let behaviour = ManifoldBehaviour::new(local_peer_id, &local_key)?;

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

        Ok(Self { swarm, simulation })
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
}
