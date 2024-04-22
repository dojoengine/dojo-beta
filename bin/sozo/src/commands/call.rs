use anyhow::Result;
use clap::Args;
use scarb::core::Config;
use starknet::core::types::FieldElement;

use super::options::starknet::StarknetOptions;
use super::options::world::WorldOptions;
use crate::utils;
use tracing::trace;

pub(crate) const LOG_TARGET: &str = "sozo::cli::commands::call";

#[derive(Debug, Args)]
#[command(about = "Call a system with the given calldata.")]
pub struct CallArgs {
    #[arg(help = "The address or the fully qualified name of the contract to call.")]
    pub contract: String,

    #[arg(help = "The name of the entrypoint to call.")]
    pub entrypoint: String,

    #[arg(short, long)]
    #[arg(value_delimiter = ',')]
    #[arg(help = "The calldata to be passed to the entrypoint. Comma separated values e.g., \
                  0x12345,0x69420.")]
    pub calldata: Vec<FieldElement>,

    #[arg(short, long)]
    #[arg(help = "The block ID (could be a hash, a number, 'pending' or 'latest')")]
    pub block_id: Option<String>,

    #[command(flatten)]
    pub starknet: StarknetOptions,

    #[command(flatten)]
    pub world: WorldOptions,
}

impl CallArgs {
    pub fn run(self, config: &Config) -> Result<()> {
        trace!(target: LOG_TARGET, "Contract: {}, Entrypoint: {}, Calldata: {:?}, Block ID: {:?}", 
               self.contract, self.entrypoint, self.calldata, self.block_id);

        let env_metadata = utils::load_metadata_from_config(config)?;
        trace!(target: LOG_TARGET, "Fetched environment metadata");
        
        config.tokio_handle().block_on(async {
            trace!(target: LOG_TARGET, "Initializing world reader from metadata");

            let world_reader =
                utils::world_reader_from_env_metadata(self.world, self.starknet, &env_metadata)
                    .await
                    .unwrap();

            trace!(target: LOG_TARGET, "World reader initialized");
            sozo_ops::call::call(
                world_reader,
                self.contract,
                self.entrypoint,
                self.calldata,
                self.block_id,
            )
            .await
        })
    }
}
