use std::collections::HashMap;
use std::fmt::Display;
use std::fs;
use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use camino::Utf8PathBuf;
use starknet::core::types::FieldElement;

use super::object::{ClassMigration, ContractMigration, WorldContractMigration};
use super::MigrationStrategy;
use crate::config::{EnvironmentConfig, WorldConfig};
use crate::manifest::Manifest;

#[cfg(test)]
#[path = "world_test.rs"]
mod test;

/// Represents differences between a local and remote contract.
#[derive(Debug, Default, Clone)]
pub struct ContractDiff {
    pub name: String,
    pub local: FieldElement,
    pub remote: Option<FieldElement>,
    pub address: Option<FieldElement>,
}

/// Represents differences between a local and remote class.
#[derive(Debug, Default, Clone)]
pub struct ClassDiff {
    pub name: String,
    pub local: FieldElement,
    pub remote: Option<FieldElement>,
}

/// Represents the state differences between the local and remote worlds.
#[derive(Debug)]
pub struct WorldDiff {
    world: ContractDiff,
    executor: ContractDiff,
    contracts: Vec<ClassDiff>,
    components: Vec<ClassDiff>,
    systems: Vec<ClassDiff>,
    world_config: WorldConfig,
}

impl WorldDiff {
    pub async fn from_path(
        target_dir: Utf8PathBuf,
        world_config: WorldConfig,
        environment_config: EnvironmentConfig,
    ) -> Result<WorldDiff> {
        let local_manifest = Manifest::load_from_path(target_dir.join("manifest.json"))?;

        let remote_manifest = if let Some(world_address) = world_config.address {
            let provider = environment_config.provider()?;
            Manifest::from_remote(provider, world_address, Some(local_manifest.clone()))
                .await
                .map(|m| Some(m))
                .map_err(|e| anyhow!("Failed creating remote manifest: {e}"))?
        } else {
            None
        };

        let systems = local_manifest
            .systems
            .iter()
            .map(|system| {
                ClassDiff {
                    // because the name returns by the `name` method of a
                    // system contract is without the 'System' suffix
                    name: system.name.strip_suffix("System").unwrap_or(&system.name).to_string(),
                    local: system.class_hash,
                    remote: remote_manifest.as_ref().and_then(|m| {
                        m.systems.iter().find(|e| e.name == system.name).map(|s| s.class_hash)
                    }),
                }
            })
            .collect::<Vec<_>>();

        let components = local_manifest
            .components
            .iter()
            .map(|component| ClassDiff {
                name: component.name.to_string(),
                local: component.class_hash,
                remote: remote_manifest.as_ref().and_then(|m| {
                    m.components.iter().find(|e| e.name == component.name).map(|s| s.class_hash)
                }),
            })
            .collect::<Vec<_>>();

        let contracts = local_manifest
            .contracts
            .iter()
            .map(|contract| ClassDiff {
                name: contract.name.to_string(),
                local: contract.class_hash,
                remote: None,
            })
            .collect::<Vec<_>>();

        let world = ContractDiff {
            name: "World".into(),
            address: world_config.address,
            local: local_manifest.world,
            remote: remote_manifest.as_ref().map(|m| m.world),
        };

        let executor = ContractDiff {
            name: "Executor".into(),
            address: None,
            local: local_manifest.executor,
            remote: remote_manifest.map(|m| m.executor),
        };

        Ok(WorldDiff { world, executor, systems, contracts, components, world_config })
    }

    /// construct migration strategy
    /// evaluate which contracts/classes need to be declared/deployed
    pub async fn prepare_for_migration(
        &self,
        target_dir: Utf8PathBuf,
    ) -> Result<MigrationStrategy> {
        let entries = fs::read_dir(target_dir)
            .map_err(|err| anyhow!("Failed reading source directory: {err}"))?;

        let mut artifact_paths = HashMap::new();
        for entry in entries.flatten() {
            let file_name = entry.file_name();
            let file_name_str = file_name.to_string_lossy();
            if file_name_str == "manifest.json" || !file_name_str.ends_with(".json") {
                continue;
            }

            let name =
                file_name_str.split('_').last().unwrap().trim_end_matches(".json").to_string();

            artifact_paths.insert(name, entry.path());
        }

        // We don't need to care if a contract has already been declared or not, because
        // the migration strategy will take care of that.

        // If the world contract needs to be migrated, then all contracts need to be migrated
        // else we need to evaluate which contracts need to be migrated.
        let world = evaluate_contract_to_migrate(&self.world, &artifact_paths, false)?
            .map(|c| WorldContractMigration(c));
        let executor =
            evaluate_contract_to_migrate(&self.executor, &artifact_paths, world.is_some())?;
        let components =
            evaluate_components_to_migrate(&self.components, &artifact_paths, world.is_some())?;
        let systems = evaluate_systems_to_migrate(&self.systems, &artifact_paths, world.is_some())?;

        Ok(MigrationStrategy {
            world,
            executor,
            systems,
            components,
            world_config: self.world_config,
        })
    }
}

impl Display for ContractDiff {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}:", self.name)?;
        if let Some(address) = self.address {
            writeln!(f, "   Address: {address:#x}",)?;
        }
        writeln!(f, "   Local: {:#x}", self.local)?;

        if let Some(remote) = self.remote {
            writeln!(f, "   Remote: {remote:#x}")?;
        }

        Ok(())
    }
}

impl Display for ClassDiff {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}:", self.name)?;
        writeln!(f, "   Local: {:#x}", self.local)?;

        if let Some(remote) = self.remote {
            writeln!(f, "   Remote: {remote:#x}")?;
        }

        Ok(())
    }
}

impl Display for WorldDiff {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.world)?;
        writeln!(f, "{}", self.executor)?;

        for component in &self.components {
            writeln!(f, "{component}")?;
        }

        for system in &self.systems {
            writeln!(f, "{system}")?;
        }

        for contract in &self.contracts {
            writeln!(f, "{contract}")?;
        }

        Ok(())
    }
}

fn evaluate_systems_to_migrate(
    systems: &[ClassDiff],
    artifact_paths: &HashMap<String, PathBuf>,
    world_contract_will_migrate: bool,
) -> Result<Vec<ClassMigration>> {
    let mut syst_to_migrate = vec![];

    for s in systems {
        match s.remote {
            Some(remote) if remote == s.local && !world_contract_will_migrate => continue,
            _ => {
                let path = find_artifact_path(&format!("{}System", s.name), artifact_paths)?;
                syst_to_migrate.push(ClassMigration {
                    // declared: false,
                    class: s.clone(),
                    artifact_path: path.clone(),
                });
            }
        }
    }

    Ok(syst_to_migrate)
}

fn evaluate_components_to_migrate(
    components: &[ClassDiff],
    artifact_paths: &HashMap<String, PathBuf>,
    world_contract_will_migrate: bool,
) -> Result<Vec<ClassMigration>> {
    let mut comps_to_migrate = vec![];

    for c in components {
        match c.remote {
            Some(remote) if remote == c.local && !world_contract_will_migrate => continue,
            _ => {
                let path = find_artifact_path(&format!("{}Component", c.name), artifact_paths)?;
                comps_to_migrate.push(ClassMigration {
                    class: c.clone(),
                    artifact_path: path.clone(),
                    ..Default::default()
                });
            }
        }
    }

    Ok(comps_to_migrate)
}

fn evaluate_contract_to_migrate(
    contract: &ContractDiff,
    artifact_paths: &HashMap<String, PathBuf>,
    world_contract_will_migrate: bool,
) -> Result<Option<ContractMigration>> {
    if world_contract_will_migrate
        || contract.address.is_none()
        || matches!(contract.remote, Some(remote_hash) if remote_hash != contract.local)
    {
        let path = find_artifact_path(&contract.name, artifact_paths)?;

        // TODO: generate random salt
        Ok(Some(ContractMigration {
            contract: contract.clone(),
            artifact_path: path.clone(),
            ..Default::default()
        }))
    } else {
        Ok(None)
    }
}

fn find_artifact_path<'a>(
    contract_name: &str,
    artifact_paths: &'a HashMap<String, PathBuf>,
) -> Result<&'a PathBuf> {
    artifact_paths
        .get(contract_name)
        .with_context(|| anyhow!("missing contract artifact for `{}` contract", contract_name))
}
