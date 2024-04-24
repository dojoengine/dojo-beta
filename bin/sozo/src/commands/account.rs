use std::path::PathBuf;

use anyhow::Result;
use clap::{Args, Subcommand};
use scarb::core::Config;
use sozo_ops::account;
use starknet::signers::LocalWallet;
use starknet_crypto::FieldElement;

use super::options::fee::FeeOptions;
use super::options::signer::SignerOptions;
use super::options::starknet::StarknetOptions;
use crate::utils;
use tracing::trace;

pub(crate) const LOG_TARGET: &str = "sozo::cli::commands::account";

#[derive(Debug, Args)]
pub struct AccountArgs {
    #[clap(subcommand)]
    command: AccountCommand,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Subcommand)]
pub enum AccountCommand {
    #[clap(about = "Create a new account configuration without actually deploying.")]
    New {
        #[clap(flatten)]
        signer: SignerOptions,

        #[clap(long, short, help = "Overwrite the account config file if it already exists")]
        force: bool,

        #[clap(help = "Path to save the account config file")]
        file: PathBuf,
    },

    #[clap(about = "Deploy account contract with a DeployAccount transaction.")]
    Deploy {
        #[clap(flatten)]
        starknet: StarknetOptions,

        #[clap(flatten)]
        signer: SignerOptions,

        #[clap(flatten)]
        fee: FeeOptions,

        #[clap(long, help = "Simulate the transaction only")]
        simulate: bool,

        #[clap(long, help = "Provide transaction nonce manually")]
        nonce: Option<FieldElement>,

        #[clap(
            long,
            env = "STARKNET_POLL_INTERVAL",
            default_value = "1000",
            help = "Transaction result poll interval in milliseconds"
        )]
        poll_interval: u64,

        #[clap(help = "Path to the account config file")]
        file: PathBuf,

        #[clap(long, help = "Don't wait for user confirmation")]
        no_confirmation: bool,
    },

    #[clap(about = "Fetch account config from an already deployed account contract.")]
    Fetch {
        #[clap(flatten)]
        starknet: StarknetOptions,

        #[clap(long, help = "Overwrite the file if it already exists")]
        force: bool,

        #[clap(long, help = "Path to save the account config file")]
        output: PathBuf,

        #[clap(help = "Contract address")]
        address: FieldElement,
    },
}

impl AccountArgs {
    pub fn run(self, config: &Config) -> Result<()> {
        trace!(target: LOG_TARGET, command=?self.command, "Executing command.");
        let env_metadata = utils::load_metadata_from_config(config)?;

        config.tokio_handle().block_on(async {
            match self.command {
                AccountCommand::New { signer, force, file } => {
                    trace!(
                        target: LOG_TARGET,
                        ?signer,
                        force,
                        ?file,
                        "Executing New command."
                    );
                    let signer: LocalWallet = signer.signer(env_metadata.as_ref(), false)?;
                    account::new(signer, force, file).await
                }
                AccountCommand::Deploy {
                    starknet,
                    signer,
                    fee,
                    simulate,
                    nonce,
                    poll_interval,
                    file,
                    no_confirmation,
                } => {
                    trace!(
                        target: LOG_TARGET,
                        ?starknet,
                        ?signer,
                        ?fee,
                        simulate,
                        ?nonce,
                        poll_interval,
                        ?file,
                        no_confirmation,
                        "Executing Deploy command."
                    );
                    let provider = starknet.provider(env_metadata.as_ref())?;
                    let signer: LocalWallet = signer.signer(env_metadata.as_ref(), false)?;
                    let fee_setting = fee.into_setting()?;
                    account::deploy(
                        provider,
                        signer,
                        fee_setting,
                        simulate,
                        nonce,
                        poll_interval,
                        file,
                        no_confirmation,
                    )
                    .await
                }
                AccountCommand::Fetch { starknet, force, output, address } => {
                    trace!(
                        target: LOG_TARGET,
                        ?starknet,
                        force,
                        ?output,
                        ?address,
                        "Executing Fetch command."
                    );
                    let provider = starknet.provider(env_metadata.as_ref())?;
                    account::fetch(provider, force, output, address).await
                }
            }
        })
    }
}