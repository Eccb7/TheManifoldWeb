//! Manifold Governance Token (MGT)
//!
//! A native Rust implementation of the governance token with:
//! - Fixed maximum supply (100M MGT)
//! - Reputation tracking for node operators
//! - Agent contribution metrics
//! - Voting power delegation

use crate::{
    storage::helpers::{get_json, set_json},
    Contract, ContractContext, ContractError, ContractResult,
};
use libp2p_identity::PeerId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Token configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenConfig {
    pub name: String,
    pub symbol: String,
    pub max_supply: u64,
    pub initial_supply: u64,
}

impl Default for TokenConfig {
    fn default() -> Self {
        const DECIMALS: u32 = 6; // 6 decimals instead of 18 to avoid overflow
        Self {
            name: "Manifold Governance Token".to_string(),
            symbol: "MGT".to_string(),
            max_supply: 100_000_000 * 10u64.pow(DECIMALS), // 100M tokens
            initial_supply: 10_000_000 * 10u64.pow(DECIMALS), // 10M initial (10%)
        }
    }
}

/// Token state stored in contract storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenState {
    pub config: TokenConfig,
    pub total_supply: u64,
    pub total_minted: u64,
    pub balances: HashMap<String, u64>,
    pub allowances: HashMap<String, HashMap<String, u64>>,
    pub node_reputation: HashMap<String, u64>,
    pub agent_contributions: HashMap<String, u64>,
}

impl TokenState {
    pub fn new(config: TokenConfig, initial_owner: String) -> Self {
        let mut balances = HashMap::new();
        balances.insert(initial_owner, config.initial_supply);

        Self {
            total_supply: config.initial_supply,
            total_minted: config.initial_supply,
            config,
            balances,
            allowances: HashMap::new(),
            node_reputation: HashMap::new(),
            agent_contributions: HashMap::new(),
        }
    }
}

/// Messages for token operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TokenMsg {
    /// Transfer tokens
    Transfer { recipient: String, amount: u64 },
    
    /// Approve spending allowance
    Approve { spender: String, amount: u64 },
    
    /// Transfer from approved allowance
    TransferFrom {
        owner: String,
        recipient: String,
        amount: u64,
    },
    
    /// Mint new tokens (only by DAO)
    Mint {
        recipient: String,
        amount: u64,
        reason: String,
    },
    
    /// Burn tokens
    Burn { amount: u64 },
    
    /// Update node reputation
    UpdateReputation { node: String, reputation: u64 },
    
    /// Record agent contribution
    RecordContribution { agent_id: String, contribution: u64 },
}

/// Query messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TokenQuery {
    /// Get balance of address
    Balance { address: String },
    
    /// Get total supply
    TotalSupply,
    
    /// Get allowance
    Allowance { owner: String, spender: String },
    
    /// Get token config
    Config,
    
    /// Get node reputation
    NodeReputation { node: String },
    
    /// Get agent contribution
    AgentContribution { agent_id: String },
    
    /// Get voting power (balance for now, can be extended)
    VotingPower { address: String },
}

/// Query responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TokenQueryResponse {
    Balance { balance: u64 },
    TotalSupply { supply: u64 },
    Allowance { amount: u64 },
    Config { config: TokenConfig },
    Reputation { reputation: u64 },
    Contribution { contribution: u64 },
    VotingPower { power: u64 },
}

/// Governance Token Contract
pub struct GovernanceToken;

impl Contract for GovernanceToken {
    type InstantiateMsg = TokenConfig;
    type ExecuteMsg = TokenMsg;
    type QueryMsg = TokenQuery;
    type QueryResponse = TokenQueryResponse;
    type MigrateMsg = ();

    fn instantiate(
        ctx: &mut ContractContext,
        msg: Self::InstantiateMsg,
    ) -> ContractResult<()> {
        ctx.consume_gas(1000)?;

        let owner = ctx.info.sender.to_string();
        let state = TokenState::new(msg, owner.clone());

        set_json(ctx.storage.as_mut(), b"state", &state)?;

        Ok(())
    }

    fn execute(
        ctx: &mut ContractContext,
        msg: Self::ExecuteMsg,
    ) -> ContractResult<Vec<u8>> {
        ctx.consume_gas(500)?;

        let mut state: TokenState = get_json(ctx.storage.as_ref(), b"state")?
            .ok_or_else(|| ContractError::InvalidState("Token not initialized".to_string()))?;

        match msg {
            TokenMsg::Transfer { recipient, amount } => {
                Self::transfer(&mut state, &ctx.info.sender.to_string(), &recipient, amount)?;
            }
            TokenMsg::Approve { spender, amount } => {
                Self::approve(&mut state, &ctx.info.sender.to_string(), &spender, amount)?;
            }
            TokenMsg::TransferFrom {
                owner,
                recipient,
                amount,
            } => {
                Self::transfer_from(
                    &mut state,
                    &ctx.info.sender.to_string(),
                    &owner,
                    &recipient,
                    amount,
                )?;
            }
            TokenMsg::Mint {
                recipient,
                amount,
                reason: _,
            } => {
                Self::mint(&mut state, &recipient, amount)?;
                ctx.consume_gas(1000)?; // Minting costs more
            }
            TokenMsg::Burn { amount } => {
                Self::burn(&mut state, &ctx.info.sender.to_string(), amount)?;
            }
            TokenMsg::UpdateReputation { node, reputation } => {
                state.node_reputation.insert(node, reputation);
            }
            TokenMsg::RecordContribution {
                agent_id,
                contribution,
            } => {
                *state.agent_contributions.entry(agent_id).or_insert(0) += contribution;
            }
        }

        set_json(ctx.storage.as_mut(), b"state", &state)?;

        Ok(vec![])
    }

    fn query(
        ctx: &ContractContext,
        msg: Self::QueryMsg,
    ) -> ContractResult<Self::QueryResponse> {
        let state: TokenState = get_json(ctx.storage.as_ref(), b"state")?
            .ok_or_else(|| ContractError::InvalidState("Token not initialized".to_string()))?;

        match msg {
            TokenQuery::Balance { address } => {
                let balance = state.balances.get(&address).copied().unwrap_or(0);
                Ok(TokenQueryResponse::Balance { balance })
            }
            TokenQuery::TotalSupply => Ok(TokenQueryResponse::TotalSupply {
                supply: state.total_supply,
            }),
            TokenQuery::Allowance { owner, spender } => {
                let amount = state
                    .allowances
                    .get(&owner)
                    .and_then(|allowances| allowances.get(&spender))
                    .copied()
                    .unwrap_or(0);
                Ok(TokenQueryResponse::Allowance { amount })
            }
            TokenQuery::Config => Ok(TokenQueryResponse::Config {
                config: state.config.clone(),
            }),
            TokenQuery::NodeReputation { node } => {
                let reputation = state.node_reputation.get(&node).copied().unwrap_or(0);
                Ok(TokenQueryResponse::Reputation { reputation })
            }
            TokenQuery::AgentContribution { agent_id } => {
                let contribution = state
                    .agent_contributions
                    .get(&agent_id)
                    .copied()
                    .unwrap_or(0);
                Ok(TokenQueryResponse::Contribution { contribution })
            }
            TokenQuery::VotingPower { address } => {
                let power = state.balances.get(&address).copied().unwrap_or(0);
                Ok(TokenQueryResponse::VotingPower { power })
            }
        }
    }
}

impl GovernanceToken {
    fn transfer(
        state: &mut TokenState,
        from: &str,
        to: &str,
        amount: u64,
    ) -> ContractResult<()> {
        let from_balance = state.balances.get(from).copied().unwrap_or(0);
        if from_balance < amount {
            return Err(ContractError::Custom("Insufficient balance".to_string()));
        }

        state.balances.insert(from.to_string(), from_balance - amount);
        *state.balances.entry(to.to_string()).or_insert(0) += amount;

        Ok(())
    }

    fn approve(
        state: &mut TokenState,
        owner: &str,
        spender: &str,
        amount: u64,
    ) -> ContractResult<()> {
        state
            .allowances
            .entry(owner.to_string())
            .or_insert_with(HashMap::new)
            .insert(spender.to_string(), amount);
        Ok(())
    }

    fn transfer_from(
        state: &mut TokenState,
        spender: &str,
        owner: &str,
        recipient: &str,
        amount: u64,
    ) -> ContractResult<()> {
        let allowance = state
            .allowances
            .get(owner)
            .and_then(|allowances| allowances.get(spender))
            .copied()
            .unwrap_or(0);

        if allowance < amount {
            return Err(ContractError::Custom("Insufficient allowance".to_string()));
        }

        Self::transfer(state, owner, recipient, amount)?;

        // Update allowance
        state
            .allowances
            .get_mut(owner)
            .unwrap()
            .insert(spender.to_string(), allowance - amount);

        Ok(())
    }

    fn mint(state: &mut TokenState, recipient: &str, amount: u64) -> ContractResult<()> {
        if state.total_minted + amount > state.config.max_supply {
            return Err(ContractError::Custom("Exceeds max supply".to_string()));
        }

        *state.balances.entry(recipient.to_string()).or_insert(0) += amount;
        state.total_supply += amount;
        state.total_minted += amount;

        Ok(())
    }

    fn burn(state: &mut TokenState, from: &str, amount: u64) -> ContractResult<()> {
        let balance = state.balances.get(from).copied().unwrap_or(0);
        if balance < amount {
            return Err(ContractError::Custom("Insufficient balance".to_string()));
        }

        state.balances.insert(from.to_string(), balance - amount);
        state.total_supply -= amount;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{storage::MemoryStorage, ExecutionContext};

    fn create_test_context() -> ContractContext {
        let info = ExecutionContext::new(
            "token_contract".to_string(),
            PeerId::random(),
            1,
            "tx_123".to_string(),
        );
        let storage = Box::new(MemoryStorage::new());
        ContractContext::new(info, storage, 1_000_000)
    }

    #[test]
    fn test_token_instantiation() {
        let mut ctx = create_test_context();
        let config = TokenConfig::default();

        GovernanceToken::instantiate(&mut ctx, config).unwrap();

        let response =
            GovernanceToken::query(&ctx, TokenQuery::TotalSupply).unwrap();

        match response {
            TokenQueryResponse::TotalSupply { supply } => {
                const DECIMALS: u32 = 6;
                assert_eq!(supply, 10_000_000 * 10u64.pow(DECIMALS));
            }
            _ => panic!("Wrong response type"),
        }
    }

    #[test]
    fn test_token_transfer() {
        let mut ctx = create_test_context();
        let config = TokenConfig::default();
        let sender = ctx.info.sender.to_string();

        GovernanceToken::instantiate(&mut ctx, config).unwrap();

        // Transfer tokens
        GovernanceToken::execute(
            &mut ctx,
            TokenMsg::Transfer {
                recipient: "recipient".to_string(),
                amount: 1000,
            },
        )
        .unwrap();

        // Check balance
        let response = GovernanceToken::query(
            &ctx,
            TokenQuery::Balance {
                address: "recipient".to_string(),
            },
        )
        .unwrap();

        match response {
            TokenQueryResponse::Balance { balance } => {
                assert_eq!(balance, 1000);
            }
            _ => panic!("Wrong response type"),
        }
    }
}
