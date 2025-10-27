# Manifold Web Governance Contracts

Smart contracts for decentralized governance of The Manifold Web protocol.

## Overview

This directory contains Solidity smart contracts built with [Foundry](https://book.getfoundry.sh/) that enable on-chain governance for The Manifold Web.

### Contracts

#### ManifoldGovernanceToken (MGT)

ERC-20 governance token with voting extensions.

**Features:**
- **Max Supply**: 100 million MGT tokens
- **ERC20Votes**: Built-in voting power tracking
- **ERC20Permit**: Gasless approvals via signatures
- **Reputation System**: Node reputation tracking
- **Agent Contributions**: Record agent performance metrics

**Token Distribution:**
- 10% initial supply to deployer for bootstrapping
- 90% minted as rewards for:
  - Running simulation nodes
  - Agent success (energy collected, survival time)
  - Network contributions (IPFS hosting, consensus participation)

#### ManifoldDAO

Decentralized Autonomous Organization for protocol governance.

**Features:**
- **Token-Weighted Voting**: 1 token = 1 vote
- **Proposal Types**:
  - Parameter changes (sector size, energy decay, mutation rate)
  - Resource allocation (token grants, funding)
  - Protocol upgrades (new features, bug fixes)
  - Custom proposals (arbitrary on-chain actions)
- **Time-Locked Execution**: Security delay before execution
- **Quorum Requirements**: Minimum participation threshold

**Governance Flow:**
1. Create proposal (requires minimum token threshold)
2. Voting period (default: 7 days)
3. Queue successful proposal
4. Execution delay (default: 2 days)
5. Execute proposal on-chain

## Installation

### Prerequisites

- [Foundry](https://book.getfoundry.sh/getting-started/installation)
- Git

### Setup

```bash
# Install Foundry (if not already installed)
curl -L https://foundry.paradigm.xyz | bash
foundryup

# Navigate to contracts directory
cd contracts

# Install dependencies (OpenZeppelin contracts)
forge install OpenZeppelin/openzeppelin-contracts --no-commit
```

## Usage

### Build

```bash
forge build
```

### Test

```bash
# Run all tests
forge test

# Run tests with verbosity
forge test -vvv

# Run specific test
forge test --match-test test_CreateProposal

# Generate gas report
forge test --gas-report
```

### Deploy

#### Local (Anvil)

```bash
# Start local node
anvil

# Deploy contracts (in another terminal)
forge script script/Deploy.s.sol:DeployScript --rpc-url http://localhost:8545 --broadcast
```

#### Testnet (Sepolia)

```bash
# Set environment variables
export PRIVATE_KEY=your_private_key
export SEPOLIA_RPC_URL=your_sepolia_rpc_url
export ETHERSCAN_API_KEY=your_etherscan_api_key

# Deploy and verify
forge script script/Deploy.s.sol:DeployScript \
    --rpc-url $SEPOLIA_RPC_URL \
    --broadcast \
    --verify
```

#### Mainnet

```bash
# Set environment variables
export PRIVATE_KEY=your_private_key
export MAINNET_RPC_URL=your_mainnet_rpc_url
export ETHERSCAN_API_KEY=your_etherscan_api_key

# Deploy and verify
forge script script/Deploy.s.sol:DeployScript \
    --rpc-url $MAINNET_RPC_URL \
    --broadcast \
    --verify
```

## Contract Addresses

### Sepolia Testnet
- ManifoldGovernanceToken: `TBD`
- ManifoldDAO: `TBD`

### Mainnet
- ManifoldGovernanceToken: `TBD`
- ManifoldDAO: `TBD`

## Governance Parameters

| Parameter | Value | Description |
|-----------|-------|-------------|
| Proposal Threshold | 10,000 MGT | Minimum tokens to create proposal |
| Voting Period | 50,400 blocks (~7 days) | How long voting is open |
| Execution Delay | 2 days | Time delay before execution |
| Quorum | 10% | Minimum participation required |

## Example Governance Proposal

```solidity
// 1. Create proposal
uint256 proposalId = dao.propose(
    ManifoldDAO.ProposalType.ParameterChange,
    "Increase sector size from 100 to 200 units",
    abi.encodeWithSignature("updateSectorSize(uint256)", 200)
);

// 2. Vote on proposal
dao.castVote(proposalId, true); // true = for, false = against

// 3. Queue proposal (after voting ends and proposal succeeds)
dao.queue(proposalId);

// 4. Execute proposal (after execution delay)
dao.execute(proposalId);
```

## Integration with Manifold Node

The governance contracts are designed to integrate with the Rust-based manifold-node:

1. **Node Operators**: Run simulation nodes and earn MGT tokens
2. **Token Holders**: Vote on protocol parameters and upgrades
3. **Proposals**: Submitted on-chain, executed by DAO contract
4. **State Sync**: Consensus layer verifies on-chain governance decisions

### Off-Chain → On-Chain Flow

```
┌──────────────────┐
│  Manifold Node   │
│  (Off-Chain)     │
└─────────┬────────┘
          │
          │ 1. Node earns tokens
          │    for participation
          ▼
┌──────────────────┐
│ Governance Token │
│   (On-Chain)     │
└─────────┬────────┘
          │
          │ 2. Token holders
          │    create proposals
          ▼
┌──────────────────┐
│  ManifoldDAO     │
│   (On-Chain)     │
└─────────┬────────┘
          │
          │ 3. Votes cast,
          │    proposals executed
          ▼
┌──────────────────┐
│ Protocol Updates │
│  (Off-Chain)     │
└──────────────────┘
```

## Security Considerations

- **Time Delays**: Execution delay allows community to react to malicious proposals
- **Quorum Requirements**: Prevents minority attacks
- **OpenZeppelin**: Uses battle-tested contract libraries
- **Audits**: Contracts should be audited before mainnet deployment
- **Multi-Sig**: Consider deploying with Gnosis Safe as initial owner

## Development

### Project Structure

```
contracts/
├── src/
│   ├── ManifoldGovernanceToken.sol  # ERC-20 governance token
│   └── ManifoldDAO.sol              # DAO governance contract
├── test/
│   ├── ManifoldGovernanceToken.t.sol
│   └── ManifoldDAO.t.sol
├── script/
│   └── Deploy.s.sol                 # Deployment script
├── foundry.toml                     # Foundry configuration
└── README.md                        # This file
```

### Testing Best Practices

- Write comprehensive unit tests for all functions
- Test edge cases and failure modes
- Use fuzzing for parameter validation
- Generate gas reports to optimize costs
- Simulate governance workflows end-to-end

### Code Style

- Follow [Solidity Style Guide](https://docs.soliditylang.org/en/latest/style-guide.html)
- Use NatSpec comments for all public functions
- Keep functions small and focused
- Emit events for all state changes

## Future Enhancements

- [ ] **Quadratic Voting**: Implement quadratic cost for votes (cost = votes²)
- [ ] **Conviction Voting**: Time-weighted voting for long-term alignment
- [ ] **Delegation**: Allow token holders to delegate voting power
- [ ] **Multi-Sig Integration**: Support for Gnosis Safe execution
- [ ] **On-Chain Execution**: Implement actual execution logic (currently placeholder)
- [ ] **Governance Analytics**: Track participation and proposal success rates
- [ ] **Token Vesting**: Gradual unlock for team/investor allocations

## Resources

- [Foundry Book](https://book.getfoundry.sh/)
- [OpenZeppelin Contracts](https://docs.openzeppelin.com/contracts/)
- [Solidity Documentation](https://docs.soliditylang.org/)
- [The Manifold Web Docs](../docs/ARCHITECTURE.md)

## License

MIT License - see [LICENSE](../LICENSE) for details.
