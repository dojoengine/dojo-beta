use celestia_rpc::{BlobClient, Client};
use celestia_types::nmt::Namespace;
use celestia_types::{Blob, Commitment, TxConfig};
use katana_pipeline::os::StarknetOsOutput;
use katana_pipeline::stage::state_diffs::PublishedStateDiff;
use rand::Rng;
use starknet::core::types::Felt;

const NODE_URL: &str = "http://celestia.glihm.xyz";
const AUTH_TOKEN: &str = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.\
                          eyJBbGxvdyI6WyJwdWJsaWMiLCJyZWFkIiwid3JpdGUiLCJhZG1pbiJdfQ.\
                          D05Buftacwpp2pApZwKpdEcqyLFAGw9JRrOBcBGcMO4";

fn random_felt() -> Felt {
    let mut rng = rand::thread_rng();
    Felt::from_bytes_be(&rng.gen::<[u8; 32]>())
}

fn random_os_output(prev_block_number: Felt, new_block_number: Felt) -> StarknetOsOutput {
    StarknetOsOutput {
        new_block_number,
        prev_block_number,
        final_root: random_felt(),
        initial_root: random_felt(),
        prev_block_hash: random_felt(),
        new_block_hash: random_felt(),
        os_program_hash: random_felt(),
        starknet_os_config_hash: random_felt(),
        ..Default::default()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ns = Namespace::new_v0(b"katana")?;
    let client = Client::new(NODE_URL, Some(&AUTH_TOKEN)).await?;

    let mut blob_ids: Vec<(u64, Commitment)> = Vec::new();
    let mut prev_da_height = None;
    let mut prev_da_commitment = None;

    for i in 0..5 {
        let prev_block_number = Felt::from(i - 1);
        let new_block_number = Felt::from(i);

        let os_output = random_os_output(prev_block_number, new_block_number);
        let da_object = PublishedStateDiff {
            prev_state_root: os_output.initial_root,
            state_root: os_output.final_root,
            prev_da_commitment,
            prev_da_height,
            ..Default::default()
        };

        let data = postcard::to_stdvec(&da_object)?;
        let commitment = Commitment::from_blob(ns, 0, &data)?;

        println!("Submitting blob {}", i + 1);

        // submit blob
        let blob = Blob::new(ns, data)?;
        let height = client.blob_submit(&[blob], TxConfig::default()).await?;

        println!("Blob {} submitted at block {height}", i + 1);

        blob_ids.push((height, commitment));
        prev_da_height = Some(height);
        prev_da_commitment = Some(commitment);
    }

    // Serialize and save to JSON file
    let json = serde_json::to_string_pretty(&blob_ids)?;
    std::fs::write(&format!("blob_ids_{}.json", 3), json)?;

    Ok(())
}
