pub mod os;
pub mod stage;
pub mod verification;

use core::future::IntoFuture;

use futures::future::BoxFuture;
use katana_primitives::state::StateUpdates;
use katana_provider::traits::block::BlockWriter;
use tracing::info;
use url::Url;

use crate::stage::{DATip, Stage, StateDiffDownloader};

/// The result of a pipeline execution.
pub type PipelineResult = Result<(), Error>;

/// The future type for [Pipeline]'s implementation of [IntoFuture].
pub type PipelineFut = BoxFuture<'static, PipelineResult>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Stage(#[from] stage::Error),
}

/// Manages the execution of stages.
///
/// The pipeline drives the execution of stages, running each stage to completion in the order they
/// were added.
pub struct Pipeline {
    stages: Vec<Box<dyn Stage>>,
}

impl Pipeline {
    /// Create a new empty pipeline.
    pub fn new() -> Self {
        Self { stages: Vec::new() }
    }

    /// Insert a new stage into the pipeline.
    pub fn add_stage(&mut self, stage: Box<dyn Stage>) {
        self.stages.push(stage);
    }

    /// Start the pipeline.
    pub async fn run(&mut self) -> PipelineResult {
        for stage in &mut self.stages {
            info!(id = %stage.id(), "Executing stage");
            stage.execute().await?;
        }
        Ok(())
    }
}

impl IntoFuture for Pipeline {
    type Output = PipelineResult;
    type IntoFuture = PipelineFut;

    fn into_future(mut self) -> Self::IntoFuture {
        Box::pin(async move { self.run().await })
    }
}

// 1. fetch state diffs
// 2. verify state diffs
// 3. apply state diffs
pub async fn da_sync(
    provider: impl BlockWriter,
    downloader: StateDiffDownloader,
) -> anyhow::Result<()> {
    let diffs = downloader.fetch().await?;
    let Some(head) = diffs.last() else { panic!("no state diffs found") };

    let latest_number = head.new_block_number;
    let latest_hash = head.new_block_hash;
    let latest_root = head.final_root;

    // todo: we should sync with p2p to get the headers
    let merged_updates = diffs.into_iter().fold(StateUpdates::default(), |mut batch, diff| {
        batch.merge(diff.state_updates);
        batch
    });

    // apply to storage
    // provider.insert_block_with_states_and_receipts(block, merged_updates, receipts, executions);

    Ok(())
}

pub async fn retrieve_da_tip(peer: Url) -> anyhow::Result<DATip> {
    todo!()
}

// NAME: my_celes_key
// ADDRESS: celestia18d0ehf33c47zxw3dchjkxhtgg2lzpkv9xr96gd
// MNEMONIC (save this somewhere safe!!!):
// title clown increase shell labor spring round enforce vault inner equal mixed belt power cigar
// bird observe youth pair outer assume supply fuel exist
