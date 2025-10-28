//! Observer implementation for monitoring network activity.

use crate::dead_reckoning::{DeadReckoning, PredictedAgent};
use anyhow::Result;
use futures::StreamExt;
use glam::Vec3;
use libp2p::{
    gossipsub,
    identity,
    noise,
    swarm::NetworkBehaviour,
    tcp,
    yamux,
    Multiaddr,
    PeerId,
    Swarm,
    SwarmBuilder,
};
use manifold_protocol::Action;
use std::collections::HashMap;
use std::time::Duration;
use tracing::{info, warn};

/// Topic for monitoring agent actions.
const ACTIONS_TOPIC: &str = "manifold-actions";

/// Read-only observer that monitors network gossipsub messages.
pub struct Observer {
    swarm: Swarm<ObserverBehaviour>,
    /// Dead reckoning engine for smooth agent position prediction
    dead_reckoning: DeadReckoning,
    /// Map of agent IDs to their predicted states
    predicted_agents: HashMap<String, PredictedAgent>,
}

/// Minimal network behaviour for read-only observation.
#[derive(NetworkBehaviour)]
struct ObserverBehaviour {
    gossipsub: gossipsub::Behaviour,
}

impl Observer {
    /// Create a new observer instance.
    pub async fn new() -> Result<Self> {
        let local_key = identity::Keypair::generate_ed25519();
        let local_peer_id = PeerId::from(local_key.public());

        info!("Observer peer ID: {}", local_peer_id);

        // Configure Gossipsub for read-only mode
        let gossipsub_config = gossipsub::ConfigBuilder::default()
            .heartbeat_interval(Duration::from_secs(1))
            .validation_mode(gossipsub::ValidationMode::Permissive) // More lenient for observers
            .build()
            .map_err(|e| anyhow::anyhow!("Gossipsub config error: {}", e))?;

        let gossipsub = gossipsub::Behaviour::new(
            gossipsub::MessageAuthenticity::Signed(local_key.clone()),
            gossipsub_config,
        )
        .map_err(|e| anyhow::anyhow!("Gossipsub initialization error: {}", e))?;

        let behaviour = ObserverBehaviour {
            gossipsub,
        };

        // Build swarm
        let swarm = SwarmBuilder::with_existing_identity(local_key)
            .with_tokio()
            .with_tcp(
                tcp::Config::default(),
                noise::Config::new,
                yamux::Config::default,
            )?
            .with_behaviour(|_| behaviour)?
            .with_swarm_config(|c| {
                c.with_idle_connection_timeout(Duration::from_secs(60))
            })
            .build();

        Ok(Self {
            swarm,
            dead_reckoning: DeadReckoning::new(),
            predicted_agents: HashMap::new(),
        })
    }

    /// Subscribe to observation topics.
    fn subscribe_topics(&mut self) -> Result<()> {
        let topic = gossipsub::IdentTopic::new(ACTIONS_TOPIC);
        self.swarm.behaviour_mut().gossipsub.subscribe(&topic)?;
        info!("📡 Subscribed to topic: {}", ACTIONS_TOPIC);
        Ok(())
    }

    /// Start listening on a local address.
    pub async fn listen(&mut self) -> Result<()> {
        let listen_addr: Multiaddr = "/ip4/0.0.0.0/tcp/0".parse()?;
        self.swarm.listen_on(listen_addr)?;
        Ok(())
    }

    /// Connect to a known node (optional, for bootstrapping).
    #[allow(dead_code)]
    pub fn connect(&mut self, addr: Multiaddr) -> Result<()> {
        self.swarm.dial(addr)?;
        info!("Connecting to node...");
        Ok(())
    }

    /// Main observation loop.
    pub async fn run(mut self) -> Result<()> {
        self.subscribe_topics()?;
        self.listen().await?;

        info!("👁️  Observer running. Monitoring network activity...");
        info!("Press Ctrl+C to stop");

        loop {
            match self.swarm.select_next_some().await {
                libp2p::swarm::SwarmEvent::NewListenAddr { address, .. } => {
                    info!("🎧 Listening on {}", address);
                }

                libp2p::swarm::SwarmEvent::Behaviour(
                    ObserverBehaviourEvent::Gossipsub(gossipsub::Event::Message {
                        propagation_source,
                        message_id,
                        message,
                    }),
                ) => {
                    self.handle_action_message(
                        propagation_source,
                        message_id,
                        message,
                    );
                }

                libp2p::swarm::SwarmEvent::ConnectionEstablished {
                    peer_id,
                    endpoint,
                    ..
                } => {
                    info!("🤝 Connected to peer {}: {:?}", peer_id, endpoint);
                }

                libp2p::swarm::SwarmEvent::ConnectionClosed {
                    peer_id,
                    cause,
                    ..
                } => {
                    warn!("🔌 Connection closed with {}: {:?}", peer_id, cause);
                }

                _ => {}
            }
        }
    }

    /// Process and display an action message from the network.
    fn handle_action_message(
        &mut self,
        source: PeerId,
        message_id: gossipsub::MessageId,
        message: gossipsub::Message,
    ) {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Try to decode as Action
        match serde_json::from_slice::<Action>(&message.data) {
            Ok(action) => {
                info!(
                    "📊 [{}] Action from {}: {:?}",
                    timestamp, source, action
                );

                // TODO: Store action for visualization
                // TODO: Update 3D scene with agent positions and movements
                self.display_action(&action, &source.to_string());
            }
            Err(_) => {
                // Not a valid Action, display raw message
                warn!(
                    "📨 [{}] Raw message {} from {}: {}",
                    timestamp,
                    message_id,
                    source,
                    String::from_utf8_lossy(&message.data)
                );
            }
        }
    }

    /// Display action in a human-readable format.
    ///
    /// Uses dead reckoning for smooth position prediction of agent movements.
    /// TODO: Replace console output with 3D visualization using wgpu/rend3
    /// (Section 5.3)
    fn display_action(&mut self, action: &Action, agent_id: &str) {
        match action {
            Action::Move { target } => {
                // Update or create predicted agent state
                if let Some(predicted_agent) = self.predicted_agents.get_mut(agent_id) {
                    // Update existing agent with new authoritative position
                    // For now, we assume zero velocity/acceleration since we don't have
                    // that data in the Action message. In a real implementation,
                    // the state updates would include velocity/acceleration fields.
                    self.dead_reckoning.update_agent(
                        predicted_agent,
                        *target,
                        Vec3::ZERO, // TODO: Get actual velocity from state update
                        Vec3::ZERO, // TODO: Get actual acceleration from state update
                    );

                    // Get smoothed display position
                    let display_pos = self.dead_reckoning.get_display_position(predicted_agent);
                    
                    info!(
                        "  ➡️  Agent {} moved to [{:.2}, {:.2}, {:.2}] (predicted: [{:.2}, {:.2}, {:.2}])",
                        agent_id,
                        target.x, target.y, target.z,
                        display_pos.x, display_pos.y, display_pos.z
                    );
                } else {
                    // Create new predicted agent
                    let predicted_agent = self.dead_reckoning.create_agent(
                        *target,
                        Vec3::ZERO,
                        Vec3::ZERO,
                    );
                    self.predicted_agents.insert(agent_id.to_string(), predicted_agent);
                    
                    info!("  ➡️  Agent {} moved to [{:.2}, {:.2}, {:.2}]", agent_id, target.x, target.y, target.z);
                }
            }
            Action::Consume { resource_id } => {
                info!("  🍽️  Agent {} consumed resource: {}", agent_id, resource_id);
            }
            Action::Replicate { partner_id } => {
                if let Some(partner) = partner_id {
                    info!("  👶 Agent {} replicated with partner: {}", agent_id, partner);
                } else {
                    info!("  👶 Agent {} replicated (asexual)", agent_id);
                }
            }
            Action::Broadcast { message } => {
                info!(
                    "  📢 Agent {} broadcast: {}",
                    agent_id,
                    String::from_utf8_lossy(message)
                );
            }
            Action::Propose { proposal } => {
                info!("  🗳️  Agent {} proposed: {}", agent_id, proposal.description);
            }
            Action::Vote {
                proposal_id,
                support,
            } => {
                let vote = if *support { "FOR" } else { "AGAINST" };
                info!("  ✅ Agent {} voted {} on proposal: {}", agent_id, vote, proposal_id);
            }
        }
    }

    /// Update all predicted agent positions based on elapsed time.
    ///
    /// This should be called regularly (e.g., every frame) to maintain
    /// smooth predictions between authoritative updates.
    #[allow(dead_code)]
    fn update_predictions(&mut self) {
        for (agent_id, predicted_agent) in self.predicted_agents.iter_mut() {
            let _predicted_pos = self.dead_reckoning.predict_position(predicted_agent);
            let _display_pos = self.dead_reckoning.get_display_position(predicted_agent);
            
            // Get time since last update for staleness detection
            let time_since_update = self.dead_reckoning.time_since_update(predicted_agent);
            
            // Log if prediction is getting stale (no updates for > 5 seconds)
            if time_since_update > Duration::from_secs(5) {
                warn!(
                    "⚠️  Agent {} prediction stale: {:.1}s since last update",
                    agent_id,
                    time_since_update.as_secs_f32()
                );
            }
        }
    }
}

// TODO: Implement 3D visualization module
// This would use wgpu for rendering and rend3 for scene graph management
//
// mod visualization {
//     use wgpu;
//     use rend3;
//
//     pub struct Visualizer {
//         // wgpu context
//         // rend3 renderer
//         // Camera state
//         // Agent mesh instances
//     }
//
//     impl Visualizer {
//         pub fn new() -> Self { todo!() }
//         pub fn update_agent_position(&mut self, agent_id: &str, position: Vec3) {
// todo!() }         pub fn render_frame(&mut self) { todo!() }
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_observer_creation() {
        let observer = Observer::new().await;
        assert!(observer.is_ok());
    }
}
