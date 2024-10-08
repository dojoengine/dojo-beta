use std::collections::{BTreeMap, BTreeSet};

use crate::class::{ClassHash, CompiledClass, CompiledClassHash, FlattenedSierraClass};
use crate::contract::{ContractAddress, Nonce, StorageKey, StorageValue};

/// State updates.
///
/// Represents all the state updates after performing some executions on a state.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StateUpdates {
    /// A mapping of contract addresses to their updated nonces.
    pub nonce_updates: BTreeMap<ContractAddress, Nonce>,
    /// A mapping of contract addresses to their updated storage entries.
    pub storage_updates: BTreeMap<ContractAddress, BTreeMap<StorageKey, StorageValue>>,
    /// A mapping of contract addresses to their updated class hashes.
    pub deployed_contracts: BTreeMap<ContractAddress, ClassHash>,
    /// A mapping of newly declared class hashes to their compiled class hashes.
    pub declared_classes: BTreeMap<ClassHash, CompiledClassHash>,
    /// A mapping of newly declared legacy class hashes.
    pub deprecated_declared_classes: BTreeSet<ClassHash>,
    /// A mapping of replaced contract addresses to their new class hashes ie using `replace_class`
    /// syscall.
    pub replaced_classes: BTreeMap<ContractAddress, ClassHash>,
}

impl StateUpdates {
    pub fn merge(&mut self, other: StateUpdates) {
        self.nonce_updates.extend(other.nonce_updates);

        for (addr, storage) in other.storage_updates {
            self.storage_updates.entry(addr).or_default().extend(storage);
        }

        self.replaced_classes.extend(other.replaced_classes);
        self.deployed_contracts.extend(other.deployed_contracts);

        self.declared_classes.extend(other.declared_classes);
        self.deprecated_declared_classes.extend(other.deprecated_declared_classes);
    }
}

/// State update with declared classes definition.
#[derive(Debug, Default, Clone)]
pub struct StateUpdatesWithDeclaredClasses {
    /// State updates.
    pub state_updates: StateUpdates,
    /// A mapping of class hashes to their sierra classes definition.
    pub declared_sierra_classes: BTreeMap<ClassHash, FlattenedSierraClass>,
    /// A mapping of class hashes to their compiled classes definition.
    pub declared_compiled_classes: BTreeMap<ClassHash, CompiledClass>,
}
