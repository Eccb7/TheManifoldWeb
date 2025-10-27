//! Agent simulation engine and genome evolution.
//!
//! Manages agent lifecycle, energy consumption, reproduction, and genetic
//! algorithms for offspring generation.

use glam::Vec3;
use libp2p::PeerId;
use manifold_protocol::{Agent, AgentHandoff, Genome, SectorId, StateHash};
use rand::Rng;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use tracing::{debug, info, warn};

/// Manages sector assignment and agent handoff for distributed simulation.
///
/// The manifold space is partitioned into 3D sectors. Each node manages
/// a subset of sectors, and agents are handed off between nodes when they
/// move across sector boundaries.
pub struct SectorManager {
    /// Size of each sector (cubic dimension)
    sector_size: f32,
    
    /// Sectors managed by this node
    local_sectors: Vec<SectorId>,
    
    /// Mapping of sector IDs to responsible node peer IDs
    /// TODO: Implement DHT-based sector assignment for decentralization
    sector_ownership: HashMap<SectorId, PeerId>,
}

impl SectorManager {
    pub fn new(sector_size: f32, local_peer_id: PeerId) -> Self {
        let mut manager = Self {
            sector_size,
            local_sectors: Vec::new(),
            sector_ownership: HashMap::new(),
        };
        
        // Start by claiming sector 0
        manager.claim_sector(0, local_peer_id);
        
        manager
    }
    
    /// Claim ownership of a sector.
    pub fn claim_sector(&mut self, sector_id: SectorId, owner: PeerId) {
        self.sector_ownership.insert(sector_id, owner);
        if !self.local_sectors.contains(&sector_id) {
            self.local_sectors.push(sector_id);
        }
    }
    
    /// Get the node responsible for a given sector.
    pub fn get_sector_owner(&self, sector_id: SectorId) -> Option<PeerId> {
        self.sector_ownership.get(&sector_id).copied()
    }
    
    /// Check if this node manages a given sector.
    pub fn is_local_sector(&self, sector_id: SectorId) -> bool {
        self.local_sectors.contains(&sector_id)
    }
    
    /// Calculate which sector a position belongs to.
    pub fn position_to_sector(&self, position: Vec3) -> SectorId {
        let x_grid = (position.x / self.sector_size).floor() as i64;
        let y_grid = (position.y / self.sector_size).floor() as i64;
        let z_grid = (position.z / self.sector_size).floor() as i64;
        
        // Spatial hash function
        let hash = (x_grid.wrapping_mul(73856093) 
                   ^ y_grid.wrapping_mul(19349663) 
                   ^ z_grid.wrapping_mul(83492791)) as u64;
        hash
    }
    
    /// Prepare an agent for handoff to another sector.
    pub fn prepare_handoff(&self, agent: &Agent, target_sector: SectorId, source_node: PeerId) -> AgentHandoff {
        AgentHandoff::new(
            agent.clone(),
            agent.sector_id,
            target_sector,
            source_node,
        )
    }
}

/// Manages the simulation state for all agents in the local node.
pub struct Simulation {
    /// Local peer ID
    local_peer_id: PeerId,

    /// Active agents indexed by their ID string
    agents: HashMap<String, Agent>,

    /// Simulation tick counter
    tick_count: u64,

    /// Mutation rate for genetic algorithm (0.0 to 1.0)
    mutation_rate: f64,
    
    /// Sector manager for distributed simulation
    sector_manager: SectorManager,
    
    /// Pending agent handoffs to other nodes
    pending_handoffs: Vec<AgentHandoff>,
}

impl Simulation {
    /// Create a new simulation instance.
    pub fn new(local_peer_id: PeerId) -> Self {
        Self {
            local_peer_id,
            agents: HashMap::new(),
            tick_count: 0,
            mutation_rate: 0.01, // 1% mutation rate
            sector_manager: SectorManager::new(100.0, local_peer_id), // 100 unit sectors
            pending_handoffs: Vec::new(),
        }
    }

    /// Process a single simulation tick.
    ///
    /// Updates agent states, processes queued actions, and handles energy
    /// decay.
    pub fn tick(&mut self) {
        self.tick_count += 1;

        if self.tick_count % 100 == 0 {
            debug!(
                "Simulation tick {}: {} agents active",
                self.tick_count,
                self.agents.len()
            );
        }

        // Check for agents that need sector reassignment
        self.check_sector_boundaries();

        // TODO: Process agent actions from their genome execution
        // TODO: Apply energy decay
        // TODO: Check for replication conditions
        // TODO: Remove agents with zero energy
    }
    
    /// Check if any agents have moved across sector boundaries and prepare handoffs.
    fn check_sector_boundaries(&mut self) {
        let mut agents_to_handoff = Vec::new();
        
        for (agent_id, agent) in &mut self.agents {
            let new_sector = self.sector_manager.position_to_sector(agent.position);
            
            if new_sector != agent.sector_id {
                // Agent has crossed sector boundary
                if self.sector_manager.is_local_sector(new_sector) {
                    // Simple local reassignment
                    info!(
                        "🔄 Agent {} moved from sector {} to local sector {}",
                        agent_id, agent.sector_id, new_sector
                    );
                    agent.sector_id = new_sector;
                } else {
                    // Need to hand off to another node
                    // TODO: Implement redundant ghost agents in overlapping boundary zones
                    warn!(
                        "📦 Agent {} needs handoff from sector {} to remote sector {}",
                        agent_id, agent.sector_id, new_sector
                    );
                    
                    let handoff = self.sector_manager.prepare_handoff(
                        agent,
                        new_sector,
                        self.local_peer_id,
                    );
                    agents_to_handoff.push((agent_id.clone(), handoff));
                }
            }
        }
        
        // Queue handoffs and remove agents
        for (agent_id, handoff) in agents_to_handoff {
            self.pending_handoffs.push(handoff);
            self.agents.remove(&agent_id);
            info!("📤 Agent {} queued for handoff", agent_id);
        }
    }
    
    /// Accept an agent from another node via handoff.
    pub fn receive_agent_handoff(&mut self, handoff: AgentHandoff) -> anyhow::Result<()> {
        let agent_id = handoff.agent.id.to_string();
        
        // Verify this node manages the target sector
        if !self.sector_manager.is_local_sector(handoff.to_sector) {
            anyhow::bail!(
                "Cannot accept handoff for sector {} - not managed by this node",
                handoff.to_sector
            );
        }
        
        let mut agent = handoff.agent;
        agent.sector_id = handoff.to_sector;
        
        self.agents.insert(agent_id.clone(), agent);
        info!(
            "📥 Accepted agent {} handoff from sector {} to sector {}",
            agent_id, handoff.from_sector, handoff.to_sector
        );
        
        Ok(())
    }
    
    /// Get pending handoffs and clear the queue.
    pub fn take_pending_handoffs(&mut self) -> Vec<AgentHandoff> {
        std::mem::take(&mut self.pending_handoffs)
    }

    /// Calculate a deterministic hash of the current simulation state.
    ///
    /// This hash is used for consensus - all nodes should compute the same
    /// hash if they have the same simulation state.
    ///
    /// # Returns
    /// A 32-byte SHA-256 hash of the simulation state.
    pub fn calculate_state_hash(&self) -> StateHash {
        let mut hasher = Sha256::new();

        // Hash tick count
        hasher.update(self.tick_count.to_le_bytes());

        // Hash each agent in a deterministic order (sorted by ID)
        let mut agent_ids: Vec<_> = self.agents.keys().collect();
        agent_ids.sort();

        for agent_id in agent_ids {
            if let Some(agent) = self.agents.get(agent_id) {
                // Hash agent ID
                hasher.update(agent_id.as_bytes());

                // Hash agent energy
                hasher.update(agent.energy.to_le_bytes());

                // Hash agent position (convert to bytes)
                hasher.update(agent.position.x.to_le_bytes());
                hasher.update(agent.position.y.to_le_bytes());
                hasher.update(agent.position.z.to_le_bytes());

                // Hash genome CID
                hasher.update(agent.genome.cid.as_bytes());

                // Hash genome parameters
                hasher.update(&agent.genome.parameters);
            }
        }

        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        hash
    }

    /// Spawn a new agent with the given genome CID and initial energy.
    pub fn spawn_agent(&mut self, cid: &str, initial_energy: u64) -> anyhow::Result<String> {
        // Generate a new peer ID for the agent
        let agent_id = PeerId::random();

        // Create genome with placeholder parameters
        // TODO: Fetch WASM module from IPFS using CID
        let genome = Genome::new(cid.to_string(), vec![0; 32]);

        // Spawn at random position
        let mut rng = rand::thread_rng();
        let position = Vec3::new(
            rng.gen_range(-100.0..100.0),
            rng.gen_range(-100.0..100.0),
            rng.gen_range(-100.0..100.0),
        );

        let mut agent = Agent::new(agent_id, genome, initial_energy, position);
        
        // Assign agent to appropriate sector
        let sector_id = self.sector_manager.position_to_sector(position);
        agent.sector_id = sector_id;
        
        let agent_id_str = agent_id.to_string();
        self.agents.insert(agent_id_str.clone(), agent);

        info!(
            "🐣 Spawned agent {} at position {:?} in sector {} with {} energy",
            agent_id_str, position, sector_id, initial_energy
        );

        Ok(agent_id_str)
    }

    /// Execute genome in a sandboxed WASM runtime.
    ///
    /// This is a placeholder demonstrating the planned WASM execution
    /// architecture.
    /// // TODO: Implement WASM sandboxing using wasmtime or wasmer (Section
    /// 3.1)
    #[allow(dead_code)]
    fn execute_genome_stub(&self, _genome: &Genome) -> anyhow::Result<Vec<u8>> {
        // Placeholder: In production, this will:
        // 1. Fetch WASM blob from IPFS using genome.cid
        // 2. Instantiate WASM module in sandboxed runtime
        // 3. Inject genome.parameters as input
        // 4. Execute and collect output actions
        // 5. Apply resource limits (CPU cycles, memory)

        Ok(vec![])
    }

    /// Evolve offspring from two parent agents using genetic algorithms.
    ///
    /// Performs single-point crossover on parameter byte arrays and applies
    /// bit-flip mutation based on the configured mutation rate.
    ///
    /// # Algorithm
    /// 1. Select random crossover point
    /// 2. Create offspring by combining parent1[0..point] + parent2[point..]
    /// 3. Apply bit-flip mutations with probability `mutation_rate`
    ///
    /// # Returns
    /// A new `Genome` with the same CID as parent_a (inheritance model)
    /// and evolved parameters.
    pub fn evolve_offspring(
        &self,
        parent_a: &Agent,
        parent_b: &Agent,
        mutation_rate: f64,
    ) -> Genome {
        let params_a = &parent_a.genome.parameters;
        let params_b = &parent_b.genome.parameters;

        // Ensure both parameter arrays have data
        if params_a.is_empty() || params_b.is_empty() {
            return Genome::new(parent_a.genome.cid.clone(), params_a.clone());
        }

        // Single-point crossover
        let mut rng = rand::thread_rng();
        let crossover_point = rng.gen_range(0..params_a.len().min(params_b.len()));

        let mut offspring_params = Vec::with_capacity(params_a.len());
        offspring_params.extend_from_slice(&params_a[..crossover_point]);
        offspring_params.extend_from_slice(&params_b[crossover_point..]);

        // Pad if parent_a is longer
        if params_a.len() > crossover_point + params_b.len() - crossover_point {
            offspring_params
                .extend_from_slice(&params_a[params_b.len()..]);
        }

        // Apply bit-flip mutation
        for byte in &mut offspring_params {
            if rng.gen::<f64>() < mutation_rate {
                let bit_position = rng.gen_range(0..8);
                *byte ^= 1 << bit_position;
            }
        }

        // Inherit CID from parent_a (simple inheritance model)
        // TODO: Support multiple genome templates and adaptive CID selection
        Genome::new(parent_a.genome.cid.clone(), offspring_params)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evolve_offspring() {
        let sim = Simulation::new(PeerId::random());

        let parent_a = Agent::new(
            PeerId::random(),
            Genome::new("QmTest1".to_string(), vec![0xFF; 10]),
            100,
            Vec3::ZERO,
        );

        let parent_b = Agent::new(
            PeerId::random(),
            Genome::new("QmTest2".to_string(), vec![0x00; 10]),
            100,
            Vec3::ZERO,
        );

        let offspring = sim.evolve_offspring(&parent_a, &parent_b, 0.1);

        // Offspring should inherit parent_a's CID
        assert_eq!(offspring.cid, "QmTest1");

        // Offspring parameters should be mix of both parents
        assert_eq!(offspring.parameters.len(), 10);

        // Not all bytes should be 0xFF or 0x00 (crossover happened)
        let all_ff = offspring.parameters.iter().all(|&b| b == 0xFF);
        let all_00 = offspring.parameters.iter().all(|&b| b == 0x00);
        assert!(!(all_ff || all_00), "Crossover should produce mixed genes");
    }

    #[test]
    fn test_spawn_agent() {
        let mut sim = Simulation::new(PeerId::random());

        let result = sim.spawn_agent("QmTestCID", 1000);
        assert!(result.is_ok());

        let agent_id = result.unwrap();
        assert!(sim.agents.contains_key(&agent_id));
        assert_eq!(sim.agents[&agent_id].energy, 1000);
    }

    #[test]
    fn test_state_hash_deterministic() {
        let mut sim1 = Simulation::new(PeerId::random());
        let mut sim2 = Simulation::new(PeerId::random());

        // Same state should produce same hash
        let hash1 = sim1.calculate_state_hash();
        let hash2 = sim2.calculate_state_hash();
        assert_eq!(hash1, hash2);

        // Add same agent to both
        let agent_id = "test-agent-1";
        let genome = Genome::new("QmTest".to_string(), vec![1, 2, 3]);
        let agent = Agent::new(PeerId::random(), genome, 100, Vec3::ZERO);
        
        sim1.agents.insert(agent_id.to_string(), agent.clone());
        sim2.agents.insert(agent_id.to_string(), agent);

        let hash1 = sim1.calculate_state_hash();
        let hash2 = sim2.calculate_state_hash();
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_state_hash_changes_with_state() {
        let mut sim = Simulation::new(PeerId::random());
        
        let hash1 = sim.calculate_state_hash();
        
        // Spawn an agent
        sim.spawn_agent("QmTest", 100).unwrap();
        
        let hash2 = sim.calculate_state_hash();
        
        // Hashes should be different
        assert_ne!(hash1, hash2);
    }
}
