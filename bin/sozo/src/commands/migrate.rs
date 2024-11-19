use anyhow::{Context, Result};
use clap::Args;
use colored::Colorize;
use dojo_utils::{self, TxnConfig};
use dojo_world::contracts::WorldContract;
use dojo_world::metadata::IpfsMetadataService;
use scarb::core::{Config, Workspace};
use sozo_ops::migrate::{Migration, MigrationResult};
use sozo_ops::migration_ui::MigrationUi;
use sozo_scarbext::WorkspaceExt;
use starknet::core::utils::parse_cairo_short_string;
use starknet::providers::Provider;
use tabled::settings::Style;
use tabled::{Table, Tabled};
use tracing::trace;

use super::options::account::AccountOptions;
use super::options::ipfs::IpfsOptions;
use super::options::starknet::StarknetOptions;
use super::options::transaction::TransactionOptions;
use super::options::world::WorldOptions;
use crate::utils;

#[derive(Debug, Clone, Args)]
pub struct MigrateArgs {
    #[command(flatten)]
    pub transaction: TransactionOptions,

    #[command(flatten)]
    pub world: WorldOptions,

    #[command(flatten)]
    pub starknet: StarknetOptions,

    #[command(flatten)]
    pub account: AccountOptions,

    #[command(flatten)]
    pub ipfs: IpfsOptions,
}

impl MigrateArgs {
    /// Runs the migration.
    pub fn run(self, config: &Config) -> Result<()> {
        trace!(args = ?self);

        let ws = scarb::ops::read_workspace(config.manifest_path(), config)?;
        ws.profile_check()?;
        ws.ensure_profile_artifacts()?;

        let MigrateArgs { world, starknet, account, ipfs, .. } = self;

        config.tokio_handle().block_on(async {
            print_banner(&ws, &starknet).await?;

            let mut spinner = MigrationUi::new(Some("Evaluating world diff..."));

            let (world_diff, account, rpc_url) = utils::get_world_diff_and_account(
                account,
                starknet,
                world,
                &ws,
                &mut Some(&mut spinner),
            )
            .await?;

            let world_address = world_diff.world_info.address;
            let profile_config = ws.load_profile_config()?;

            let mut txn_config: TxnConfig = self.transaction.try_into()?;
            txn_config.wait = true;

            let migration = Migration::new(
                world_diff,
                WorldContract::new(world_address, &account),
                txn_config,
                ws.load_profile_config()?,
                rpc_url,
            );

            let MigrationResult { manifest, has_changes } =
                migration.migrate(&mut spinner).await.context("Migration failed.")?;

            let ipfs_url = ipfs.url(profile_config.env.as_ref());
            let ipfs_username = ipfs.username(profile_config.env.as_ref());
            let ipfs_password = ipfs.password(profile_config.env.as_ref());

            let mut metadata_upload_text = String::new();

            if ipfs_url.is_some() && ipfs_username.is_some() && ipfs_password.is_some() {
                let mut metadata_service = IpfsMetadataService::new(
                    &ipfs_url.unwrap(),
                    &ipfs_username.unwrap(),
                    &ipfs_password.unwrap(),
                )?;

                migration
                    .upload_metadata(&mut spinner, &mut metadata_service)
                    .await
                    .context("Metadata upload failed.")?;
            } else {
                metadata_upload_text = "\nMetadata: No IPFS credentials has been found => \
                                        metadata upload has been ignored."
                    .to_string();
            };

            spinner.update_text("Writing manifest...");
            ws.write_manifest_profile(manifest).context("🪦 Failed to write manifest.")?;

            let colored_address = format!("{:#066x}", world_address).green();

            let (symbol, end_text) = if has_changes {
                (
                    "⛩️ ",
                    format!(
                        "Migration successful with world at address {}{metadata_upload_text}",
                        colored_address
                    ),
                )
            } else {
                (
                    "🪨 ",
                    format!(
                        "No changes for world at address {:#066x}{metadata_upload_text}",
                        world_address
                    ),
                )
            };

            spinner.stop_and_persist_boxed(symbol, end_text);

            Ok(())
        })
    }
}

#[derive(Debug, Tabled)]
pub struct Banner {
    pub profile: String,
    pub chain_id: String,
    pub rpc_url: String,
}

/// Prints the migration banner.
async fn print_banner(ws: &Workspace<'_>, starknet: &StarknetOptions) -> Result<()> {
    let profile_config = ws.load_profile_config()?;
    let (provider, rpc_url) = starknet.provider(profile_config.env.as_ref())?;

    let chain_id = provider.chain_id().await?;
    let chain_id =
        parse_cairo_short_string(&chain_id).with_context(|| "Cannot parse chain_id as string")?;

    let banner = Banner {
        profile: ws.current_profile().expect("Scarb profile should be set.").to_string(),
        chain_id,
        rpc_url,
    };

    println!();
    println!("{}", Table::new(&[banner]).with(Style::psql()));
    println!();

    Ok(())
}
