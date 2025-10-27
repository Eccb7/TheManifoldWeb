//! # Observer Client
//!
//! Read-only monitoring client for The Manifold Web.
//!
//! Subscribes to network events via gossipsub and displays agent activity.
//! Includes dead reckoning for smooth agent position prediction to compensate
//! for network latency.
//!
//! Future versions will include 3D visualization using wgpu/rend3.

mod dead_reckoning;
mod observer;

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

    info!("👁️  Starting Manifold Observer Client...");

    // Create and run observer
    let mut observer = observer::Observer::new().await?;
    observer.run().await?;

    Ok(())
}
