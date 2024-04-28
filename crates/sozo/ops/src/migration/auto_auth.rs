use anyhow::Result;
use dojo_world::contracts::{cairo_utils, WorldContract};
use dojo_world::manifest::BaseManifest;
use dojo_world::migration::TxnConfig;
use scarb::core::Workspace;
use scarb_ui::Ui;
use starknet::accounts::ConnectedAccount;

use super::ui::MigrationUi;
use super::MigrationOutput;
use crate::auth::{grant_writer, ModelContract};

pub async fn auto_authorize<'a, A>(
    ws: &Workspace<'_>,
    world: &'a WorldContract<A>,
    txn_config: &TxnConfig,
    local_manifest: &BaseManifest,
    migration_output: &MigrationOutput,
) -> Result<()>
where
    A: ConnectedAccount + Sync + Send + 'static,
{
    let ui = ws.config().ui();

    ui.print(" ");
    ui.print_step(7, "🌐", "Authorizing Models to Systems (based on overlay)...");
    ui.print(" ");
    let models_contracts = compute_models_contracts(&ui, local_manifest, migration_output)?;
    grant_writer(world, models_contracts, *txn_config).await
}

pub fn compute_models_contracts(
    ui: &Ui,
    local_manifest: &BaseManifest,
    migration_output: &MigrationOutput,
) -> Result<Vec<crate::auth::ModelContract>> {
    let mut res = vec![];
    let local_contracts = &local_manifest.contracts;

    // from all the contracts that where migrated successfully
    for migrated_contract in migration_output.contracts.iter().flatten() {
        // find that contract from local_manifest based on its name
        let contract = local_contracts
            .iter()
            .find(|c| migrated_contract.name == c.name)
            .expect("we know this contract exists");

        ui.print_sub(format!(
            "Authorizing {} for Models: {:?}",
            contract.name, contract.inner.writes
        ));

        // read all the models that its supposed to write to and
        for model in &contract.inner.writes {
            let model = cairo_utils::str_to_felt(model)?;
            let contract_addr_str = format!("{:#x}", migrated_contract.contract_address);

            res.push(ModelContract { model, contract: contract_addr_str });
        }
    }

    Ok(res)
}
