use core::fmt;
use std::collections::VecDeque;
use std::path::PathBuf;
use std::str::FromStr;

use chrono::{DateTime, Utc};
use dojo_types::schema::Ty;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use starknet::core::types::Felt;

#[derive(Debug, Serialize, Deserialize)]
pub struct SQLFelt(pub Felt);

impl From<SQLFelt> for Felt {
    fn from(field_element: SQLFelt) -> Self {
        field_element.0
    }
}

impl TryFrom<String> for SQLFelt {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Ok(SQLFelt(Felt::from_hex(&value)?))
    }
}

impl fmt::LowerHex for SQLFelt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(FromRow, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Entity {
    pub id: String,
    pub keys: String,
    pub event_id: String,
    pub executed_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    // this should never be None
    #[sqlx(skip)]
    pub updated_model: Option<Ty>,
    #[sqlx(skip)]
    pub deleted: bool,
}

#[derive(FromRow, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EventMessage {
    pub id: String,
    pub keys: String,
    pub event_id: String,
    pub executed_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    // this should never be None
    #[sqlx(skip)]
    pub updated_model: Option<Ty>,
}

#[derive(FromRow, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Model {
    pub id: String,
    pub namespace: String,
    pub name: String,
    pub class_hash: String,
    pub contract_address: String,
    pub transaction_hash: String,
    pub executed_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(FromRow, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    pub id: String,
    pub keys: String,
    pub data: String,
    pub transaction_hash: String,
    pub executed_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Default, Deserialize, Debug, Clone)]
pub struct ToriiConfig {
    /// contract addresses to index
    pub contracts: VecDeque<Contract>,
}

impl ToriiConfig {
    pub fn load_from_path(path: &PathBuf) -> Result<Self, anyhow::Error> {
        let config = std::fs::read_to_string(path)?;
        let config: Self = toml::from_str(&config)?;
        Ok(config)
    }
}

#[derive(Deserialize, Debug, Clone, Copy)]
pub struct Contract {
    pub address: Felt,
    pub r#type: ContractType,
}

#[derive(Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ContractType {
    WORLD,
    ERC20,
    ERC20Legacy,
    ERC721,
    ERC721Legacy,
}

impl FromStr for ContractType {
    type Err = anyhow::Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input.to_lowercase().as_str() {
            "world" => Ok(ContractType::WORLD),
            "erc20" => Ok(ContractType::ERC20),
            "erc721" => Ok(ContractType::ERC721),
            "erc20legacy" | "erc20_legacy" => Ok(ContractType::ERC20Legacy),
            "erc721legacy" | "erc721_legacy" => Ok(ContractType::ERC721Legacy),
            _ => Err(anyhow::anyhow!("Invalid ERC type: {}", input)),
        }
    }
}

impl std::fmt::Display for ContractType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ContractType::WORLD => write!(f, "WORLD"),
            ContractType::ERC20 => write!(f, "ERC20"),
            ContractType::ERC721 => write!(f, "ERC721"),
            ContractType::ERC20Legacy => write!(f, "ERC20Legacy"),
            ContractType::ERC721Legacy => write!(f, "ERC721Legacy"),
        }
    }
}
