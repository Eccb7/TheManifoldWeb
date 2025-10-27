// SPDX-License-Identifier: MIT
pragma solidity ^0.8.23;

import {Script, console2} from "forge-std/Script.sol";
import {ManifoldGovernanceToken} from "../src/ManifoldGovernanceToken.sol";
import {ManifoldDAO} from "../src/ManifoldDAO.sol";

/// @title Deploy Script for Manifold Governance Contracts
/// @notice Deploys ManifoldGovernanceToken and ManifoldDAO contracts
/// 
/// Usage:
/// forge script script/Deploy.s.sol:DeployScript --rpc-url <RPC_URL> --broadcast --verify
contract DeployScript is Script {
    // Deployment parameters
    uint256 constant PROPOSAL_THRESHOLD = 10_000 * 10 ** 18; // 10,000 tokens
    uint256 constant VOTING_PERIOD = 50400; // ~7 days (assuming 12s blocks)
    uint256 constant EXECUTION_DELAY = 2 days;
    uint256 constant QUORUM_PERCENTAGE = 10; // 10%
    
    function run() public {
        // Get deployer from environment
        uint256 deployerPrivateKey = vm.envUint("PRIVATE_KEY");
        address deployer = vm.addr(deployerPrivateKey);
        
        console2.log("Deploying from:", deployer);
        console2.log("Deployer balance:", deployer.balance);
        
        vm.startBroadcast(deployerPrivateKey);
        
        // Deploy Governance Token
        console2.log("\nDeploying ManifoldGovernanceToken...");
        ManifoldGovernanceToken token = new ManifoldGovernanceToken(deployer);
        console2.log("ManifoldGovernanceToken deployed at:", address(token));
        console2.log("Initial supply:", token.totalSupply() / 10 ** 18, "MGT");
        
        // Deploy DAO
        console2.log("\nDeploying ManifoldDAO...");
        ManifoldDAO dao = new ManifoldDAO(
            address(token),
            PROPOSAL_THRESHOLD,
            VOTING_PERIOD,
            EXECUTION_DELAY,
            QUORUM_PERCENTAGE
        );
        console2.log("ManifoldDAO deployed at:", address(dao));
        
        // Transfer token ownership to DAO (optional, for full decentralization)
        // token.transferOwnership(address(dao));
        // console2.log("Token ownership transferred to DAO");
        
        vm.stopBroadcast();
        
        console2.log("\n=== Deployment Summary ===");
        console2.log("ManifoldGovernanceToken:", address(token));
        console2.log("ManifoldDAO:", address(dao));
        console2.log("\nDAO Parameters:");
        console2.log("- Proposal Threshold:", PROPOSAL_THRESHOLD / 10 ** 18, "MGT");
        console2.log("- Voting Period:", VOTING_PERIOD, "blocks");
        console2.log("- Execution Delay:", EXECUTION_DELAY / 1 days, "days");
        console2.log("- Quorum:", QUORUM_PERCENTAGE, "%");
    }
}
