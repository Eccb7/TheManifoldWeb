//! Agent simulation engine and genome evolution.
//!
//! Manages agent lifecycle, energy consumption, reproduction, and genetic
//! algorithms for offspring generation.

use glam::Vec3;
use libp2p::PeerId;
use manifold_protocol::{Agent, Genome};
use rand::Rng;
use std::collections::HashMap;
use tracing::{debug, info};

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
}

impl Simulation {
    /// Create a new simulation instance.
    pub fn new(local_peer_id: PeerId) -> Self {
        Self {
            local_peer_id,
            agents: HashMap::new(),
            tick_count: 0,
            mutation_rate: 0.01, // 1% mutation rate
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

        // TODO: Process agent actions from their genome execution
        // TODO: Apply energy decay
        // TODO: Check for replication conditions
        // TODO: Remove agents with zero energy
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

        let agent = Agent::new(agent_id, genome, initial_energy, position);
        let agent_id_str = agent_id.to_string();

        self.agents.insert(agent_id_str.clone(), agent);

        info!(
            "🐣 Spawned agent {} at position {:?} with {} energy",
            agent_id_str, position, initial_energy
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
}
