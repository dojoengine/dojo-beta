use anyhow::Result;
use celestia_rpc::{BlobClient, Client};
use celestia_types::nmt::Namespace;
pub use celestia_types::Commitment;
use katana_core::backend::storage::{Blockchain, Database};
use katana_primitives::state::{StateUpdates, StateUpdatesWithDeclaredClasses};
use katana_primitives::Felt;
use katana_provider::traits::state_update::StateUpdateWriter;
use num_traits::ToPrimitive;
use serde::{Deserialize, Serialize};
use tracing::trace;
use url::Url;

use crate::os::StarknetOsOutput;
use crate::stage::{Stage, StageId, StageResult};

/// Data availability block height.
pub type BlockHeight = u64;

#[derive(Debug, Clone)]
pub struct DATip {
    pub block_height: BlockHeight,
    pub commitment: Commitment,
}

impl Default for DATip {
    fn default() -> Self {
        Self { block_height: 0, commitment: Commitment([0u8; 32]) }
    }
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

    pub content: StarknetOsOutput,
}

impl From<&PublishedStateDiff> for VerifiedStateDiff {
    fn from(published: &PublishedStateDiff) -> Self {
        Self {
            initial_root: published.prev_state_root,
            final_root: published.state_root,
            prev_block_number: published.content.prev_block_number,
            new_block_number: published.content.new_block_number,
            prev_block_hash: published.content.prev_block_hash,
            new_block_hash: published.content.new_block_hash,
            state_updates: Default::default(),
        }
    }
}

pub struct CelestiaClient {
    client: Client,
    namespace: Namespace,
}

impl CelestiaClient {
    pub async fn new(
        addr: Url,
        auth_token: Option<String>,
        namespace: impl AsRef<[u8]>,
    ) -> Result<Self> {
        let namespace = Namespace::new_v0(namespace.as_ref())?;
        let client = if let Some(token) = auth_token {
            Client::new(addr.as_str(), Some(&token)).await?
        } else {
            Client::new(addr.as_str(), None).await?
        };

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

pub struct Downloader {
    da_client: CelestiaClient,
    tip: DATip,
}

impl Downloader {
    pub fn new(da_client: CelestiaClient, da_tip: DATip) -> Self {
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

            trace!(target: "pipeline", %height, "Retrieving state diff");

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

pub struct StateDiffs {
    downloader: Downloader,
    provider: Blockchain,
}

impl StateDiffs {
    pub fn new(downloader: Downloader, provider: Blockchain) -> Self {
        Self { downloader, provider }
    }
}

#[async_trait::async_trait]
impl Stage for StateDiffs {
    fn id(&self) -> StageId {
        StageId::StateDiffs
    }

    #[tracing::instrument(skip_all, name = "Stage", fields(id = %self.id()))]
    async fn execute(&mut self) -> StageResult {
        let diffs = self.downloader.fetch().await?;

        for diff in diffs {
            let block_number = diff.new_block_number.to_u64().expect("Fit into u64");
            let state_updates = diff.state_updates;

            let states = StateUpdatesWithDeclaredClasses { state_updates, ..Default::default() };
            self.provider.provider().apply_state_updates(block_number, states)?;
        }

        Ok(())
    }
}

// NAME: my_celes_key
// ADDRESS: celestia18d0ehf33c47zxw3dchjkxhtgg2lzpkv9xr96gd
// MNEMONIC (save this somewhere safe!!!):
// title clown increase shell labor spring round enforce vault inner equal mixed belt power cigar
// bird observe youth pair outer assume supply fuel exist
