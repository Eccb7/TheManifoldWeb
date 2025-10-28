//! # Manifold Node
//!
//! The core network node that orchestrates agent lifecycle, evolution, and
//! peer-to-peer communication in The Manifold Web.

mod behaviour;
mod network;
mod simulation;

use anyhow::Result;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    info!("🌐 Starting Manifold Node...");

    // Initialize network
    let network = network::Network::new().await?;

    info!("📡 Network initialized");
    info!("🔑 Peer ID: {}", network.local_peer_id());

    // Start the simulation and network event loop
    network.run().await?;

    Ok(())
}
