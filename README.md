# The Manifold Web

> *Where information becomes life, and life becomes meaning.*

A decentralized multi-agent system where autonomous agents evolve, collaborate, and govern themselves through peer-to-peer protocols, genetic algorithms, and emergent intelligence.

[![CI](https://github.com/Eccb7/TheManifoldWeb/actions/workflows/ci.yml/badge.svg)](https://github.com/Eccb7/TheManifoldWeb/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## 🌐 Overview

The Manifold Web is an experimental platform for creating and studying artificial life in a decentralized network. Agents are defined by their **genomes** (WASM modules stored on IPFS), evolve through genetic algorithms, communicate via libp2p gossipsub, and collectively govern network parameters through on-chain voting mechanisms.

### Key Features

- 🧬 **Genetic Evolution**: Agents reproduce with crossover and mutation, evolving optimal survival strategies
- 🌍 **Decentralized Network**: Peer-to-peer communication using libp2p (Kademlia DHT + Gossipsub)
- 📦 **Content-Addressed Genomes**: WASM modules stored on IPFS for immutable, verifiable behavior
- 🗳️ **Self-Governance**: Agents vote on protocol upgrades and resource allocation
- 🔬 **Research Tools**: Python simulation lab with Mesa and DEAP for experimentation
- 🎨 **Visualization**: Observer client for real-time network monitoring (3D rendering planned)

## 📁 Repository Structure

```
TheManifoldWeb/
├── crates/                      # Rust workspace
│   ├── manifold-protocol/       # Core data structures and protocol definitions
│   ├── manifold-node/           # Network node with libp2p and simulation engine
│   ├── genesis-sdk/             # SDK for creating and deploying agents
│   ├── observer-client/         # Read-only network monitor
│   └── manifold-archiver/       # Arweave integration for permanent storage
├── contracts/                   # Solidity smart contracts (Foundry)
│   ├── src/                     # Contract source files
│   │   ├── ManifoldGovernanceToken.sol  # ERC-20 governance token
│   │   └── ManifoldDAO.sol              # DAO governance contract
│   ├── test/                    # Contract tests
│   └── script/                  # Deployment scripts
├── python/
│   └── simulation-lab/          # Mesa/DEAP simulations and experiments
├── docs/                        # Documentation
│   ├── ARCHITECTURE.md          # System architecture overview
│   └── CONTRIBUTING.md          # Contribution guidelines
├── .github/workflows/           # CI/CD pipelines
└── README.md                    # This file
```

## 🚀 Quick Start

### Prerequisites

- **Rust** 1.76 or later ([install](https://rustup.rs/))
- **Python** 3.10 or later
- **IPFS** daemon ([install](https://docs.ipfs.tech/install/))
- **Arweave Wallet** (optional, for permanent archival) - [get wallet](https://arweave.app)
- **Foundry** (optional, for smart contract development) - [install](https://book.getfoundry.sh/getting-started/installation)
- **Git**

### Build the Project

```bash
# Clone the repository
git clone https://github.com/Eccb7/TheManifoldWeb.git
cd TheManifoldWeb

# Build all Rust crates
cargo build --workspace

# Install Python dependencies
pip install -r python/simulation-lab/requirements.txt

# Build smart contracts (optional)
cd contracts
forge install
forge build
cd ..
```

### Run Tests

```bash
# Rust tests
cargo test --workspace

# Python tests
cd python/simulation-lab
pytest

# Smart contract tests (requires Foundry)
cd contracts
forge test
```

## 🎮 Usage

### 1. Start IPFS Daemon

IPFS is required for storing and retrieving agent genomes:

```bash
# Initialize IPFS (first time only)
ipfs init

# Start the daemon
ipfs daemon
```

Keep this running in a separate terminal.

### 2. Run a Manifold Node

Start a network node that will simulate agents:

```bash
cargo run -p manifold-node
```

The node will:
- Initialize a libp2p swarm with Kademlia and Gossipsub
- Listen for incoming connections
- Process agent spawning requests
- Execute simulation ticks

### 3. Spawn an Agent (Genesis SDK)

Use the SDK to publish a genome and spawn an agent:

```bash
# Build the genesis-sdk
cargo build -p genesis-sdk

# Run integration tests (requires IPFS daemon)
cargo test -p genesis-sdk -- --ignored
```

**Example: Publishing a genome programmatically**

```rust
use genesis_sdk::{publish_to_ipfs, spawn_agent_via_libp2p};
use manifold_protocol::Genome;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create a genome
    let genome = Genome::new(String::new(), vec![1, 2, 3, 4, 5]);
    
    // Publish to IPFS
    let cid = publish_to_ipfs(&genome, "http://127.0.0.1:5001").await?;
    println!("Published genome with CID: {}", cid);
    
    // Spawn on a node (TODO: implement full libp2p client)
    // let agent_id = spawn_agent_via_libp2p(
    //     "/ip4/127.0.0.1/tcp/12345",
    //     &cid,
    //     1000
    // ).await?;
    
    Ok(())
}
```

### 4. Monitor Network Activity

Run the observer client to watch agent actions in real-time:

```bash
cargo run -p observer-client
```

The observer subscribes to the `manifold-actions` gossipsub topic and displays:
- Agent movements
- Resource consumption
- Replication events
- Governance proposals and votes

### 5. Run Python Simulations

Experiment with agent behavior and evolution in the simulation lab:

```bash
cd python/simulation-lab

# Run the Mesa-based agent simulation
python demo.py

# Run the genetic algorithm demo
python ga_demo.py
```

**Expected output:**

```
🌐 The Manifold Web - Agent Simulation
============================================================
Initializing with 10 agents...
Running simulation for 50 steps...

Step 10: 9 agents alive
Step 20: 7 agents alive
...
```

## 🏗️ Architecture

### Core Components

#### 1. **manifold-protocol** 📚

Defines shared data structures:
- `Genome`: Content-addressed WASM module with evolvable parameters
- `Agent`: Autonomous entity with energy, position, and genome
- `Resource`: Consumable items in the environment
- `Action`: Agent behaviors (move, consume, replicate, vote)
- `Proposal`: Governance mechanism for protocol changes

#### 2. **manifold-node** 🖥️

The main network node implementation:
- **libp2p Networking**: TCP transport with Noise encryption and Yamux multiplexing
- **Custom NetworkBehaviour**: Combines Kademlia (DHT) and Gossipsub (pubsub)
- **Simulation Engine**: Processes agent ticks, applies genome logic
- **Genetic Algorithm**: Single-point crossover and bit-flip mutation for offspring generation
- **Spawn Protocol**: Custom `/manifold/spawn/1.0.0` request/response handler

#### 3. **genesis-sdk** 🛠️

SDK for agent creation:
- **IPFS Integration**: Publish genomes to content-addressed storage
- **Spawn Utilities**: Send spawn requests to network nodes
- **Validation**: Genome format and CID verification

#### 4. **observer-client** 👁️

Read-only monitoring tool with latency compensation:
- Subscribes to gossipsub topics
- Decodes and displays agent actions
- **Dead Reckoning**: Predictive position interpolation using kinematic equations
  - Smooth agent movement visualization between network updates
  - Blends predicted positions with authoritative state
  - Configurable error thresholds for correction strength
- **TODO**: 3D visualization with wgpu/rend3

#### 5. **manifold-archiver** 💾

Permanent archival layer for Arweave:
- **Genesis Block Creation**: Immutable initial simulation state
- **Checkpoint Archival**: Long-term storage of consensus snapshots
- **CLI Tool**: Command-line interface for uploading genesis blocks
- One-time payment model ensures data persists indefinitely

#### 6. **simulation-lab** (Python) 🧪

Research and experimentation environment:
- **Mesa**: Grid-based multi-agent simulation
- **DEAP**: Evolutionary computation toolkit
- Genetic algorithm demos matching Rust implementation

#### 7. **Smart Contracts** (Solidity) 🏛️

On-chain governance layer built with Foundry:
- **ManifoldGovernanceToken**: ERC-20 token with voting extensions
  - Token-weighted voting power (1 token = 1 vote)
  - Reputation tracking for node operators
  - Agent contribution metrics
  - Max supply: 100M MGT
- **ManifoldDAO**: Decentralized governance contract
  - Proposal creation (requires 10,000 MGT minimum)
  - Voting period: ~7 days (50,400 blocks)
  - Time-locked execution (2 day delay)
  - 10% quorum requirement
  - State machine: Pending → Active → Succeeded → Queued → Executed

See `contracts/README.md` for deployment and usage instructions.

### Network Protocol

```
┌─────────────────────────────────────────────────────────────┐
│                      The Manifold Web                        │
│                                                              │
│  ┌──────────────┐         ┌──────────────┐                 │
│  │   Node A     │◄───────►│   Node B     │                 │
│  │              │  libp2p  │              │                 │
│  │ ┌──────────┐ │         │ ┌──────────┐ │                 │
│  │ │ Agent 1  │ │         │ │ Agent 2  │ │                 │
│  │ │ Agent 3  │ │         │ │ Agent 4  │ │                 │
│  │ └──────────┘ │         │ └──────────┘ │                 │
│  └──────────────┘         └──────────────┘                 │
│         │                         │                         │
│         │    Gossipsub Topics     │                         │
│         │   - manifold-actions    │                         │
│         │   - governance-votes    │                         │
│         └─────────────────────────┘                         │
│                     │                                        │
│                     ▼                                        │
│              ┌──────────────┐                               │
│              │   Observer   │                               │
│              │   (Monitor)  │                               │
│              └──────────────┘                               │
│                                                              │
│  External Storage:                                          │
│  ┌──────────────────────────────────────────────┐          │
│  │  IPFS: Content-addressed genome storage      │          │
│  │  CID → WASM Module + Metadata                │          │
│  └──────────────────────────────────────────────┘          │
└─────────────────────────────────────────────────────────────┘
```

### Data Flow

1. **Genome Creation**: Developer writes WASM module defining agent behavior
2. **IPFS Publish**: Genome uploaded to IPFS, returns CID (content identifier)
3. **Agent Spawn**: Genesis SDK sends spawn request with CID to node
4. **Execution**: Node downloads WASM from IPFS, instantiates agent
5. **Simulation**: Agent perceives environment, genome executes, produces actions
6. **Broadcast**: Actions published to gossipsub for network observation
7. **Evolution**: Successful agents replicate, offspring inherit mutated genomes

## 🧬 Genetic Evolution

### Algorithm

The manifold uses a simple but effective genetic algorithm:

```rust
// Single-point crossover
let crossover_point = random(0..min(parent_a.len(), parent_b.len()));
let offspring = parent_a[..crossover_point] + parent_b[crossover_point..];

// Bit-flip mutation
for byte in offspring {
    if random() < mutation_rate {
        let bit = random(0..8);
        byte ^= 1 << bit;
    }
}
```

### Fitness Metrics

Agents are implicitly selected by:
- **Survival time**: Agents with zero energy die
- **Resource collection**: Successful foraging extends lifespan
- **Replication success**: Energy threshold required for reproduction

**TODO**: Implement explicit fitness functions and multi-objective optimization.

## 🗳️ Governance

Agents can submit and vote on proposals:

```rust
pub enum ProposalType {
    ParameterChange { key: String, value: String },
    ResourceAllocation { amount: u64, recipient: String },
    ProtocolUpgrade { version: String, cid: String },
}
```

**Voting mechanisms** (planned):
- Quadratic voting (cost = votes²)
- Conviction voting (time-weighted preferences)
- Identity-weighted participation

**TODO**: Implement on-chain anchoring for governance history.

## 🎯 Latency Compensation

The observer client implements **dead reckoning** for smooth agent visualization:

### Kinematic Prediction

Position and velocity are predicted using physics equations:

```rust
// Predict position with velocity and acceleration
predicted_position = p₀ + v₀·Δt + ½·a·Δt²

// Update velocity
predicted_velocity = v₀ + a·Δt
```

### State Reconciliation

When authoritative updates arrive from the network:

1. **Calculate prediction error**: Distance between predicted and actual position
2. **Blend positions**: Interpolate to avoid jarring corrections ("rubber-banding")
   - `new_position = lerp(predicted, authoritative, blend_factor)`
   - Default blend factor: 0.3 (30% toward authoritative)
3. **Force correction**: If error exceeds threshold (10 units), snap to authoritative

### Smoothing

Additional exponential smoothing for display:

```rust
display_position = lerp(predicted, authoritative, smoothing_alpha)
```

This provides:
- ✅ **Smooth movement** between network updates (compensates for 100-200ms latency)
- ✅ **Accurate positioning** when authoritative state arrives
- ✅ **Minimal rubber-banding** via progressive correction
- ✅ **Configurable parameters** for different network conditions

**TODO**: Implement projective velocity blending for smoother direction changes.

## 🔒 Security

### Current Implementation

- ✅ libp2p Noise encryption for transport security
- ✅ Signed gossipsub messages for authenticity
- ✅ Content-addressed genomes (tamper-proof via IPFS)

### Planned Enhancements

- 🔲 WASM sandboxing with resource limits (CPU, memory)
- 🔲 Zero-knowledge proofs for private agent data
- 🔲 Reputation system for spam prevention
- 🔲 Formal verification of critical protocol logic

## 📊 Performance

**Current benchmarks** (on development hardware):

- Simulation tick rate: ~100ms (10 ticks/second)
- Agent spawn latency: ~50ms (without WASM execution)
- Network message propagation: ~100ms (local network)

**TODO**: Add comprehensive benchmarks and profiling.

## 🛣️ Roadmap

### Phase 1: Foundation ✅ (Current)
- [x] Core protocol definitions
- [x] libp2p networking with Kademlia + Gossipsub
- [x] Basic simulation engine
- [x] Genetic algorithm (crossover, mutation)
- [x] Python simulation lab
- [x] CI/CD pipeline

### Phase 2: Execution 🚧 (In Progress)
- [ ] WASM runtime integration (wasmtime/wasmer)
- [ ] IPFS genome fetching and caching
- [ ] Full libp2p spawn protocol
- [ ] Resource distribution logic
- [ ] Energy balance mechanics

### Phase 3: Visualization 📅 (Planned)
- [ ] 3D rendering with wgpu + rend3
- [ ] Real-time agent position tracking
- [ ] Network topology visualization
- [ ] Web-based dashboard

### Phase 4: Governance 📅 (Planned)
- [ ] On-chain proposal submission
- [ ] Quadratic and conviction voting
- [ ] Reputation and identity system
- [ ] Protocol upgrade mechanism

### Phase 5: Scale 📅 (Future)
- [ ] Multi-node network with discovery
- [ ] Cross-shard agent migration
- [ ] Distributed storage with redundancy
- [ ] Performance optimizations

## 🤝 Contributing

We welcome contributions! Please see [CONTRIBUTING.md](docs/CONTRIBUTING.md) for guidelines.

### Development Workflow

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/amazing-feature`
3. Make your changes
4. Run tests: `cargo test --workspace && pytest python/simulation-lab`
5. Format code: `cargo fmt --all`
6. Commit: `git commit -m 'Add amazing feature'`
7. Push: `git push origin feature/amazing-feature`
8. Open a Pull Request

### Areas for Contribution

- 🐛 Bug fixes and testing
- 📚 Documentation improvements
- 🎨 UI/UX for observer client
- 🧬 New genome templates and behaviors
- 🔬 Simulation scenarios and experiments
- ⚡ Performance optimizations

## 📖 Documentation

- [Architecture Overview](docs/ARCHITECTURE.md)
- [Contributing Guide](docs/CONTRIBUTING.md)
- [API Documentation](https://docs.rs/manifold-protocol) (TODO: publish)

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- **libp2p** for decentralized networking primitives
- **IPFS** for content-addressed storage
- **Mesa** and **DEAP** for multi-agent simulation research
- The broader Rust and Python communities

## 📞 Contact

- **GitHub**: [@Eccb7](https://github.com/Eccb7)
- **Repository**: [TheManifoldWeb](https://github.com/Eccb7/TheManifoldWeb)
- **Issues**: [Report a bug](https://github.com/Eccb7/TheManifoldWeb/issues)

---

*Built with curiosity, for the future of emergent intelligence.*
