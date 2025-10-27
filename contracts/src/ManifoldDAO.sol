// SPDX-License-Identifier: MIT
pragma solidity ^0.8.23;

import {ManifoldGovernanceToken} from "./ManifoldGovernanceToken.sol";

/// @title ManifoldDAO
/// @notice Decentralized Autonomous Organization for governing The Manifold Web
/// @dev Implements proposal creation, voting, and execution for protocol governance
///
/// Governance Features:
/// - Token-weighted voting (1 token = 1 vote)
/// - Proposal types: Parameter changes, resource allocation, protocol upgrades
/// - Quadratic voting support (planned future upgrade)
/// - Time-locked execution for security
contract ManifoldDAO {
    /// @notice Governance token contract
    ManifoldGovernanceToken public immutable governanceToken;
    
    /// @notice Minimum tokens required to create a proposal
    uint256 public proposalThreshold;
    
    /// @notice Voting period duration in blocks
    uint256 public votingPeriod;
    
    /// @notice Time delay before execution after proposal passes
    uint256 public executionDelay;
    
    /// @notice Quorum percentage required for proposal to pass (out of 100)
    uint256 public quorumPercentage;
    
    /// @notice Counter for proposal IDs
    uint256 public proposalCount;
    
    /// @notice Enum for proposal states
    enum ProposalState {
        Pending,
        Active,
        Defeated,
        Succeeded,
        Queued,
        Executed,
        Canceled
    }
    
    /// @notice Enum for proposal types
    enum ProposalType {
        ParameterChange,
        ResourceAllocation,
        ProtocolUpgrade,
        Custom
    }
    
    /// @notice Struct representing a governance proposal
    struct Proposal {
        uint256 id;
        address proposer;
        ProposalType proposalType;
        string description;
        uint256 startBlock;
        uint256 endBlock;
        uint256 executionTime;
        uint256 forVotes;
        uint256 againstVotes;
        uint256 abstainVotes;
        bool executed;
        bool canceled;
        bytes executionData;
    }
    
    /// @notice Mapping from proposal ID to Proposal
    mapping(uint256 => Proposal) public proposals;
    
    /// @notice Mapping from proposal ID to voter address to vote cast
    mapping(uint256 => mapping(address => bool)) public hasVoted;
    
    /// @notice Mapping from proposal ID to voter address to vote weight
    mapping(uint256 => mapping(address => uint256)) public voteWeight;
    
    /// @notice Events
    event ProposalCreated(
        uint256 indexed proposalId,
        address indexed proposer,
        ProposalType proposalType,
        string description,
        uint256 startBlock,
        uint256 endBlock
    );
    
    event VoteCast(
        uint256 indexed proposalId,
        address indexed voter,
        bool support,
        uint256 weight
    );
    
    event ProposalQueued(uint256 indexed proposalId, uint256 executionTime);
    
    event ProposalExecuted(uint256 indexed proposalId);
    
    event ProposalCanceled(uint256 indexed proposalId);
    
    event ParametersUpdated(
        uint256 proposalThreshold,
        uint256 votingPeriod,
        uint256 executionDelay,
        uint256 quorumPercentage
    );
    
    /// @notice Constructor initializes DAO with governance token and parameters
    /// @param _governanceToken Address of the governance token contract
    /// @param _proposalThreshold Minimum tokens to create a proposal
    /// @param _votingPeriod Voting period in blocks
    /// @param _executionDelay Delay before execution in seconds
    /// @param _quorumPercentage Quorum percentage (0-100)
    constructor(
        address _governanceToken,
        uint256 _proposalThreshold,
        uint256 _votingPeriod,
        uint256 _executionDelay,
        uint256 _quorumPercentage
    ) {
        require(_governanceToken != address(0), "Invalid token address");
        require(_quorumPercentage <= 100, "Invalid quorum");
        
        governanceToken = ManifoldGovernanceToken(_governanceToken);
        proposalThreshold = _proposalThreshold;
        votingPeriod = _votingPeriod;
        executionDelay = _executionDelay;
        quorumPercentage = _quorumPercentage;
    }
    
    /// @notice Create a new governance proposal
    /// @param proposalType Type of proposal
    /// @param description Human-readable description
    /// @param executionData Encoded function call data for execution
    /// @return proposalId ID of the created proposal
    function propose(
        ProposalType proposalType,
        string calldata description,
        bytes calldata executionData
    ) external returns (uint256 proposalId) {
        require(
            governanceToken.getVotingPower(msg.sender) >= proposalThreshold,
            "Below proposal threshold"
        );
        
        proposalId = ++proposalCount;
        uint256 startBlock = block.number + 1;
        uint256 endBlock = startBlock + votingPeriod;
        
        proposals[proposalId] = Proposal({
            id: proposalId,
            proposer: msg.sender,
            proposalType: proposalType,
            description: description,
            startBlock: startBlock,
            endBlock: endBlock,
            executionTime: 0,
            forVotes: 0,
            againstVotes: 0,
            abstainVotes: 0,
            executed: false,
            canceled: false,
            executionData: executionData
        });
        
        emit ProposalCreated(
            proposalId,
            msg.sender,
            proposalType,
            description,
            startBlock,
            endBlock
        );
    }
    
    /// @notice Cast a vote on a proposal
    /// @param proposalId ID of the proposal
    /// @param support True for yes, false for no
    function castVote(uint256 proposalId, bool support) external {
        require(state(proposalId) == ProposalState.Active, "Voting not active");
        require(!hasVoted[proposalId][msg.sender], "Already voted");
        
        uint256 weight = governanceToken.getVotingPower(msg.sender);
        require(weight > 0, "No voting power");
        
        Proposal storage proposal = proposals[proposalId];
        
        if (support) {
            proposal.forVotes += weight;
        } else {
            proposal.againstVotes += weight;
        }
        
        hasVoted[proposalId][msg.sender] = true;
        voteWeight[proposalId][msg.sender] = weight;
        
        emit VoteCast(proposalId, msg.sender, support, weight);
    }
    
    /// @notice Queue a successful proposal for execution
    /// @param proposalId ID of the proposal
    function queue(uint256 proposalId) external {
        require(state(proposalId) == ProposalState.Succeeded, "Not succeeded");
        
        Proposal storage proposal = proposals[proposalId];
        proposal.executionTime = block.timestamp + executionDelay;
        
        emit ProposalQueued(proposalId, proposal.executionTime);
    }
    
    /// @notice Execute a queued proposal
    /// @param proposalId ID of the proposal
    function execute(uint256 proposalId) external {
        require(state(proposalId) == ProposalState.Queued, "Not queued");
        
        Proposal storage proposal = proposals[proposalId];
        require(block.timestamp >= proposal.executionTime, "Execution delayed");
        
        proposal.executed = true;
        
        // TODO: Implement on-chain execution logic
        // This would involve calling external contracts or updating state
        // based on proposal.executionData and proposal.proposalType
        //
        // Examples:
        // - ParameterChange: Update protocol parameters
        // - ResourceAllocation: Transfer tokens or resources
        // - ProtocolUpgrade: Delegate call to new implementation
        //
        // For now, we just mark as executed
        
        emit ProposalExecuted(proposalId);
    }
    
    /// @notice Cancel a proposal (only by proposer before execution)
    /// @param proposalId ID of the proposal
    function cancel(uint256 proposalId) external {
        Proposal storage proposal = proposals[proposalId];
        require(msg.sender == proposal.proposer, "Not proposer");
        require(!proposal.executed, "Already executed");
        require(state(proposalId) != ProposalState.Executed, "Cannot cancel");
        
        proposal.canceled = true;
        
        emit ProposalCanceled(proposalId);
    }
    
    /// @notice Get the current state of a proposal
    /// @param proposalId ID of the proposal
    /// @return Current ProposalState
    function state(uint256 proposalId) public view returns (ProposalState) {
        Proposal storage proposal = proposals[proposalId];
        
        if (proposal.canceled) {
            return ProposalState.Canceled;
        }
        
        if (proposal.executed) {
            return ProposalState.Executed;
        }
        
        if (block.number < proposal.startBlock) {
            return ProposalState.Pending;
        }
        
        if (block.number <= proposal.endBlock) {
            return ProposalState.Active;
        }
        
        // Check if proposal succeeded
        uint256 totalSupply = governanceToken.totalSupply();
        uint256 quorum = (totalSupply * quorumPercentage) / 100;
        
        if (proposal.forVotes <= proposal.againstVotes || proposal.forVotes < quorum) {
            return ProposalState.Defeated;
        }
        
        if (proposal.executionTime == 0) {
            return ProposalState.Succeeded;
        }
        
        return ProposalState.Queued;
    }
    
    /// @notice Update DAO parameters (only callable by DAO itself via proposal)
    /// @param _proposalThreshold New proposal threshold
    /// @param _votingPeriod New voting period
    /// @param _executionDelay New execution delay
    /// @param _quorumPercentage New quorum percentage
    function updateParameters(
        uint256 _proposalThreshold,
        uint256 _votingPeriod,
        uint256 _executionDelay,
        uint256 _quorumPercentage
    ) external {
        require(msg.sender == address(this), "Only DAO can update");
        require(_quorumPercentage <= 100, "Invalid quorum");
        
        proposalThreshold = _proposalThreshold;
        votingPeriod = _votingPeriod;
        executionDelay = _executionDelay;
        quorumPercentage = _quorumPercentage;
        
        emit ParametersUpdated(
            _proposalThreshold,
            _votingPeriod,
            _executionDelay,
            _quorumPercentage
        );
    }
    
    /// @notice Get proposal details
    /// @param proposalId ID of the proposal
    /// @return Full proposal struct
    function getProposal(uint256 proposalId) external view returns (Proposal memory) {
        return proposals[proposalId];
    }
    
    /// @notice Check if an address has voted on a proposal
    /// @param proposalId ID of the proposal
    /// @param voter Address to check
    /// @return True if voted, false otherwise
    function hasVotedOnProposal(uint256 proposalId, address voter) external view returns (bool) {
        return hasVoted[proposalId][voter];
    }
}
