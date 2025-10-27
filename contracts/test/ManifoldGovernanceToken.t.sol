// SPDX-License-Identifier: MIT
pragma solidity ^0.8.23;

import {Test, console2} from "forge-std/Test.sol";
import {ManifoldGovernanceToken} from "../src/ManifoldGovernanceToken.sol";

contract ManifoldGovernanceTokenTest is Test {
    ManifoldGovernanceToken public token;
    address public owner;
    address public user1;
    address public user2;
    
    function setUp() public {
        owner = address(this);
        user1 = makeAddr("user1");
        user2 = makeAddr("user2");
        
        token = new ManifoldGovernanceToken(owner);
    }
    
    function test_InitialState() public {
        assertEq(token.name(), "Manifold Governance Token");
        assertEq(token.symbol(), "MGT");
        assertEq(token.totalSupply(), token.MAX_SUPPLY() / 10);
        assertEq(token.balanceOf(owner), token.MAX_SUPPLY() / 10);
    }
    
    function test_MintReward() public {
        uint256 amount = 1000 * 10 ** 18;
        
        token.mintReward(user1, amount, "Node participation");
        
        assertEq(token.balanceOf(user1), amount);
        assertEq(token.totalMinted(), token.MAX_SUPPLY() / 10 + amount);
    }
    
    function test_MintReward_ExceedsMaxSupply() public {
        uint256 exceedingAmount = token.MAX_SUPPLY();
        
        vm.expectRevert("Exceeds max supply");
        token.mintReward(user1, exceedingAmount, "Too much");
    }
    
    function test_MintReward_OnlyOwner() public {
        vm.prank(user1);
        vm.expectRevert();
        token.mintReward(user2, 1000, "Unauthorized");
    }
    
    function test_UpdateNodeReputation() public {
        token.updateNodeReputation(user1, 100);
        assertEq(token.nodeReputation(user1), 100);
        
        token.updateNodeReputation(user1, 200);
        assertEq(token.nodeReputation(user1), 200);
    }
    
    function test_RecordAgentContribution() public {
        bytes32 agentId = keccak256("agent1");
        
        token.recordAgentContribution(agentId, 50);
        assertEq(token.agentContribution(agentId), 50);
        
        token.recordAgentContribution(agentId, 30);
        assertEq(token.agentContribution(agentId), 80);
    }
    
    function test_GetVotingPower() public {
        uint256 amount = 5000 * 10 ** 18;
        token.mintReward(user1, amount, "Test");
        
        assertEq(token.getVotingPower(user1), amount);
    }
    
    function test_Transfer() public {
        uint256 amount = 1000 * 10 ** 18;
        token.mintReward(user1, amount, "Test");
        
        vm.prank(user1);
        token.transfer(user2, 500 * 10 ** 18);
        
        assertEq(token.balanceOf(user1), 500 * 10 ** 18);
        assertEq(token.balanceOf(user2), 500 * 10 ** 18);
    }
}
