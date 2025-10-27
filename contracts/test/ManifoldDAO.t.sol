// SPDX-License-Identifier: MIT
pragma solidity ^0.8.23;

import {Test, console2} from "forge-std/Test.sol";
import {ManifoldGovernanceToken} from "../src/ManifoldGovernanceToken.sol";
import {ManifoldDAO} from "../src/ManifoldDAO.sol";

contract ManifoldDAOTest is Test {
    ManifoldGovernanceToken public token;
    ManifoldDAO public dao;
    
    address public owner;
    address public proposer;
    address public voter1;
    address public voter2;
    address public voter3;
    
    uint256 constant PROPOSAL_THRESHOLD = 1000 * 10 ** 18;
    uint256 constant VOTING_PERIOD = 100; // blocks
    uint256 constant EXECUTION_DELAY = 2 days;
    uint256 constant QUORUM_PERCENTAGE = 10; // 10%
    
    function setUp() public {
        owner = address(this);
        proposer = makeAddr("proposer");
        voter1 = makeAddr("voter1");
        voter2 = makeAddr("voter2");
        voter3 = makeAddr("voter3");
        
        // Deploy token and DAO
        token = new ManifoldGovernanceToken(owner);
        dao = new ManifoldDAO(
            address(token),
            PROPOSAL_THRESHOLD,
            VOTING_PERIOD,
            EXECUTION_DELAY,
            QUORUM_PERCENTAGE
        );
        
        // Distribute tokens for testing
        token.mintReward(proposer, 2000 * 10 ** 18, "Test setup");
        token.mintReward(voter1, 5000 * 10 ** 18, "Test setup");
        token.mintReward(voter2, 3000 * 10 ** 18, "Test setup");
        token.mintReward(voter3, 2000 * 10 ** 18, "Test setup");
    }
    
    function test_InitialState() public {
        assertEq(address(dao.governanceToken()), address(token));
        assertEq(dao.proposalThreshold(), PROPOSAL_THRESHOLD);
        assertEq(dao.votingPeriod(), VOTING_PERIOD);
        assertEq(dao.executionDelay(), EXECUTION_DELAY);
        assertEq(dao.quorumPercentage(), QUORUM_PERCENTAGE);
        assertEq(dao.proposalCount(), 0);
    }
    
    function test_CreateProposal() public {
        vm.prank(proposer);
        uint256 proposalId = dao.propose(
            ManifoldDAO.ProposalType.ParameterChange,
            "Increase sector size to 200",
            ""
        );
        
        assertEq(proposalId, 1);
        assertEq(dao.proposalCount(), 1);
        
        ManifoldDAO.Proposal memory proposal = dao.getProposal(proposalId);
        assertEq(proposal.proposer, proposer);
        assertEq(uint(proposal.proposalType), uint(ManifoldDAO.ProposalType.ParameterChange));
        assertEq(proposal.description, "Increase sector size to 200");
    }
    
    function test_CreateProposal_BelowThreshold() public {
        address lowPowerUser = makeAddr("lowPower");
        token.mintReward(lowPowerUser, 500 * 10 ** 18, "Below threshold");
        
        vm.prank(lowPowerUser);
        vm.expectRevert("Below proposal threshold");
        dao.propose(
            ManifoldDAO.ProposalType.ParameterChange,
            "Should fail",
            ""
        );
    }
    
    function test_CastVote() public {
        // Create proposal
        vm.prank(proposer);
        uint256 proposalId = dao.propose(
            ManifoldDAO.ProposalType.ResourceAllocation,
            "Allocate 1000 tokens to research",
            ""
        );
        
        // Advance to voting period
        vm.roll(block.number + 2);
        
        // Cast votes
        vm.prank(voter1);
        dao.castVote(proposalId, true);
        
        vm.prank(voter2);
        dao.castVote(proposalId, true);
        
        vm.prank(voter3);
        dao.castVote(proposalId, false);
        
        // Check vote tracking
        assertTrue(dao.hasVotedOnProposal(proposalId, voter1));
        assertTrue(dao.hasVotedOnProposal(proposalId, voter2));
        assertTrue(dao.hasVotedOnProposal(proposalId, voter3));
        
        ManifoldDAO.Proposal memory proposal = dao.getProposal(proposalId);
        assertEq(proposal.forVotes, 8000 * 10 ** 18); // voter1 + voter2
        assertEq(proposal.againstVotes, 2000 * 10 ** 18); // voter3
    }
    
    function test_CastVote_AlreadyVoted() public {
        vm.prank(proposer);
        uint256 proposalId = dao.propose(
            ManifoldDAO.ProposalType.Custom,
            "Test proposal",
            ""
        );
        
        vm.roll(block.number + 2);
        
        vm.prank(voter1);
        dao.castVote(proposalId, true);
        
        vm.prank(voter1);
        vm.expectRevert("Already voted");
        dao.castVote(proposalId, false);
    }
    
    function test_ProposalState_Active() public {
        vm.prank(proposer);
        uint256 proposalId = dao.propose(
            ManifoldDAO.ProposalType.ParameterChange,
            "Test",
            ""
        );
        
        vm.roll(block.number + 2);
        
        assertEq(uint(dao.state(proposalId)), uint(ManifoldDAO.ProposalState.Active));
    }
    
    function test_ProposalState_Succeeded() public {
        vm.prank(proposer);
        uint256 proposalId = dao.propose(
            ManifoldDAO.ProposalType.ProtocolUpgrade,
            "Upgrade to v2",
            ""
        );
        
        vm.roll(block.number + 2);
        
        // Cast enough votes to pass
        vm.prank(voter1);
        dao.castVote(proposalId, true);
        
        // Advance past voting period
        vm.roll(block.number + VOTING_PERIOD + 1);
        
        assertEq(uint(dao.state(proposalId)), uint(ManifoldDAO.ProposalState.Succeeded));
    }
    
    function test_ProposalState_Defeated() public {
        vm.prank(proposer);
        uint256 proposalId = dao.propose(
            ManifoldDAO.ProposalType.Custom,
            "Will be defeated",
            ""
        );
        
        vm.roll(block.number + 2);
        
        // Cast more against votes
        vm.prank(voter1);
        dao.castVote(proposalId, false);
        
        vm.prank(voter2);
        dao.castVote(proposalId, false);
        
        // Advance past voting period
        vm.roll(block.number + VOTING_PERIOD + 1);
        
        assertEq(uint(dao.state(proposalId)), uint(ManifoldDAO.ProposalState.Defeated));
    }
    
    function test_QueueProposal() public {
        vm.prank(proposer);
        uint256 proposalId = dao.propose(
            ManifoldDAO.ProposalType.ResourceAllocation,
            "Test queue",
            ""
        );
        
        vm.roll(block.number + 2);
        
        vm.prank(voter1);
        dao.castVote(proposalId, true);
        
        vm.roll(block.number + VOTING_PERIOD + 1);
        
        dao.queue(proposalId);
        
        assertEq(uint(dao.state(proposalId)), uint(ManifoldDAO.ProposalState.Queued));
    }
    
    function test_ExecuteProposal() public {
        vm.prank(proposer);
        uint256 proposalId = dao.propose(
            ManifoldDAO.ProposalType.ParameterChange,
            "Test execute",
            ""
        );
        
        vm.roll(block.number + 2);
        
        vm.prank(voter1);
        dao.castVote(proposalId, true);
        
        vm.roll(block.number + VOTING_PERIOD + 1);
        
        dao.queue(proposalId);
        
        // Advance time past execution delay
        vm.warp(block.timestamp + EXECUTION_DELAY + 1);
        
        dao.execute(proposalId);
        
        assertEq(uint(dao.state(proposalId)), uint(ManifoldDAO.ProposalState.Executed));
    }
    
    function test_CancelProposal() public {
        vm.prank(proposer);
        uint256 proposalId = dao.propose(
            ManifoldDAO.ProposalType.Custom,
            "Will be canceled",
            ""
        );
        
        vm.prank(proposer);
        dao.cancel(proposalId);
        
        assertEq(uint(dao.state(proposalId)), uint(ManifoldDAO.ProposalState.Canceled));
    }
    
    function test_CancelProposal_NotProposer() public {
        vm.prank(proposer);
        uint256 proposalId = dao.propose(
            ManifoldDAO.ProposalType.Custom,
            "Test",
            ""
        );
        
        vm.prank(voter1);
        vm.expectRevert("Not proposer");
        dao.cancel(proposalId);
    }
}
