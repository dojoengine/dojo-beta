use std::fmt;

use num_bigint::BigUint;
use starknet::core::utils::normalize_address;

use crate::class::ClassHash;
use crate::Felt;

/// Represents the type for a contract storage key.
pub type StorageKey = Felt;
/// Represents the type for a contract storage value.
pub type StorageValue = Felt;

/// Represents the type for a contract nonce.
pub type Nonce = Felt;

/// Represents a contract address.
#[derive(Default, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Hash, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Address(Felt);

impl Address {
    /// Creates a new [`Address`] from the raw internal representation.
    pub const fn from_raw(value: [u64; 4]) -> Self {
        Self(Felt::from_raw(value))
    }
}

impl core::ops::Deref for Address {
    type Target = Felt;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#x}", self.0)
    }
}

impl From<Felt> for Address {
    fn from(value: Felt) -> Self {
        Address(normalize_address(value))
    }
}

impl From<Address> for Felt {
    fn from(value: Address) -> Self {
        value.0
    }
}

impl From<&BigUint> for Address {
    fn from(biguint: &BigUint) -> Self {
        Self::from(Felt::from_bytes_le_slice(&biguint.to_bytes_le()))
    }
}

impl From<BigUint> for Address {
    fn from(biguint: BigUint) -> Self {
        Self::from(Felt::from_bytes_le_slice(&biguint.to_bytes_le()))
    }
}

/// Represents a generic contract instance information.
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GenericContractInfo {
    /// The nonce of the contract.
    pub nonce: Nonce,
    /// The hash of the contract class.
    pub class_hash: ClassHash,
}
