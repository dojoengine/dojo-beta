use account_sdk::account::session::hash::{AllowedMethod, Session};
use account_sdk::account::session::SessionAccount;
use account_sdk::signers::HashSigner;
use anyhow::{bail, Context, Result};
use camino::{Utf8Path, Utf8PathBuf};
use dojo_world::contracts::naming::get_name_from_tag;
use dojo_world::manifest::{BaseManifest, Class, DojoContract, Manifest};
use dojo_world::migration::strategy::generate_salt;
use scarb::core::Config;
use slot::session::Policy;
use starknet::core::types::contract::{AbiEntry, StateMutability};
use starknet::core::types::StarknetError::ContractNotFound;
use starknet::core::types::{BlockId, BlockTag, Felt};
use starknet::core::utils::{cairo_short_string_to_felt, get_contract_address};
use starknet::macros::{felt, short_string};
use starknet::providers::Provider;
use starknet::providers::ProviderError::StarknetError;
use starknet::signers::SigningKey;
use starknet_crypto::poseidon_hash_single;
use tracing::{trace, warn};
use url::Url;

use super::WorldAddressOrName;

// This type comes from account_sdk, which doesn't derive Debug.
#[allow(missing_debug_implementations)]
pub type ControllerSessionAccount<P> = SessionAccount<P, SigningKey, SigningKey>;

/// Create a new Catridge Controller account based on session key.
#[tracing::instrument(
    name = "create_controller",
    skip(rpc_url, provider, world_addr_or_name, config)
)]
pub async fn create_controller<P>(
    // Ideally we can get the url from the provider so we dont have to pass an extra url param here
    rpc_url: Url,
    provider: P,
    // Use to either specify the world address or compute the world address from the world name
    world_addr_or_name: WorldAddressOrName,
    config: &Config,
) -> Result<ControllerSessionAccount<P>>
where
    P: Provider,
    P: Send + Sync,
{
    let chain_id = provider.chain_id().await?;
    let credentials = slot::credential::Credentials::load()?;

    let username = credentials.account.id;
    let contract_address = credentials.account.contract_address;

    trace!(
        %username,
        chain = format!("{chain_id:#x}"),
        address = format!("{contract_address:#x}"),
        "Creating Controller session account"
    );

    // Check if the session exists, if not create a new one
    let session_details = match slot::session::get(chain_id)? {
        Some(session) => {
            trace!(expires_at = %session.expires_at, policies = session.policies.len(), "Found existing session.");

            // Perform policies diff check. For security reasons, we will always create a new
            // session here if the current policies are different from the existing
            // session.
            //
            // TODO(kariy): maybe don't need to update if current policies is a
            // subset of the existing policies.
            let policies = collect_policies(world_addr_or_name, contract_address, config)?;

            if policies != session.policies {
                trace!(
                    new_policies = policies.len(),
                    existing_policies = session.policies.len(),
                    "Policies have changed. Creating new session."
                );

                let session = slot::session::create(rpc_url.clone(), &policies).await?;
                slot::session::store(chain_id, &session)?;
                session
            } else {
                session
            }
        }

        // Create a new session if not found
        None => {
            trace!(%username, chain = format!("{chain_id:#}"), "Creating new session.");
            let policies = collect_policies(world_addr_or_name, contract_address, config)?;
            let session = slot::session::create(rpc_url.clone(), &policies).await?;
            slot::session::store(chain_id, &session)?;
            session
        }
    };

    let methods = session_details
        .policies
        .into_iter()
        .map(|p| AllowedMethod::new(p.target, &p.method))
        .collect::<Result<Vec<AllowedMethod>, _>>()?;

    // Copied from `account-wasm` <https://github.com/cartridge-gg/controller/blob/0dd4dd6cbc5fcd3b9a1fd8d63dc127f6312b733f/packages/account-wasm/src/lib.rs#L78-L88>
    let guardian = SigningKey::from_secret_scalar(short_string!("CARTRIDGE_GUARDIAN"));
    let signer = SigningKey::from_secret_scalar(session_details.credentials.private_key);
    // TODO(kariy): make `expires_at` a `u64` type in the session struct
    let expires_at = session_details.expires_at.parse::<u64>()?;
    let session = Session::new(methods, expires_at, &signer.signer())?;

    // make sure account exist on the provided chain, if not, we deploy it first before proceeding
    deploy_account_if_not_exist(rpc_url, &provider, chain_id, contract_address, &username)
        .await
        .with_context(|| format!("Deploying Controller account on chain {chain_id}"))?;

    let session_account = SessionAccount::new(
        provider,
        signer,
        guardian,
        contract_address,
        chain_id,
        session_details.credentials.authorization,
        session,
    );

    Ok(session_account)
}

/// Policies are the building block of a session key. It's what defines what methods are allowed for
/// an external signer to execute using the session key.
///
/// This function collect all the contracts' methods in the current project according to the
/// project's base manifest ( `/manifests/<profile>/base` ) and convert them into policies.
fn collect_policies(
    world_addr_or_name: WorldAddressOrName,
    user_address: Felt,
    config: &Config,
) -> Result<Vec<Policy>> {
    let root_dir = config.root();
    let manifest = get_project_base_manifest(root_dir, config.profile().as_str())?;
    let policies =
        collect_policies_from_base_manifest(world_addr_or_name, user_address, root_dir, manifest)?;
    trace!(policies_count = policies.len(), "Extracted policies from project.");
    Ok(policies)
}

fn get_project_base_manifest(root_dir: &Utf8Path, profile: &str) -> Result<BaseManifest> {
    let mut manifest_path = root_dir.to_path_buf();
    manifest_path.extend(["manifests", profile, "base"]);
    Ok(BaseManifest::load_from_path(&manifest_path)?)
}

fn collect_policies_from_base_manifest(
    world_address: WorldAddressOrName,
    user_address: Felt,
    base_path: &Utf8Path,
    manifest: BaseManifest,
) -> Result<Vec<Policy>> {
    let mut policies: Vec<Policy> = Vec::new();
    let base_path: Utf8PathBuf = base_path.to_path_buf();

    // compute the world address here if it's a name
    let world_address = get_dojo_world_address(world_address, &manifest)?;

    // get methods from all project contracts
    for contract in manifest.contracts {
        let contract_address = get_dojo_contract_address(world_address, &contract, &manifest.base);
        let abis = contract.inner.abi.unwrap().load_abi_string(&base_path)?;
        let abis = serde_json::from_str::<Vec<AbiEntry>>(&abis)?;
        policies_from_abis(&mut policies, &contract.inner.tag, contract_address, &abis);
    }

    // get method from world contract
    let abis = manifest.world.inner.abi.unwrap().load_abi_string(&base_path)?;
    let abis = serde_json::from_str::<Vec<AbiEntry>>(&abis)?;
    policies_from_abis(&mut policies, "world", world_address, &abis);

    // special policy for sending declare tx
    // corresponds to [account_sdk::account::DECLARATION_SELECTOR]
    let method = "__declare_transaction__".to_string();
    policies.push(Policy { target: user_address, method });
    trace!("Adding declare transaction policy");

    // for deploying using udc
    let method = "deployContract".to_string();
    const UDC_ADDRESS: Felt =
        felt!("0x041a78e741e5af2fec34b695679bc6891742439f7afb8484ecd7766661ad02bf");
    policies.push(Policy { target: UDC_ADDRESS, method });
    trace!("Adding UDC deployment policy");

    Ok(policies)
}

/// Recursively extract methods and convert them into policies from the all the
/// ABIs in the project.
fn policies_from_abis(
    policies: &mut Vec<Policy>,
    contract_tag: &str,
    contract_address: Felt,
    entries: &[AbiEntry],
) {
    for entry in entries {
        match entry {
            AbiEntry::Function(f) => {
                // we only create policies for non-view functions
                if let StateMutability::External = f.state_mutability {
                    let policy = Policy { target: contract_address, method: f.name.to_string() };
                    trace!(tag = contract_tag, target = format!("{:#x}", policy.target), method = %policy.method, "Adding policy");
                    policies.push(policy);
                }
            }

            AbiEntry::Interface(i) => {
                policies_from_abis(policies, contract_tag, contract_address, &i.items)
            }

            _ => {}
        }
    }
}

fn get_dojo_contract_address(
    world_address: Felt,
    contract: &Manifest<DojoContract>,
    base_class: &Manifest<Class>,
) -> Felt {
    // The `base_class_hash` field in the Contract's base manifest is initially set to ZERO,
    // so we need to use the `class_hash` from the base class manifest instead.
    let base_class_hash = if contract.inner.base_class_hash != Felt::ZERO {
        contract.inner.base_class_hash
    } else {
        base_class.inner.class_hash
    };

    if let Some(address) = contract.inner.address {
        address
    } else {
        let salt = generate_salt(&get_name_from_tag(&contract.inner.tag));
        get_contract_address(salt, base_class_hash, &[], world_address)
    }
}

fn get_dojo_world_address(
    world_address: WorldAddressOrName,
    manifest: &BaseManifest,
) -> Result<Felt> {
    match world_address {
        WorldAddressOrName::Address(addr) => Ok(addr),
        WorldAddressOrName::Name(name) => {
            let seed = cairo_short_string_to_felt(&name).context("Failed to parse World name.")?;
            let salt = poseidon_hash_single(seed);
            let address = get_contract_address(
                salt,
                manifest.world.inner.original_class_hash,
                &[manifest.base.inner.original_class_hash],
                Felt::ZERO,
            );
            Ok(address)
        }
    }
}

/// This function will call the `cartridge_deployController` method to deploy the account if it
/// doesn't yet exist on the chain. But this JSON-RPC method is only available on Katana deployed on
/// Slot. If the `rpc_url` is not a Slot url, it will return an error.
///
/// `cartridge_deployController` is not a method that Katana itself exposes. It's from a middleware
/// layer that is deployed on top of the Katana deployment on Slot. This method will deploy the
/// contract of a user based on the Slot deployment.
async fn deploy_account_if_not_exist(
    rpc_url: Url,
    provider: &impl Provider,
    chain_id: Felt,
    address: Felt,
    username: &str,
) -> Result<()> {
    use reqwest::Client;
    use serde_json::json;

    // Check if the account exists on the chain
    match provider.get_class_at(BlockId::Tag(BlockTag::Pending), address).await {
        Ok(_) => Ok(()),

        // if account doesn't exist, deploy it by calling `cartridge_deployController` method
        Err(err @ StarknetError(ContractNotFound)) => {
            trace!(
                %username,
                chain = format!("{chain_id:#}"),
                address = format!("{address:#x}"),
                "Controller does not exist on chain. Attempting to deploy..."
            );

            // Skip deployment if the rpc_url is not a Slot instance
            if !rpc_url.host_str().map_or(false, |host| host.contains("api.cartridge.gg")) {
                warn!(%rpc_url, "Unable to deploy Controller on non-Slot instance.");
                bail!("Controller with username '{username}' does not exist: {err}");
            }

            let response = Client::new()
                .post(rpc_url)
                .json(&json!({
                    "id": 1,
                    "jsonrpc": "2.0",
                    "method": "cartridge_deployController",
                    "params": { "id": username },
                }))
                .send()
                .await?;

            if !response.status().is_success() {
                bail!("Failed to deploy controller: {}", response.status());
            }

            Ok(())
        }

        Err(e) => bail!(e),
    }
}
