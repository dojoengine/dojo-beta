use std::fs;

use anyhow::Result;
use celestia_types::Commitment;
use katana_pipeline::stage::state_diffs::{CelestiaClient, DATip, Downloader};
use url::Url;

// --- build the syncing stage

// const NODE_URL: &str = "http://celestia-arabica.cartridge.gg";
// const AUTH_TOKEN: &str = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.\
//                           eyJBbGxvdyI6WyJwdWJsaWMiLCJyZWFkIiwid3JpdGUiLCJhZG1pbiJdfQ.\
//                           l26OoOmRmLhKdvvUaeqhSpt2d5eZTWkaixSZeje7XIY";

const NODE_URL: &str = "http://celestia-arabica.cartridge.gg";
const AUTH_TOKEN: &str = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.\
                          eyJBbGxvdyI6WyJwdWJsaWMiLCJyZWFkIiwid3JpdGUiLCJhZG1pbiJdfQ.\
                          l26OoOmRmLhKdvvUaeqhSpt2d5eZTWkaixSZeje7XIY";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create DAClient
    let url = Url::parse(NODE_URL)?;
    let token = Some(AUTH_TOKEN.to_string());
    let da_client = CelestiaClient::new(url, token, "katana").await?;

    // Read the JSON file
    let json_content = fs::read_to_string("../../../blob_ids_2.json")?;
    let blob_ids: Vec<(u64, Commitment)> = serde_json::from_str(&json_content)?;

    // Get the last blob ID (which is the tip)
    let tip = blob_ids.last().unwrap();
    let tip = DATip { block_height: tip.0, commitment: tip.1 };

    // Create StateDiffDownloader
    let downloader = Downloader::new(da_client, tip);

    // Fetch state diffs
    let diffs = downloader.fetch().await?;
    assert_eq!(diffs.len(), blob_ids.len());

    Ok(())
}
