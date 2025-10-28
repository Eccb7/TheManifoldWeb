//! Simple token contract template for developers

use crate::{Contract, ContractContext, ContractResult, ContractError};
use crate::storage::helpers::{set_json, get_json};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleTokenConfig {
    pub name: String,
    pub symbol: String,
    pub decimals: u32,
    pub initial_supply: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SimpleTokenMsg {
    Transfer { to: String, amount: u64 },
    Approve { spender: String, amount: u64 },
    TransferFrom { from: String, to: String, amount: u64 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SimpleTokenQuery {
    Balance { address: String },
    Allowance { owner: String, spender: String },
    TotalSupply,
    TokenInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenInfo {
    pub name: String,
    pub symbol: String,
    pub decimals: u32,
    pub total_supply: u64,
}

pub struct SimpleTokenTemplate;

impl Contract for SimpleTokenTemplate {
    type InstantiateMsg = SimpleTokenConfig;
    type ExecuteMsg = SimpleTokenMsg;
    type QueryMsg = SimpleTokenQuery;
    type MigrateMsg = ();
    type QueryResponse = Vec<u8>;

    fn instantiate(ctx: &mut ContractContext, msg: Self::InstantiateMsg) -> ContractResult<()> {
        // Store token info
        set_json(&mut *ctx.storage, b"config", &msg)?;
        set_json(&mut *ctx.storage, b"total_supply", &msg.initial_supply)?;
        
        // Mint initial supply to deployer
        let deployer = ctx.info.sender.to_string();
        set_json(&mut *ctx.storage, format!("balance:{}", deployer).as_bytes(), &msg.initial_supply)?;
        
        ctx.add_attribute("action", "instantiate");
        ctx.add_attribute("name", &msg.name);
        ctx.add_attribute("symbol", &msg.symbol);
        ctx.add_attribute("initial_supply", &msg.initial_supply.to_string());
        
        Ok(())
    }

    fn execute(ctx: &mut ContractContext, msg: Self::ExecuteMsg) -> ContractResult<Vec<u8>> {
        match msg {
            SimpleTokenMsg::Transfer { to, amount } => {
                Self::transfer(ctx, &ctx.info.sender.to_string(), &to, amount)?;
                Ok(vec![])
            }
            SimpleTokenMsg::Approve { spender, amount } => {
                Self::approve(ctx, &ctx.info.sender.to_string(), &spender, amount)?;
                Ok(vec![])
            }
            SimpleTokenMsg::TransferFrom { from, to, amount } => {
                Self::transfer_from(ctx, &ctx.info.sender.to_string(), &from, &to, amount)?;
                Ok(vec![])
            }
        }
    }

    fn query(ctx: &ContractContext, msg: Self::QueryMsg) -> ContractResult<Self::QueryResponse> {
        match msg {
            SimpleTokenQuery::Balance { address } => {
                let balance = Self::get_balance(ctx, &address);
                Ok(serde_json::to_vec(&balance)?)
            }
            SimpleTokenQuery::Allowance { owner, spender } => {
                let allowance = Self::get_allowance(ctx, &owner, &spender);
                Ok(serde_json::to_vec(&allowance)?)
            }
            SimpleTokenQuery::TotalSupply => {
                let supply: u64 = get_json(&*ctx.storage, b"total_supply")?.unwrap_or(0);
                Ok(serde_json::to_vec(&supply)?)
            }
            SimpleTokenQuery::TokenInfo => {
                let config: SimpleTokenConfig = get_json(&*ctx.storage, b"config")?
                    .ok_or(ContractError::NotFound("config not found".to_string()))?;
                let total_supply: u64 = get_json(&*ctx.storage, b"total_supply")?.unwrap_or(0);
                let info = TokenInfo {
                    name: config.name,
                    symbol: config.symbol,
                    decimals: config.decimals,
                    total_supply,
                };
                Ok(serde_json::to_vec(&info)?)
            }
        }
    }

    fn migrate(_ctx: &mut ContractContext, _msg: Self::MigrateMsg) -> ContractResult<()> {
        Err(ContractError::Unauthorized("migration not supported".to_string()))
    }
}

impl SimpleTokenTemplate {
    fn get_balance(ctx: &ContractContext, address: &str) -> u64 {
        get_json(&*ctx.storage, format!("balance:{}", address).as_bytes())
            .ok()
            .flatten()
            .unwrap_or(0)
    }

    fn get_allowance(ctx: &ContractContext, owner: &str, spender: &str) -> u64 {
        get_json(&*ctx.storage, format!("allowance:{}:{}", owner, spender).as_bytes())
            .ok()
            .flatten()
            .unwrap_or(0)
    }

    fn transfer(ctx: &mut ContractContext, from: &str, to: &str, amount: u64) -> ContractResult<()> {
        let from_balance = Self::get_balance(ctx, from);
        if from_balance < amount {
            return Err(ContractError::InsufficientFunds(
                format!("insufficient balance: {} < {}", from_balance, amount)
            ));
        }

        let to_balance = Self::get_balance(ctx, to);
        set_json(&mut *ctx.storage, format!("balance:{}", from).as_bytes(), &(from_balance - amount))?;
        set_json(&mut *ctx.storage, format!("balance:{}", to).as_bytes(), &(to_balance + amount))?;

        ctx.add_attribute("action", "transfer");
        ctx.add_attribute("from", from);
        ctx.add_attribute("to", to);
        ctx.add_attribute("amount", &amount.to_string());

        Ok(())
    }

    fn approve(ctx: &mut ContractContext, owner: &str, spender: &str, amount: u64) -> ContractResult<()> {
        set_json(&mut *ctx.storage, format!("allowance:{}:{}", owner, spender).as_bytes(), &amount)?;

        ctx.add_attribute("action", "approve");
        ctx.add_attribute("owner", owner);
        ctx.add_attribute("spender", spender);
        ctx.add_attribute("amount", &amount.to_string());

        Ok(())
    }

    fn transfer_from(
        ctx: &mut ContractContext,
        spender: &str,
        from: &str,
        to: &str,
        amount: u64,
    ) -> ContractResult<()> {
        let allowance = Self::get_allowance(ctx, from, spender);
        if allowance < amount {
            return Err(ContractError::Unauthorized(
                format!("insufficient allowance: {} < {}", allowance, amount)
            ));
        }

        Self::transfer(ctx, from, to, amount)?;
        set_json(&mut *ctx.storage, format!("allowance:{}:{}", from, spender).as_bytes(), &(allowance - amount))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ExecutionContext, MemoryStorage};
    use libp2p_identity::PeerId;

    #[test]
    fn test_simple_token_instantiate() {
        let peer = PeerId::random();
        let info = ExecutionContext::new(
            "contract_addr".to_string(),
            peer,
            100,
            "tx_hash".to_string(),
        );
        let storage = Box::new(MemoryStorage::new());
        let mut ctx = ContractContext::new(info, storage, 10_000_000);

        let config = SimpleTokenConfig {
            name: "Test Token".to_string(),
            symbol: "TEST".to_string(),
            decimals: 6,
            initial_supply: 1_000_000_000_000,
        };

        SimpleTokenTemplate::instantiate(&mut ctx, config).unwrap();

        let balance = SimpleTokenTemplate::get_balance(&ctx, &peer.to_string());
        assert_eq!(balance, 1_000_000_000_000);
    }

    #[test]
    fn test_simple_token_transfer() {
        let peer = PeerId::random();
        let recipient = PeerId::random();
        let info = ExecutionContext::new(
            "contract_addr".to_string(),
            peer.clone(),
            100,
            "tx_hash".to_string(),
        );
        let storage = Box::new(MemoryStorage::new());
        let mut ctx = ContractContext::new(info, storage, 10_000_000);

        let config = SimpleTokenConfig {
            name: "Test Token".to_string(),
            symbol: "TEST".to_string(),
            decimals: 6,
            initial_supply: 1_000_000_000_000,
        };

        SimpleTokenTemplate::instantiate(&mut ctx, config).unwrap();

        SimpleTokenTemplate::execute(
            &mut ctx,
            SimpleTokenMsg::Transfer {
                to: recipient.to_string(),
                amount: 500_000_000_000,
            },
        )
        .unwrap();

        let sender_balance = SimpleTokenTemplate::get_balance(&ctx, &peer.to_string());
        let recipient_balance = SimpleTokenTemplate::get_balance(&ctx, &recipient.to_string());

        assert_eq!(sender_balance, 500_000_000_000);
        assert_eq!(recipient_balance, 500_000_000_000);
    }
}
