//! Custom governance contract template

use crate::{Contract, ContractContext, ContractResult, ContractError};
use crate::storage::helpers::{set_json, get_json};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomGovernanceConfig {
    pub voting_period_blocks: u64,
    pub execution_delay_blocks: u64,
    pub quorum_percentage: u8,
    pub proposal_threshold: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CustomGovernanceMsg {
    CreateProposal {
        title: String,
        description: String,
        actions: Vec<String>,
    },
    Vote {
        proposal_id: u64,
        vote: bool,
    },
    ExecuteProposal {
        proposal_id: u64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CustomGovernanceQuery {
    Proposal { proposal_id: u64 },
    Proposals,
    Config,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposal {
    pub id: u64,
    pub title: String,
    pub description: String,
    pub proposer: String,
    pub yes_votes: u64,
    pub no_votes: u64,
    pub start_block: u64,
    pub end_block: u64,
    pub executed: bool,
}

pub struct CustomGovernanceTemplate;

impl Contract for CustomGovernanceTemplate {
    type InstantiateMsg = CustomGovernanceConfig;
    type ExecuteMsg = CustomGovernanceMsg;
    type QueryMsg = CustomGovernanceQuery;
    type MigrateMsg = ();
    type QueryResponse = Vec<u8>;

    fn instantiate(ctx: &mut ContractContext, msg: Self::InstantiateMsg) -> ContractResult<()> {
        set_json(&mut *ctx.storage, b"config", &msg)?;
        set_json(&mut *ctx.storage, b"proposal_count", &0u64)?;
        
        ctx.add_attribute("action", "instantiate");
        ctx.add_attribute("voting_period", &msg.voting_period_blocks.to_string());
        
        Ok(())
    }

    fn execute(ctx: &mut ContractContext, msg: Self::ExecuteMsg) -> ContractResult<Vec<u8>> {
        match msg {
            CustomGovernanceMsg::CreateProposal { title, description, actions } => {
                Self::create_proposal(ctx, title, description, actions)?;
                Ok(vec![])
            }
            CustomGovernanceMsg::Vote { proposal_id, vote } => {
                Self::cast_vote(ctx, proposal_id, vote)?;
                Ok(vec![])
            }
            CustomGovernanceMsg::ExecuteProposal { proposal_id } => {
                Self::execute_proposal(ctx, proposal_id)?;
                Ok(vec![])
            }
        }
    }

    fn query(ctx: &ContractContext, msg: Self::QueryMsg) -> ContractResult<Self::QueryResponse> {
        match msg {
            CustomGovernanceQuery::Proposal { proposal_id } => {
                let proposal: Option<Proposal> = get_json(&*ctx.storage, format!("proposal:{}", proposal_id).as_bytes())?;
                Ok(serde_json::to_vec(&proposal)?)
            }
            CustomGovernanceQuery::Proposals => {
                let count: u64 = get_json(&*ctx.storage, b"proposal_count")?.unwrap_or(0);
                let mut proposals: Vec<Proposal> = Vec::new();
                for i in 0..count {
                    if let Some(proposal) = get_json(&*ctx.storage, format!("proposal:{}", i).as_bytes())? {
                        proposals.push(proposal);
                    }
                }
                Ok(serde_json::to_vec(&proposals)?)
            }
            CustomGovernanceQuery::Config => {
                let config: Option<CustomGovernanceConfig> = get_json(&*ctx.storage, b"config")?;
                Ok(serde_json::to_vec(&config)?)
            }
        }
    }

    fn migrate(_ctx: &mut ContractContext, _msg: Self::MigrateMsg) -> ContractResult<()> {
        Err(ContractError::Unauthorized("migration not supported".to_string()))
    }
}

impl CustomGovernanceTemplate {
    fn create_proposal(
        ctx: &mut ContractContext,
        title: String,
        description: String,
        _actions: Vec<String>,
    ) -> ContractResult<()> {
        let config: CustomGovernanceConfig = get_json(&*ctx.storage, b"config")?
            .ok_or(ContractError::NotFound("config not found".to_string()))?;

        let proposal_id: u64 = get_json(&*ctx.storage, b"proposal_count")?.unwrap_or(0);
        
        let proposal = Proposal {
            id: proposal_id,
            title,
            description,
            proposer: ctx.info.sender.to_string(),
            yes_votes: 0,
            no_votes: 0,
            start_block: ctx.info.block_height,
            end_block: ctx.info.block_height + config.voting_period_blocks,
            executed: false,
        };

        set_json(&mut *ctx.storage, format!("proposal:{}", proposal_id).as_bytes(), &proposal)?;
        set_json(&mut *ctx.storage, b"proposal_count", &(proposal_id + 1))?;

        ctx.add_attribute("action", "create_proposal");
        ctx.add_attribute("proposal_id", &proposal_id.to_string());

        Ok(())
    }

    fn cast_vote(ctx: &mut ContractContext, proposal_id: u64, vote: bool) -> ContractResult<()> {
        let mut proposal: Proposal = get_json(&*ctx.storage, format!("proposal:{}", proposal_id).as_bytes())?
            .ok_or(ContractError::NotFound("proposal not found".to_string()))?;

        if ctx.info.block_height > proposal.end_block {
            return Err(ContractError::InvalidInput("voting period ended".to_string()));
        }

        if vote {
            proposal.yes_votes += 1;
        } else {
            proposal.no_votes += 1;
        }

        set_json(&mut *ctx.storage, format!("proposal:{}", proposal_id).as_bytes(), &proposal)?;

        ctx.add_attribute("action", "vote");
        ctx.add_attribute("proposal_id", &proposal_id.to_string());
        ctx.add_attribute("vote", if vote { "yes" } else { "no" });

        Ok(())
    }

    fn execute_proposal(ctx: &mut ContractContext, proposal_id: u64) -> ContractResult<()> {
        let mut proposal: Proposal = get_json(&*ctx.storage, format!("proposal:{}", proposal_id).as_bytes())?
            .ok_or(ContractError::NotFound("proposal not found".to_string()))?;

        if proposal.executed {
            return Err(ContractError::InvalidInput("proposal already executed".to_string()));
        }

        if ctx.info.block_height <= proposal.end_block {
            return Err(ContractError::InvalidInput("voting period not ended".to_string()));
        }

        if proposal.yes_votes <= proposal.no_votes {
            return Err(ContractError::InvalidInput("proposal did not pass".to_string()));
        }

        proposal.executed = true;
        set_json(&mut *ctx.storage, format!("proposal:{}", proposal_id).as_bytes(), &proposal)?;

        ctx.add_attribute("action", "execute_proposal");
        ctx.add_attribute("proposal_id", &proposal_id.to_string());

        Ok(())
    }
}
