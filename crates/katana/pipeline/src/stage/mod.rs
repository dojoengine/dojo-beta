mod sequencing;

use anyhow::Result;
use celestia_rpc::{BlobClient, Client};
use celestia_types::nmt::Namespace;
use celestia_types::Commitment;
use katana_primitives::state::StateUpdates;
use katana_primitives::Felt;
pub use sequencing::Sequencing;
use serde::{Deserialize, Serialize};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[derive(Debug, Clone, Copy)]
pub enum StageId {
    Sequencing,
}

impl std::fmt::Display for StageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StageId::Sequencing => write!(f, "Sequencing"),
        }
    }
}

pub type StageResult = Result<(), Error>;

#[async_trait::async_trait]
pub trait Stage: Send + Sync {
    /// Returns the id which uniquely identifies the stage.
    fn id(&self) -> StageId;

    /// Executes the stage.
    async fn execute(&mut self) -> StageResult;
}

pub type BlockHeight = u64;

#[derive(Debug, Clone)]
pub struct DATip {
    pub block_height: BlockHeight,
    pub commitment: Commitment,
}

pub struct VerifiedStateDiff {
    pub initial_root: Felt,
    pub final_root: Felt,
    pub prev_block_number: Felt,
    pub new_block_number: Felt,
    pub prev_block_hash: Felt,
    pub new_block_hash: Felt,
    pub state_updates: StateUpdates,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct PublishedStateDiff {
    pub prev_state_root: Felt,
    pub state_root: Felt,

    // pointer to the previous state diffs that were published to Celestia
    pub prev_da_height: Option<BlockHeight>,
    pub prev_da_commitment: Option<Commitment>,

    pub content: serde_json::Value,
}

impl From<&PublishedStateDiff> for VerifiedStateDiff {
    fn from(published: &PublishedStateDiff) -> Self {
        todo!()
    }
}

const NODE_URL: &str = "http://celestia-arabica.cartridge.gg";
const AUTH_TOKEN: &str = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.\
                          eyJBbGxvdyI6WyJwdWJsaWMiLCJyZWFkIiwid3JpdGUiLCJhZG1pbiJdfQ.\
                          l26OoOmRmLhKdvvUaeqhSpt2d5eZTWkaixSZeje7XIY";

pub struct DAClient {
    client: Client,
    namespace: Namespace,
}

impl DAClient {
    pub async fn new(namespace: impl AsRef<[u8]>) -> Result<Self> {
        let client = Client::new(NODE_URL, Some(AUTH_TOKEN)).await?;
        let namespace = Namespace::new_v0(namespace.as_ref())?;
        Ok(Self { client, namespace })
    }

    /// * `height` DA block height
    pub async fn retrieve_state_diff(
        &self,
        height: BlockHeight,
        id: Commitment,
    ) -> Result<PublishedStateDiff> {
        let blob = self.client.blob_get(height, self.namespace, id).await?;
        let da_object = postcard::from_bytes::<PublishedStateDiff>(&blob.data)?;
        Ok(da_object)
    }
}

pub struct StateDiffDownloader {
    da_client: DAClient,
    tip: DATip,
}

impl StateDiffDownloader {
    pub fn new(da_client: DAClient, da_tip: DATip) -> Self {
        Self { da_client, tip: da_tip }
    }

    // main loop for fetching the state diffs up to the specified block height
    pub async fn fetch(&self) -> Result<Vec<VerifiedStateDiff>> {
        let mut diffs = Vec::new();

        let mut current_height = Some(self.tip.block_height);
        let mut current_id = Some(self.tip.commitment);

        // reverse downloader, which downloads from tip to 0
        while current_id.is_some() && current_height.is_some() {
            let id = current_id.unwrap();
            let height = current_height.unwrap();

            let diff = self.da_client.retrieve_state_diff(height, id).await?;

            // TODO: verify the state diff
            let verified = VerifiedStateDiff::from(&diff);

            current_height = diff.prev_da_height;
            current_id = diff.prev_da_commitment;
            diffs.push(verified);
        }

        Ok(diffs)
    }
}
