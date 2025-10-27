// SPDX-License-Identifier: MIT
pragma solidity ^0.8.23;

import {ERC20} from "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import {ERC20Permit} from "@openzeppelin/contracts/token/ERC20/extensions/ERC20Permit.sol";
import {ERC20Votes} from "@openzeppelin/contracts/token/ERC20/extensions/ERC20Votes.sol";
import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";

/// @title ManifoldGovernanceToken
/// @notice ERC-20 governance token for The Manifold Web DAO
/// @dev Extends ERC20Votes for on-chain governance and ERC20Permit for gasless approvals
/// 
/// Token holders can:
/// - Vote on protocol upgrades
/// - Propose parameter changes
/// - Allocate resources to agents
/// 
/// Tokens are earned by:
/// - Running simulation nodes (proof-of-stake)
/// - Agent success metrics (energy collected, survival time)
/// - Contributing to the network (IPFS hosting, consensus participation)
contract ManifoldGovernanceToken is ERC20, ERC20Permit, ERC20Votes, Ownable {
    /// @notice Maximum supply of governance tokens (100 million)
    uint256 public constant MAX_SUPPLY = 100_000_000 * 10 ** 18;
    
    /// @notice Tokens minted to date
    uint256 public totalMinted;
    
    /// @notice Mapping of node addresses to their reputation scores
    mapping(address => uint256) public nodeReputation;
    
    /// @notice Mapping of agent IDs to their contribution scores
    mapping(bytes32 => uint256) public agentContribution;
    
    /// @notice Event emitted when tokens are minted as rewards
    event RewardMinted(address indexed recipient, uint256 amount, string reason);
    
    /// @notice Event emitted when node reputation is updated
    event ReputationUpdated(address indexed node, uint256 oldReputation, uint256 newReputation);
    
    /// @notice Event emitted when agent contribution is recorded
    event ContributionRecorded(bytes32 indexed agentId, uint256 contribution);
    
    /// @notice Constructor initializes token with name, symbol, and mints initial supply
    /// @param initialOwner Address that will own the contract and receive initial tokens
    constructor(address initialOwner) 
        ERC20("Manifold Governance Token", "MGT") 
        ERC20Permit("Manifold Governance Token")
        Ownable(initialOwner)
    {
        // Mint 10% of max supply to initial owner for bootstrapping
        uint256 initialSupply = MAX_SUPPLY / 10;
        _mint(initialOwner, initialSupply);
        totalMinted = initialSupply;
    }
    
    /// @notice Mint tokens as rewards for network participants
    /// @dev Only callable by owner (DAO contract after governance is established)
    /// @param recipient Address to receive the tokens
    /// @param amount Amount of tokens to mint
    /// @param reason Human-readable reason for minting
    function mintReward(address recipient, uint256 amount, string calldata reason) 
        external 
        onlyOwner 
    {
        require(totalMinted + amount <= MAX_SUPPLY, "Exceeds max supply");
        require(recipient != address(0), "Invalid recipient");
        require(amount > 0, "Amount must be positive");
        
        _mint(recipient, amount);
        totalMinted += amount;
        
        emit RewardMinted(recipient, amount, reason);
    }
    
    /// @notice Update reputation score for a node
    /// @dev Higher reputation may grant more voting power in future upgrades
    /// @param node Address of the node
    /// @param reputation New reputation score
    function updateNodeReputation(address node, uint256 reputation) 
        external 
        onlyOwner 
    {
        uint256 oldReputation = nodeReputation[node];
        nodeReputation[node] = reputation;
        
        emit ReputationUpdated(node, oldReputation, reputation);
    }
    
    /// @notice Record contribution score for an agent
    /// @dev Agents that perform well may earn tokens for their owners
    /// @param agentId Unique identifier for the agent
    /// @param contribution Contribution score (energy collected, survival time, etc.)
    function recordAgentContribution(bytes32 agentId, uint256 contribution) 
        external 
        onlyOwner 
    {
        agentContribution[agentId] += contribution;
        
        emit ContributionRecorded(agentId, contribution);
    }
    
    /// @notice Get the current voting power of an account
    /// @param account Address to check
    /// @return Voting power (token balance)
    function getVotingPower(address account) external view returns (uint256) {
        return balanceOf(account);
    }
    
    // Override required by Solidity for multiple inheritance
    function _update(address from, address to, uint256 amount)
        internal
        override(ERC20, ERC20Votes)
    {
        super._update(from, to, amount);
    }

    function nonces(address owner)
        public
        view
        override(ERC20Permit, Nonces)
        returns (uint256)
    {
        return super.nonces(owner);
    }
}
