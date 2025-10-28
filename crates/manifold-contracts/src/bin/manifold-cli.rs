//! Manifold Contract CLI Tool
//!
//! Command-line interface for deploying and interacting with contracts

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use manifold_contracts::{
    governance::{DAOConfig, TokenConfig, GovernanceToken, ManifoldDAO},
    templates::{
        CustomGovernanceConfig, CustomGovernanceTemplate, SimpleTokenConfig,
        SimpleTokenTemplate,
    },
    Contract, ContractContext, ExecutionContext, MemoryStorage,
};
use libp2p_identity::PeerId;
use serde_json;
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "manifold-cli")]
#[command(about = "Manifold Smart Contract CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Deploy a contract
    Deploy {
        /// Contract type (token, dao, simple-token, custom-gov)
        #[arg(short, long)]
        contract_type: String,

        /// Configuration file path (JSON)
        #[arg(short = 'f', long)]
        config: PathBuf,

        /// Gas limit
        #[arg(short, long, default_value = "10000000")]
        gas_limit: u64,
    },

    /// Generate contract template
    Template {
        /// Template type (token, dao, simple-token, custom-gov)
        #[arg(short, long)]
        template_type: String,

        /// Output file path
        #[arg(short, long)]
        output: PathBuf,
    },

    /// Validate contract configuration
    Validate {
        /// Contract type
        #[arg(short, long)]
        contract_type: String,

        /// Configuration file path (JSON)
        #[arg(short = 'f', long)]
        config: PathBuf,
    },
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();

    match cli.command {
        Commands::Deploy {
            contract_type,
            config,
            gas_limit,
        } => deploy_contract(&contract_type, &config, gas_limit)?,
        Commands::Template {
            template_type,
            output,
        } => generate_template(&template_type, &output)?,
        Commands::Validate {
            contract_type,
            config,
        } => validate_config(&contract_type, &config)?,
    }

    Ok(())
}

fn deploy_contract(contract_type: &str, config_path: &PathBuf, gas_limit: u64) -> Result<()> {
    let config_content = fs::read_to_string(config_path)
        .context("Failed to read configuration file")?;

    let deployer = PeerId::random();
    let contract_addr = format!("contract_{}", hex::encode(&deployer.to_bytes()[..8]));

    println!("🚀 Deploying {} contract...", contract_type);
    println!("   Deployer: {}", deployer);
    println!("   Address: {}", contract_addr);
    println!("   Gas Limit: {}", gas_limit);

    let info = ExecutionContext::new(
        contract_addr.clone(),
        deployer.clone(),
        0,
        format!("deploy_{}", contract_addr),
    );
    let storage = Box::new(MemoryStorage::new());
    let mut ctx = ContractContext::new(info, storage, gas_limit);

    match contract_type {
        "token" => {
            let config: TokenConfig = serde_json::from_str(&config_content)?;
            GovernanceToken::instantiate(&mut ctx, config)?;
            println!("✅ Governance Token deployed successfully!");
        }
        "dao" => {
            let config: DAOConfig = serde_json::from_str(&config_content)?;
            ManifoldDAO::instantiate(&mut ctx, config)?;
            println!("✅ DAO deployed successfully!");
        }
        "simple-token" => {
            let config: SimpleTokenConfig = serde_json::from_str(&config_content)?;
            SimpleTokenTemplate::instantiate(&mut ctx, config)?;
            println!("✅ Simple Token deployed successfully!");
        }
        "custom-gov" => {
            let config: CustomGovernanceConfig = serde_json::from_str(&config_content)?;
            CustomGovernanceTemplate::instantiate(&mut ctx, config)?;
            println!("✅ Custom Governance deployed successfully!");
        }
        _ => anyhow::bail!("Unknown contract type: {}", contract_type),
    }

    println!("   Gas Used: {}", gas_limit - ctx.remaining_gas());
    println!("\n📋 Event Attributes:");
    for (key, value) in ctx.get_attributes() {
        println!("   - {}: {}", key, value);
    }

    Ok(())
}

fn generate_template(template_type: &str, output_path: &PathBuf) -> Result<()> {
    println!("📝 Generating {} template...", template_type);

    let template = match template_type {
        "token" => serde_json::to_string_pretty(&TokenConfig::default())?,
        "dao" => serde_json::to_string_pretty(&DAOConfig::default())?,
        "simple-token" => serde_json::to_string_pretty(&SimpleTokenConfig {
            name: "My Token".to_string(),
            symbol: "MTK".to_string(),
            decimals: 6,
            initial_supply: 1_000_000_000_000,
        })?,
        "custom-gov" => serde_json::to_string_pretty(&CustomGovernanceConfig {
            voting_period_blocks: 50_400,
            execution_delay_blocks: 14_400,
            quorum_percentage: 10,
            proposal_threshold: 10_000,
        })?,
        _ => anyhow::bail!("Unknown template type: {}", template_type),
    };

    fs::write(output_path, template)
        .context("Failed to write template file")?;

    println!("✅ Template written to: {}", output_path.display());

    Ok(())
}

fn validate_config(contract_type: &str, config_path: &PathBuf) -> Result<()> {
    println!("🔍 Validating {} configuration...", contract_type);

    let config_content = fs::read_to_string(config_path)
        .context("Failed to read configuration file")?;

    match contract_type {
        "token" => {
            let config: TokenConfig = serde_json::from_str(&config_content)?;
            println!("✅ Valid TokenConfig:");
            println!("   Name: {}", config.name);
            println!("   Symbol: {}", config.symbol);
            println!("   Max Supply: {}", config.max_supply);
            println!("   Initial Supply: {}", config.initial_supply);
        }
        "dao" => {
            let config: DAOConfig = serde_json::from_str(&config_content)?;
            println!("✅ Valid DAOConfig:");
            println!("   Token Contract: {}", config.token_contract);
            println!("   Voting Period: {} blocks", config.voting_period);
            println!("   Execution Delay: {} blocks", config.execution_delay);
            println!("   Quorum: {}%", config.quorum_percentage);
        }
        "simple-token" => {
            let config: SimpleTokenConfig = serde_json::from_str(&config_content)?;
            println!("✅ Valid SimpleTokenConfig:");
            println!("   Name: {}", config.name);
            println!("   Symbol: {}", config.symbol);
            println!("   Decimals: {}", config.decimals);
            println!("   Initial Supply: {}", config.initial_supply);
        }
        "custom-gov" => {
            let config: CustomGovernanceConfig = serde_json::from_str(&config_content)?;
            println!("✅ Valid CustomGovernanceConfig:");
            println!("   Voting Period: {} blocks", config.voting_period_blocks);
            println!("   Execution Delay: {} blocks", config.execution_delay_blocks);
            println!("   Quorum: {}%", config.quorum_percentage);
            println!("   Proposal Threshold: {}", config.proposal_threshold);
        }
        _ => anyhow::bail!("Unknown contract type: {}", contract_type),
    }

    Ok(())
}
