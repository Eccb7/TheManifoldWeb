# The Manifold Web - Architecture

## Overview

The Manifold Web is a decentralized multi-agent system where autonomous agents evolve, collaborate, and govern themselves through peer-to-peer protocols.

## System Components

### 1. manifold-protocol
Core data structures and protocol definitions shared across all components.

### 2. manifold-node
The main network node implementing libp2p networking, agent simulation, and genome evolution.

### 3. genesis-sdk
Software Development Kit for creating and deploying new agents to the network.

### 4. observer-client
Read-only client for monitoring network activity and visualizing agent behavior.

### 5. simulation-lab (Python)
Research and experimentation environment for genetic algorithms and agent behavior modeling.

## Network Architecture

- **Protocol**: libp2p with Kademlia (DHT) and Gossipsub (pubsub)
- **Transport**: TCP with Noise encryption and Yamux multiplexing
- **Storage**: IPFS for content-addressed genome storage
- **Execution**: WASM sandboxing for agent genome execution (planned)

## Data Flow

1. Agents' genomes are stored on IPFS as content-addressed WASM modules
2. Nodes execute genomes in isolated WASM runtimes
3. Agent actions are broadcast via gossipsub
4. Network state is discovered via Kademlia DHT
5. Governance proposals are voted on by agent identities

## Future Enhancements

- Zero-knowledge proof integration for privacy
- On-chain governance anchoring
- Advanced 3D visualization with wgpu/rend3
- Cross-chain bridge protocols
