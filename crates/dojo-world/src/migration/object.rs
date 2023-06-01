use std::fs::File;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use cairo_lang_starknet::casm_contract_class::CasmContractClass;
use cairo_lang_starknet::contract_class::ContractClass;
use starknet::accounts::{Account, Call, ConnectedAccount, SingleOwnerAccount};
use starknet::core::types::contract::{CompiledClass, SierraClass};
use starknet::core::types::{BlockId, BlockTag, FieldElement, FlattenedSierraClass};
use starknet::core::utils::{
    cairo_short_string_to_felt, get_contract_address, get_selector_from_name,
    CairoShortStringToFeltError,
};
use starknet::providers::Provider;
use starknet::signers::Signer;
use thiserror::Error;

use super::world::{ClassDiff, ContractDiff};

#[derive(Debug, Error)]
pub enum MigrationError {
    #[error("World contract address not found.")]
    WorldAddressNotFound,
    #[error(transparent)]
    CairoShortStringToFeltError(#[from] CairoShortStringToFeltError),
}

// TODO: evaluate the contract address when building the migration plan
#[derive(Debug, Default)]
pub struct ContractMigration {
    pub salt: FieldElement,
    pub contract: ContractDiff,
    pub artifact_path: PathBuf,
    pub contract_address: Option<FieldElement>,
}

#[derive(Debug, Default)]
pub struct ClassMigration {
    pub class: ClassDiff,
    pub artifact_path: PathBuf,
}

pub struct WorldContractMigration(pub ContractMigration);

#[async_trait]
pub trait Declarable {
    async fn declare<P, S>(&self, account: &SingleOwnerAccount<P, S>)
    where
        P: Provider + Send + Sync,
        S: Signer + Send + Sync;
}

// TODO: Remove `mut` once we can calculate the contract address before sending the tx
#[async_trait]
pub trait Deployable: Declarable {
    async fn deploy<P, S>(
        &mut self,
        constructor_params: Vec<FieldElement>,
        account: &SingleOwnerAccount<P, S>,
    ) where
        P: Provider + Send + Sync,
        S: Signer + Send + Sync;
}

#[async_trait]
impl Declarable for ClassMigration {
    async fn declare<P, S>(&self, account: &SingleOwnerAccount<P, S>)
    where
        P: Provider + Send + Sync,
        S: Signer + Send + Sync,
    {
        declare(self.class.name.clone(), &self.artifact_path, account).await;
    }
}

#[async_trait]
impl Declarable for ContractMigration {
    async fn declare<P, S>(&self, account: &SingleOwnerAccount<P, S>)
    where
        P: Provider + Send + Sync,
        S: Signer + Send + Sync,
    {
        declare(self.contract.name.clone(), &self.artifact_path, account).await;
    }
}

async fn declare<P, S>(name: String, artifact_path: &PathBuf, account: &SingleOwnerAccount<P, S>)
where
    P: Provider + Send + Sync,
    S: Signer + Send + Sync,
{
    let (flattened_class, casm_class_hash) =
        prepare_contract_declaration_params(artifact_path).unwrap();

    if account.provider().get_class(&BlockId::Tag(BlockTag::Pending), casm_class_hash).await.is_ok()
    {
        println!("{name} class already declared");
        return;
    }

    let result = account.declare(Arc::new(flattened_class), casm_class_hash).send().await;

    match result {
        Ok(result) => {
            println!("Declared `{}` class at transaction: {:#x}", name, result.transaction_hash);
        }
        Err(error) => {
            if error.to_string().contains("already declared") {
                println!("{name} class already declared")
            } else {
                panic!("Problem declaring {name} class: {error}");
            }
        }
    }
}

#[async_trait]
impl Deployable for ContractMigration {
    async fn deploy<P, S>(
        &mut self,
        constructor_calldata: Vec<FieldElement>,
        account: &SingleOwnerAccount<P, S>,
    ) where
        P: Provider + Send + Sync,
        S: Signer + Send + Sync,
    {
        self.declare(account).await;

        let calldata = [
            vec![
                self.contract.local,                            // class hash
                FieldElement::ZERO,                             // salt
                FieldElement::ZERO,                             // unique
                FieldElement::from(constructor_calldata.len()), // constructor calldata len
            ],
            constructor_calldata.clone(),
        ]
        .concat();

        let contract_address = get_contract_address(
            FieldElement::ZERO,
            self.contract.local,
            &constructor_calldata,
            FieldElement::ZERO,
        );

        self.contract_address = Some(contract_address);

        if account
            .provider()
            .get_class_hash_at(&BlockId::Tag(BlockTag::Pending), contract_address)
            .await
            .is_ok()
        {
            // self.deployed = true;
            println!("{} contract already deployed", self.contract.name);
            return;
        }

        println!("Deploying `{}` contract", self.contract.name);

        let res = account
            .execute(vec![Call {
                calldata,
                // devnet UDC address
                to: FieldElement::from_hex_be(
                    "0x41a78e741e5af2fec34b695679bc6891742439f7afb8484ecd7766661ad02bf",
                )
                .unwrap(),
                selector: get_selector_from_name("deployContract").unwrap(),
            }])
            .send()
            .await
            .unwrap_or_else(|e| panic!("problem deploying `{}` contract: {e}", self.contract.name));

        println!(
            "Deployed `{}` contract at transaction: {:#x}",
            self.contract.name, res.transaction_hash
        );
        println!("`{} `Contract address: {contract_address:#x}", self.contract.name);
    }
}

impl WorldContractMigration {
    pub async fn deploy<P, S>(
        &mut self,
        name: impl AsRef<str>,
        executor: FieldElement,
        migrator: &SingleOwnerAccount<P, S>,
    ) where
        P: Provider + Send + Sync,
        S: Signer + Send + Sync,
    {
        self.0
            .deploy(vec![cairo_short_string_to_felt(name.as_ref()).unwrap(), executor], migrator)
            .await
    }

    pub async fn set_executor<P, S>(
        &self,
        executor: FieldElement,
        migrator: &SingleOwnerAccount<P, S>,
    ) -> Result<()>
    where
        P: Provider + Send + Sync,
        S: Signer + Send + Sync,
    {
        migrator
            .execute(vec![Call {
                calldata: vec![executor],
                to: self.0.contract_address.unwrap(),
                selector: get_selector_from_name("set_executor").unwrap(),
            }])
            .send()
            .await
            .unwrap_or_else(|err| panic!("problem setting executor: {err}"));
        Ok(())
    }

    pub async fn register_component<P, S>(
        &self,
        components: &[ClassMigration],
        migrator: &SingleOwnerAccount<P, S>,
    ) -> Result<()>
    where
        P: Provider + Send + Sync,
        S: Signer + Send + Sync,
    {
        let calls = components
            .iter()
            .map(|c| Call {
                to: self.0.contract_address.unwrap(),
                selector: get_selector_from_name("register_component").unwrap(),
                calldata: vec![c.class.local],
            })
            .collect::<Vec<_>>();

        migrator
            .execute(calls)
            .send()
            .await
            .unwrap_or_else(|err| panic!("problem registering components: {err}"));

        Ok(())
    }

    pub async fn register_system<P, S>(
        &self,
        systems: &[ClassMigration],
        migrator: &SingleOwnerAccount<P, S>,
    ) -> Result<()>
    where
        P: Provider + Send + Sync,
        S: Signer + Send + Sync,
    {
        let calls = systems
            .iter()
            .map(|s| Call {
                to: self.0.contract_address.unwrap(),
                selector: get_selector_from_name("register_system").unwrap(),
                calldata: vec![s.class.local],
            })
            .collect::<Vec<_>>();

        migrator
            .execute(calls)
            .send()
            .await
            .unwrap_or_else(|err| panic!("problem registering systems: {err}"));

        Ok(())
    }
}

fn prepare_contract_declaration_params(
    artifact_path: &PathBuf,
) -> Result<(FlattenedSierraClass, FieldElement)> {
    let flattened_class = get_flattened_class(artifact_path)
        .map_err(|e| anyhow!("error flattening the contract class: {e}"))?;
    let compiled_class_hash = get_compiled_class_hash(artifact_path)
        .map_err(|e| anyhow!("error computing compiled class hash: {e}"))?;
    Ok((flattened_class, compiled_class_hash))
}

fn get_flattened_class(artifact_path: &PathBuf) -> Result<FlattenedSierraClass> {
    let file = File::open(artifact_path)?;
    let contract_artifact: SierraClass = serde_json::from_reader(&file)?;
    Ok(contract_artifact.flatten()?)
}

fn get_compiled_class_hash(artifact_path: &PathBuf) -> Result<FieldElement> {
    let file = File::open(artifact_path)?;
    let casm_contract_class: ContractClass = serde_json::from_reader(file)?;
    let casm_contract = CasmContractClass::from_contract_class(casm_contract_class, true)
        .with_context(|| "Compilation failed.")?;
    let res = serde_json::to_string_pretty(&casm_contract)?;
    let compiled_class: CompiledClass = serde_json::from_str(&res)?;
    Ok(compiled_class.class_hash()?)
}
