//! Command-line tool for archiving genesis blocks to Arweave.
//!
//! # Usage
//!
//! ```bash
//! # Create and archive a new genesis block
//! cargo run --bin archive-genesis -- \
//!     --genomes QmGenome1,QmGenome2,QmGenome3 \
//!     --energy 1000 \
//!     --creator "your-identifier" \
//!     --wallet path/to/arweave-wallet.json
//!
//! # Archive with custom world parameters
//! cargo run --bin archive-genesis -- \
//!     --genomes QmGenome1 \
//!     --energy 500 \
//!     --sector-size 200 \
//!     --mutation-rate 0.05 \
//!     --creator "creator-id" \
//!     --wallet wallet.json
//! ```

use anyhow::Result;
use clap::Parser;
use manifold_archiver::{Archiver, GenesisBlock, WorldConfig};
use tracing::{info, Level};
use tracing_subscriber;

#[derive(Parser, Debug)]
#[command(name = "archive-genesis")]
#[command(about = "Archive a genesis block to Arweave", long_about = None)]
struct Args {
    /// Comma-separated list of genome CIDs (IPFS)
    #[arg(long, value_delimiter = ',')]
    genomes: Vec<String>,
    
    /// Initial energy per agent
    #[arg(long, default_value = "1000")]
    energy: u64,
    
    /// Creator identifier
    #[arg(long)]
    creator: String,
    
    /// Path to Arweave wallet JSON file
    #[arg(long, default_value = "arweave-wallet.json")]
    wallet: String,
    
    /// Arweave gateway URL
    #[arg(long, default_value = "https://arweave.net")]
    gateway: String,
    
    /// Sector size for world partitioning
    #[arg(long, default_value = "100.0")]
    sector_size: f32,
    
    /// Maximum agents per sector
    #[arg(long, default_value = "1000")]
    max_agents: u64,
    
    /// Energy decay rate (0.0 to 1.0)
    #[arg(long, default_value = "0.01")]
    decay_rate: f64,
    
    /// Mutation rate for genetic algorithm (0.0 to 1.0)
    #[arg(long, default_value = "0.01")]
    mutation_rate: f64,
    
    /// Energy threshold for replication
    #[arg(long, default_value = "500")]
    replication_threshold: u64,
    
    /// Output genesis block JSON to file instead of uploading
    #[arg(long)]
    output: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();
    
    let args = Args::parse();
    
    info!("🚀 The Manifold Web - Genesis Block Archiver");
    info!("============================================");
    
    // Validate inputs
    if args.genomes.is_empty() {
        anyhow::bail!("At least one genome CID must be provided");
    }
    
    if args.mutation_rate < 0.0 || args.mutation_rate > 1.0 {
        anyhow::bail!("Mutation rate must be between 0.0 and 1.0");
    }
    
    if args.decay_rate < 0.0 || args.decay_rate > 1.0 {
        anyhow::bail!("Decay rate must be between 0.0 and 1.0");
    }
    
    // Create world configuration
    let world_config = WorldConfig {
        sector_size: args.sector_size,
        max_agents_per_sector: args.max_agents,
        energy_decay_rate: args.decay_rate,
        mutation_rate: args.mutation_rate,
        replication_threshold: args.replication_threshold,
    };
    
    // Create genesis block
    info!("📦 Creating genesis block...");
    let genesis = GenesisBlock::new(
        args.genomes.clone(),
        args.energy,
        world_config,
        args.creator,
    );
    
    info!("  Genesis ID: {}", genesis.id);
    info!("  Genomes: {:?}", genesis.genesis_genomes);
    info!("  Initial Energy: {}", genesis.initial_energy);
    info!("  Hash: {}", hex::encode(genesis.hash));
    info!("  Verified: {}", genesis.verify());
    
    // Output to file if requested
    if let Some(output_path) = args.output {
        let json = genesis.to_json()?;
        std::fs::write(&output_path, json)?;
        info!("✅ Genesis block written to: {}", output_path);
        return Ok(());
    }
    
    // Otherwise, archive to Arweave
    info!("🌐 Connecting to Arweave gateway: {}", args.gateway);
    let archiver = Archiver::new(&args.gateway, &args.wallet).await?;
    
    info!("📤 Archiving genesis block to Arweave...");
    let tx_id = archiver.archive_genesis_block(&genesis).await?;
    
    info!("✅ Genesis block archived successfully!");
    info!("   Transaction ID: {}", tx_id);
    info!("   View at: https://arweave.net/{}", tx_id);
    info!("   ViewBlock: https://viewblock.io/arweave/tx/{}", tx_id);
    
    // Output instructions
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🎉 Genesis Block Archived!");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("\nTransaction ID: {}", tx_id);
    println!("\nTo use this genesis in your simulation:");
    println!("  1. Note the transaction ID above");
    println!("  2. Start manifold-node with: --genesis {}", tx_id);
    println!("  3. The node will fetch and verify the genesis block");
    println!("\nTo view the genesis block:");
    println!("  https://arweave.net/{}", tx_id);
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    
    Ok(())
}
