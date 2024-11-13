use katana_primitives::Felt;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use starknet::core::types::{DataAvailabilityMode, ResourceBoundsMapping};

use crate::transactions::QUERY_VERSION_OFFSET;

pub enum BroadcastedInvokeTx {
    /// Version 1 `INVOKE` transaction.
    V1(BroadcastedInvokeTxV1),
    /// Version 3 `INVOKE` transaction.
    V3(BroadcastedInvokeTxV3),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BroadcastedInvokeTxV1 {
    /// Sender address
    pub sender_address: Felt,
    /// The data expected by the account's `execute` function (in most usecases, this includes the
    /// called contract address and a function selector)
    pub calldata: Vec<Felt>,
    /// The maximal fee that can be charged for including the transaction
    pub max_fee: Felt,
    /// Signature
    pub signature: Vec<Felt>,
    /// Nonce
    pub nonce: Felt,
    /// If set to `true`, uses a query-only transaction version that's invalid for execution
    pub is_query: bool,
}

/// Invoke transaction v3.
///
/// Initiates a transaction from a given account.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BroadcastedInvokeTxV3 {
    /// Sender address
    pub sender_address: Felt,
    /// The data expected by the account's `execute` function (in most usecases, this includes the
    /// called contract address and a function selector)
    pub calldata: Vec<Felt>,
    /// Signature
    pub signature: Vec<Felt>,
    /// Nonce
    pub nonce: Felt,
    /// Resource bounds for the transaction execution
    pub resource_bounds: ResourceBoundsMapping,
    /// The tip for the transaction
    pub tip: u64,
    /// Data needed to allow the paymaster to pay for the transaction in native tokens
    pub paymaster_data: Vec<Felt>,
    /// Data needed to deploy the account contract from which this tx will be initiated
    pub account_deployment_data: Vec<Felt>,
    /// The storage domain of the account's nonce (an account has a nonce per da mode)
    pub nonce_data_availability_mode: DataAvailabilityMode,
    /// The storage domain of the account's balance from which fee will be charged
    pub fee_data_availability_mode: DataAvailabilityMode,
    /// If set to `true`, uses a query-only transaction version that's invalid for execution
    pub is_query: bool,
}

impl Serialize for BroadcastedInvokeTxV1 {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        #[derive(Serialize)]
        struct Tagged<'a> {
            pub r#type: &'a str,
            pub sender_address: &'a Felt,
            pub calldata: &'a [Felt],
            pub max_fee: &'a Felt,
            pub version: &'a Felt,
            pub signature: &'a [Felt],
            pub nonce: &'a Felt,
        }

        let r#type = "INVOKE";

        let version = &(if self.is_query { Felt::ONE + QUERY_VERSION_OFFSET } else { Felt::ONE });

        let tagged = Tagged {
            r#type,
            sender_address: &self.sender_address,
            calldata: &self.calldata,
            max_fee: &self.max_fee,
            version,
            signature: &self.signature,
            nonce: &self.nonce,
        };

        Tagged::serialize(&tagged, serializer)
    }
}

impl<'de> Deserialize<'de> for BroadcastedInvokeTxV1 {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct Tagged {
            pub r#type: Option<String>,
            pub sender_address: Felt,
            pub calldata: Vec<Felt>,
            pub max_fee: Felt,
            pub version: Felt,
            pub signature: Vec<Felt>,
            pub nonce: Felt,
        }

        let tagged = Tagged::deserialize(deserializer)?;

        if let Some(tag_field) = &tagged.r#type {
            if tag_field != "INVOKE" {
                return Err(serde::de::Error::custom("invalid `type` value"));
            }
        }

        let is_query = if tagged.version == Felt::ONE {
            false
        } else if tagged.version == Felt::ONE + QUERY_VERSION_OFFSET {
            true
        } else {
            return Err(serde::de::Error::custom("invalid `version` value"));
        };

        Ok(Self {
            sender_address: tagged.sender_address,
            calldata: tagged.calldata,
            max_fee: tagged.max_fee,
            signature: tagged.signature,
            nonce: tagged.nonce,
            is_query,
        })
    }
}

impl Serialize for BroadcastedInvokeTxV3 {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        #[derive(Serialize)]
        struct Tagged<'a> {
            pub r#type: &'a str,
            pub sender_address: &'a Felt,
            pub calldata: &'a [Felt],
            pub version: &'a Felt,
            pub signature: &'a [Felt],
            pub nonce: &'a Felt,
            pub resource_bounds: &'a ResourceBoundsMapping,
            #[serde(serialize_with = "serialize_u64_hex")]
            pub tip: &'a u64,
            pub paymaster_data: &'a [Felt],
            pub account_deployment_data: &'a [Felt],
            pub nonce_data_availability_mode: &'a DataAvailabilityMode,
            pub fee_data_availability_mode: &'a DataAvailabilityMode,
        }

        let r#type = "INVOKE";

        let version =
            &(if self.is_query { Felt::THREE + QUERY_VERSION_OFFSET } else { Felt::THREE });

        let tagged = Tagged {
            r#type,
            sender_address: &self.sender_address,
            calldata: &self.calldata,
            version,
            signature: &self.signature,
            nonce: &self.nonce,
            resource_bounds: &self.resource_bounds,
            tip: &self.tip,
            paymaster_data: &self.paymaster_data,
            account_deployment_data: &self.account_deployment_data,
            nonce_data_availability_mode: &self.nonce_data_availability_mode,
            fee_data_availability_mode: &self.fee_data_availability_mode,
        };

        Tagged::serialize(&tagged, serializer)
    }
}

impl<'de> Deserialize<'de> for BroadcastedInvokeTxV3 {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct Tagged {
            pub r#type: Option<String>,
            pub sender_address: Felt,
            pub calldata: Vec<Felt>,
            pub version: Felt,
            pub signature: Vec<Felt>,
            pub nonce: Felt,
            pub resource_bounds: ResourceBoundsMapping,
            #[serde(deserialize_with = "deserialize_u64_hex")]
            pub tip: u64,
            pub paymaster_data: Vec<Felt>,
            pub account_deployment_data: Vec<Felt>,
            pub nonce_data_availability_mode: DataAvailabilityMode,
            pub fee_data_availability_mode: DataAvailabilityMode,
        }

        let tagged = Tagged::deserialize(deserializer)?;

        if let Some(tag_field) = &tagged.r#type {
            if tag_field != "INVOKE" {
                return Err(serde::de::Error::custom("invalid `type` value"));
            }
        }

        let is_query = if tagged.version == Felt::THREE {
            false
        } else if tagged.version == Felt::THREE + QUERY_VERSION_OFFSET {
            true
        } else {
            return Err(serde::de::Error::custom("invalid `version` value"));
        };

        Ok(Self {
            sender_address: tagged.sender_address,
            calldata: tagged.calldata,
            signature: tagged.signature,
            nonce: tagged.nonce,
            resource_bounds: tagged.resource_bounds,
            tip: tagged.tip,
            paymaster_data: tagged.paymaster_data,
            account_deployment_data: tagged.account_deployment_data,
            nonce_data_availability_mode: tagged.nonce_data_availability_mode,
            fee_data_availability_mode: tagged.fee_data_availability_mode,
            is_query,
        })
    }
}

fn serialize_u64_hex<S>(value: &u64, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&format!("{value:#x}"))
}

fn deserialize_u64_hex<'de, D: Deserializer<'de>>(deserializer: D) -> Result<u64, D::Error> {
    let s = String::deserialize(deserializer)?;
    if let Some(hex) = s.strip_prefix("0x") {
        u64::from_str_radix(hex, 16).map_err(serde::de::Error::custom)
    } else {
        Err(serde::de::Error::custom("expected hex string starting with 0x"))
    }
}
