use std::fs;

use anyhow::Result;
use celestia_types::Commitment;
use katana_pipeline::stage::{DAClient, DATip, StateDiffDownloader};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create DAClient
    let da_client = DAClient::new(b"katana").await?;

    // Read the JSON file
    let json_content = fs::read_to_string("blob_ids_2.json")?;
    let blob_ids: Vec<(u64, Commitment)> = serde_json::from_str(&json_content)?;

    // Get the last blob ID (which is the tip)
    let tip = blob_ids.last().unwrap();
    let tip = DATip { block_height: tip.0, commitment: tip.1 };

    // Create StateDiffDownloader
    let downloader = StateDiffDownloader::new(da_client, tip);

    // Fetch state diffs
    let diffs = downloader.fetch().await?;
    assert_eq!(diffs.len(), blob_ids.len());

    Ok(())
}
