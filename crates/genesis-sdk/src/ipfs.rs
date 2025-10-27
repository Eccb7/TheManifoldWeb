//! IPFS publishing utilities for genome storage.

use anyhow::{Context, Result};
use reqwest::multipart;
use serde::{Deserialize, Serialize};

/// Response from IPFS add API.
#[derive(Debug, Deserialize)]
struct IpfsAddResponse {
    #[serde(rename = "Hash")]
    hash: String,
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Size")]
    size: String,
}

/// Publish arbitrary serializable data to IPFS.
///
/// # Arguments
/// * `obj` - Any serializable object to publish
/// * `ipfs_api` - Base URL of IPFS HTTP API (e.g., "http://127.0.0.1:5001")
///
/// # Returns
/// The IPFS CID (Content Identifier) of the published object.
///
/// # Example
/// ```no_run
/// use genesis_sdk::publish_to_ipfs;
/// use manifold_protocol::Genome;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let genome = Genome::new("".to_string(), vec![1, 2, 3]);
///     let cid = publish_to_ipfs(&genome, "http://127.0.0.1:5001").await?;
///     println!("Published genome with CID: {}", cid);
///     Ok(())
/// }
/// ```
pub async fn publish_to_ipfs<T: Serialize>(obj: &T, ipfs_api: &str) -> Result<String> {
    let json_data =
        serde_json::to_vec(obj).context("Failed to serialize object to JSON")?;

    let form = multipart::Form::new().part(
        "file",
        multipart::Part::bytes(json_data)
            .file_name("genome.json")
            .mime_str("application/json")?,
    );

    let client = reqwest::Client::new();
    let url = format!("{}/api/v0/add", ipfs_api);

    let response = client
        .post(&url)
        .multipart(form)
        .send()
        .await
        .context("Failed to send request to IPFS API")?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response
            .text()
            .await
            .unwrap_or_else(|_| "Unable to read response body".to_string());
        anyhow::bail!("IPFS API error ({}): {}", status, body);
    }

    let ipfs_response: IpfsAddResponse = response
        .json()
        .await
        .context("Failed to parse IPFS response")?;

    Ok(ipfs_response.hash)
}

/// Publish raw bytes to IPFS.
///
/// Useful for publishing WASM modules directly.
///
/// # Arguments
/// * `data` - Raw bytes to publish
/// * `ipfs_api` - Base URL of IPFS HTTP API
/// * `filename` - Optional filename for the content
///
/// # Returns
/// The IPFS CID of the published data.
pub async fn publish_bytes_to_ipfs(
    data: &[u8],
    ipfs_api: &str,
    filename: Option<&str>,
) -> Result<String> {
    let form = multipart::Form::new().part(
        "file",
        multipart::Part::bytes(data.to_vec())
            .file_name(filename.unwrap_or("data.bin").to_string()),
    );

    let client = reqwest::Client::new();
    let url = format!("{}/api/v0/add", ipfs_api);

    let response = client
        .post(&url)
        .multipart(form)
        .send()
        .await
        .context("Failed to send request to IPFS API")?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response
            .text()
            .await
            .unwrap_or_else(|_| "Unable to read response body".to_string());
        anyhow::bail!("IPFS API error ({}): {}", status, body);
    }

    let ipfs_response: IpfsAddResponse = response
        .json()
        .await
        .context("Failed to parse IPFS response")?;

    Ok(ipfs_response.hash)
}

#[cfg(test)]
mod tests {
    use super::*;
    use manifold_protocol::Genome;

    #[tokio::test]
    #[ignore] // Requires running IPFS daemon
    async fn test_publish_genome_to_ipfs() {
        let genome = Genome::new("QmTest".to_string(), vec![1, 2, 3, 4, 5]);

        let result = publish_to_ipfs(&genome, "http://127.0.0.1:5001").await;

        match result {
            Ok(cid) => {
                println!("Successfully published genome with CID: {}", cid);
                assert!(!cid.is_empty());
                assert!(cid.starts_with("Qm") || cid.starts_with("bafy"));
            }
            Err(e) => {
                eprintln!("Test requires running IPFS daemon: {}", e);
                eprintln!("Run: ipfs daemon");
            }
        }
    }

    #[tokio::test]
    #[ignore] // Requires running IPFS daemon
    async fn test_publish_bytes_to_ipfs() {
        let data = b"Hello, Manifold!";

        let result =
            publish_bytes_to_ipfs(data, "http://127.0.0.1:5001", Some("hello.txt")).await;

        match result {
            Ok(cid) => {
                println!("Successfully published bytes with CID: {}", cid);
                assert!(!cid.is_empty());
            }
            Err(e) => {
                eprintln!("Test requires running IPFS daemon: {}", e);
            }
        }
    }
}
