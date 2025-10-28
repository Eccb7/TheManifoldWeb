//! Manifold DAO - Decentralized Governance
//!
//! Features:
//! - Proposal creation and voting
//! - Token-weighted voting (1 token = 1 vote)
//! - Time-locked execution
//! - Quorum requirements
//! - Multiple proposal types (parameter changes, resource allocation, upgrades)

use crate::{
    storage::helpers::{get_json, set_json},
    Contract, ContractContext, ContractError, ContractResult,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// DAO configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DAOConfig {
    /// Minimum tokens required to create proposal
    pub proposal_threshold: u64,
    
    /// Voting period in blocks
    pub voting_period: u64,
    
    /// Execution delay in blocks after proposal passes
    pub execution_delay: u64,
    
    /// Quorum percentage (0-100)
    pub quorum_percentage: u8,
    
    /// Address of governance token contract
    pub token_contract: String,
}

impl Default for DAOConfig {
    fn default() -> Self {
        const DECIMALS: u32 = 6;
        Self {
            proposal_threshold: 10_000 * 10u64.pow(DECIMALS), // 10,000 MGT
            voting_period: 50_400,                             // ~7 days (assuming 12s blocks)
            execution_delay: 14_400,                           // ~2 days
            quorum_percentage: 10,                             // 10% quorum
            token_contract: String::new(),
        }
    }
}

/// Proposal status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProposalStatus {
    Pending,
    Active,
    Defeated,
    Succeeded,
    Queued,
    Executed,
    Canceled,
}

/// Proposal types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProposalType {
    /// Change protocol parameter
    ParameterChange { key: String, value: String },
    
    /// Allocate resources
    ResourceAllocation { amount: u64, recipient: String },
    
    /// Upgrade protocol
    ProtocolUpgrade { version: String, code_hash: String },
    
    /// Custom proposal with arbitrary data
    Custom { data: Vec<u8> },
}

/// A governance proposal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    pub id: u64,
    pub proposer: String,
    pub proposal_type: ProposalType,
    pub description: String,
    pub start_block: u64,
    pub end_block: u64,
    pub execution_block: u64,
    pub votes_for: u64,
    pub votes_against: u64,
    pub votes_abstain: u64,
    pub status: ProposalStatus,
    pub executed: bool,
    pub canceled: bool,
    pub execution_data: Vec<u8>,
}

/// Vote record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    pub voter: String,
    pub proposal_id: u64,
    pub support: bool,
    pub weight: u64,
    pub block_height: u64,
}

/// DAO state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DAOState {
    pub config: DAOConfig,
    pub proposal_count: u64,
    pub proposals: HashMap<u64, Proposal>,
    pub votes: HashMap<u64, HashMap<String, Vote>>,
    pub total_token_supply: u64, // Cached from token contract
}

impl DAOState {
    pub fn new(config: DAOConfig) -> Self {
        Self {
            config,
            proposal_count: 0,
            proposals: HashMap::new(),
            votes: HashMap::new(),
            total_token_supply: 0,
        }
    }
}

/// DAO messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DAOMsg {
    /// Create a new proposal
    Propose {
        proposal_type: ProposalType,
        description: String,
        execution_data: Vec<u8>,
    },
    
    /// Cast a vote
    Vote {
        proposal_id: u64,
        support: bool,
    },
    
    /// Queue a successful proposal
    Queue { proposal_id: u64 },
    
    /// Execute a queued proposal
    Execute { proposal_id: u64 },
    
    /// Cancel a proposal (only by proposer)
    Cancel { proposal_id: u64 },
    
    /// Update DAO parameters (only by DAO itself)
    UpdateConfig { config: DAOConfig },
}

/// Query messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DAOQuery {
    /// Get proposal by ID
    Proposal { id: u64 },
    
    /// Get proposal status
    ProposalStatus { id: u64 },
    
    /// Get DAO config
    Config,
    
    /// Get vote by proposal and voter
    Vote { proposal_id: u64, voter: String },
    
    /// List all proposals
    ListProposals,
    
    /// List active proposals
    ActiveProposals,
}

/// Query responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DAOQueryResponse {
    Proposal { proposal: Proposal },
    ProposalStatus { status: ProposalStatus },
    Config { config: DAOConfig },
    Vote { vote: Option<Vote> },
    ProposalList { proposals: Vec<Proposal> },
}

/// Manifold DAO Contract
pub struct ManifoldDAO;

impl Contract for ManifoldDAO {
    type InstantiateMsg = DAOConfig;
    type ExecuteMsg = DAOMsg;
    type QueryMsg = DAOQuery;
    type QueryResponse = DAOQueryResponse;
    type MigrateMsg = ();

    fn instantiate(
        ctx: &mut ContractContext,
        msg: Self::InstantiateMsg,
    ) -> ContractResult<()> {
        ctx.consume_gas(2000)?;

        let state = DAOState::new(msg);
        set_json(ctx.storage.as_mut(), b"state", &state)?;

        Ok(())
    }

    fn execute(
        ctx: &mut ContractContext,
        msg: Self::ExecuteMsg,
    ) -> ContractResult<Vec<u8>> {
        ctx.consume_gas(1000)?;

        let mut state: DAOState = get_json(ctx.storage.as_ref(), b"state")?
            .ok_or_else(|| ContractError::InvalidState("DAO not initialized".to_string()))?;

        match msg {
            DAOMsg::Propose {
                proposal_type,
                description,
                execution_data,
            } => {
                Self::create_proposal(
                    &mut state,
                    ctx,
                    proposal_type,
                    description,
                    execution_data,
                )?;
            }
            DAOMsg::Vote {
                proposal_id,
                support,
            } => {
                Self::cast_vote(&mut state, ctx, proposal_id, support)?;
            }
            DAOMsg::Queue { proposal_id } => {
                Self::queue_proposal(&mut state, ctx, proposal_id)?;
            }
            DAOMsg::Execute { proposal_id } => {
                Self::execute_proposal(&mut state, ctx, proposal_id)?;
            }
            DAOMsg::Cancel { proposal_id } => {
                Self::cancel_proposal(&mut state, ctx, proposal_id)?;
            }
            DAOMsg::UpdateConfig { config } => {
                // Only DAO itself can update config
                if ctx.info.sender.to_string() != ctx.info.contract_address {
                    return Err(ContractError::Unauthorized(
                        "Only DAO can update config".to_string(),
                    ));
                }
                state.config = config;
            }
        }

        set_json(ctx.storage.as_mut(), b"state", &state)?;

        Ok(vec![])
    }

    fn query(
        ctx: &ContractContext,
        msg: Self::QueryMsg,
    ) -> ContractResult<Self::QueryResponse> {
        let state: DAOState = get_json(ctx.storage.as_ref(), b"state")?
            .ok_or_else(|| ContractError::InvalidState("DAO not initialized".to_string()))?;

        match msg {
            DAOQuery::Proposal { id } => {
                let proposal = state
                    .proposals
                    .get(&id)
                    .ok_or_else(|| ContractError::NotFound(format!("Proposal {}", id)))?
                    .clone();
                Ok(DAOQueryResponse::Proposal { proposal })
            }
            DAOQuery::ProposalStatus { id } => {
                let proposal = state
                    .proposals
                    .get(&id)
                    .ok_or_else(|| ContractError::NotFound(format!("Proposal {}", id)))?;
                let status = Self::get_proposal_status(proposal, &state, ctx.info.block_height);
                Ok(DAOQueryResponse::ProposalStatus { status })
            }
            DAOQuery::Config => Ok(DAOQueryResponse::Config {
                config: state.config.clone(),
            }),
            DAOQuery::Vote { proposal_id, voter } => {
                let vote = state
                    .votes
                    .get(&proposal_id)
                    .and_then(|votes| votes.get(&voter))
                    .cloned();
                Ok(DAOQueryResponse::Vote { vote })
            }
            DAOQuery::ListProposals => {
                let proposals: Vec<Proposal> = state.proposals.values().cloned().collect();
                Ok(DAOQueryResponse::ProposalList { proposals })
            }
            DAOQuery::ActiveProposals => {
                let proposals: Vec<Proposal> = state
                    .proposals
                    .values()
                    .filter(|p| {
                        let status = Self::get_proposal_status(p, &state, ctx.info.block_height);
                        status == ProposalStatus::Active
                    })
                    .cloned()
                    .collect();
                Ok(DAOQueryResponse::ProposalList { proposals })
            }
        }
    }
}

impl ManifoldDAO {
    fn create_proposal(
        state: &mut DAOState,
        ctx: &ContractContext,
        proposal_type: ProposalType,
        description: String,
        execution_data: Vec<u8>,
    ) -> ContractResult<()> {
        // TODO: Check voting power via token contract
        // For now, we assume proposer has enough tokens

        let proposal_id = state.proposal_count + 1;
        state.proposal_count = proposal_id;

        let start_block = ctx.info.block_height + 1;
        let end_block = start_block + state.config.voting_period;

        let proposal = Proposal {
            id: proposal_id,
            proposer: ctx.info.sender.to_string(),
            proposal_type,
            description,
            start_block,
            end_block,
            execution_block: 0,
            votes_for: 0,
            votes_against: 0,
            votes_abstain: 0,
            status: ProposalStatus::Pending,
            executed: false,
            canceled: false,
            execution_data,
        };

        state.proposals.insert(proposal_id, proposal);
        state.votes.insert(proposal_id, HashMap::new());

        Ok(())
    }

    fn cast_vote(
        state: &mut DAOState,
        ctx: &ContractContext,
        proposal_id: u64,
        support: bool,
    ) -> ContractResult<()> {
        // Check proposal exists and get initial status
        let (start_block, end_block, already_voted) = {
            let proposal = state
                .proposals
                .get(&proposal_id)
                .ok_or_else(|| ContractError::NotFound(format!("Proposal {}", proposal_id)))?;
            
            let voter = ctx.info.sender.to_string();
            let votes_map = state.votes.get(&proposal_id).unwrap();
            let already_voted = votes_map.contains_key(&voter);
            
            (proposal.start_block, proposal.end_block, already_voted)
        };

        // Check if voting is active
        if ctx.info.block_height < start_block || ctx.info.block_height > end_block {
            return Err(ContractError::InvalidState("Proposal not active".to_string()));
        }

        if already_voted {
            return Err(ContractError::Custom("Already voted".to_string()));
        }

        let voter = ctx.info.sender.to_string();
        
        // TODO: Get actual voting power from token contract
        let weight = 1000; // Placeholder

        let vote = Vote {
            voter: voter.clone(),
            proposal_id,
            support,
            weight,
            block_height: ctx.info.block_height,
        };

        // Update proposal votes
        let proposal = state.proposals.get_mut(&proposal_id).unwrap();
        if support {
            proposal.votes_for += weight;
        } else {
            proposal.votes_against += weight;
        }

        // Record vote
        let votes_map = state.votes.get_mut(&proposal_id).unwrap();
        votes_map.insert(voter, vote);

        Ok(())
    }

    fn queue_proposal(
        state: &mut DAOState,
        ctx: &ContractContext,
        proposal_id: u64,
    ) -> ContractResult<()> {
        // Check status first
        let is_succeeded = {
            let proposal = state
                .proposals
                .get(&proposal_id)
                .ok_or_else(|| ContractError::NotFound(format!("Proposal {}", proposal_id)))?;
            
            let status = Self::get_proposal_status(proposal, state, ctx.info.block_height);
            status == ProposalStatus::Succeeded
        };

        if !is_succeeded {
            return Err(ContractError::InvalidState(
                "Proposal not succeeded".to_string(),
            ));
        }

        let proposal = state.proposals.get_mut(&proposal_id).unwrap();
        proposal.execution_block = ctx.info.block_height + state.config.execution_delay;
        proposal.status = ProposalStatus::Queued;

        Ok(())
    }

    fn execute_proposal(
        state: &mut DAOState,
        ctx: &ContractContext,
        proposal_id: u64,
    ) -> ContractResult<()> {
        // Check status and execution block
        let (is_queued, execution_block) = {
            let proposal = state
                .proposals
                .get(&proposal_id)
                .ok_or_else(|| ContractError::NotFound(format!("Proposal {}", proposal_id)))?;
            
            let status = Self::get_proposal_status(proposal, state, ctx.info.block_height);
            (status == ProposalStatus::Queued, proposal.execution_block)
        };

        if !is_queued {
            return Err(ContractError::InvalidState("Proposal not queued".to_string()));
        }

        if ctx.info.block_height < execution_block {
            return Err(ContractError::InvalidState(
                "Execution delay not passed".to_string(),
            ));
        }

        let proposal = state.proposals.get_mut(&proposal_id).unwrap();
        proposal.executed = true;
        proposal.status = ProposalStatus::Executed;

        // TODO: Execute proposal logic based on type
        // For now, just mark as executed

        Ok(())
    }

    fn cancel_proposal(
        state: &mut DAOState,
        ctx: &ContractContext,
        proposal_id: u64,
    ) -> ContractResult<()> {
        let proposal = state
            .proposals
            .get_mut(&proposal_id)
            .ok_or_else(|| ContractError::NotFound(format!("Proposal {}", proposal_id)))?;

        if ctx.info.sender.to_string() != proposal.proposer {
            return Err(ContractError::Unauthorized("Not proposer".to_string()));
        }

        if proposal.executed {
            return Err(ContractError::InvalidState(
                "Already executed".to_string(),
            ));
        }

        proposal.canceled = true;
        proposal.status = ProposalStatus::Canceled;

        Ok(())
    }

    fn get_proposal_status(
        proposal: &Proposal,
        state: &DAOState,
        current_block: u64,
    ) -> ProposalStatus {
        if proposal.canceled {
            return ProposalStatus::Canceled;
        }

        if proposal.executed {
            return ProposalStatus::Executed;
        }

        if current_block < proposal.start_block {
            return ProposalStatus::Pending;
        }

        if current_block <= proposal.end_block {
            return ProposalStatus::Active;
        }

        // Voting ended, check if passed
        let total_votes = proposal.votes_for + proposal.votes_against;
        let quorum = (state.total_token_supply * state.config.quorum_percentage as u64) / 100;

        if proposal.votes_for <= proposal.votes_against || total_votes < quorum {
            return ProposalStatus::Defeated;
        }

        if proposal.execution_block == 0 {
            return ProposalStatus::Succeeded;
        }

        ProposalStatus::Queued
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{storage::MemoryStorage, ExecutionContext};
    use libp2p_identity::PeerId;

    fn create_test_context(block_height: u64) -> ContractContext {
        let info = ExecutionContext::new(
            "dao_contract".to_string(),
            PeerId::random(),
            block_height,
            "tx_123".to_string(),
        );
        let storage = Box::new(MemoryStorage::new());
        ContractContext::new(info, storage, 10_000_000)
    }

    #[test]
    fn test_dao_instantiation() {
        let mut ctx = create_test_context(1);
        let config = DAOConfig::default();

        ManifoldDAO::instantiate(&mut ctx, config).unwrap();

        let response = ManifoldDAO::query(&ctx, DAOQuery::Config).unwrap();

        match response {
            DAOQueryResponse::Config { config } => {
                assert_eq!(config.quorum_percentage, 10);
            }
            _ => panic!("Wrong response type"),
        }
    }

    #[test]
    fn test_create_proposal() {
        let mut ctx = create_test_context(1);
        let config = DAOConfig::default();

        ManifoldDAO::instantiate(&mut ctx, config).unwrap();

        ManifoldDAO::execute(
            &mut ctx,
            DAOMsg::Propose {
                proposal_type: ProposalType::ParameterChange {
                    key: "max_energy".to_string(),
                    value: "2000".to_string(),
                },
                description: "Increase max energy".to_string(),
                execution_data: vec![],
            },
        )
        .unwrap();

        let response = ManifoldDAO::query(&ctx, DAOQuery::Proposal { id: 1 }).unwrap();

        match response {
            DAOQueryResponse::Proposal { proposal } => {
                assert_eq!(proposal.id, 1);
                assert_eq!(proposal.description, "Increase max energy");
            }
            _ => panic!("Wrong response type"),
        }
    }
}
