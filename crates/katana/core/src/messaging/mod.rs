pub mod service;
pub use service::MessagingService;

mod ethereum_messenger;
use ethereum_messenger::EthereumMessenger;

mod starknet_messenger;
use std::path::Path;

use anyhow::Result;
use async_trait::async_trait;
use ethers::providers::ProviderError;
use serde::Deserialize;
use starknet::core::types::MsgToL1;
use starknet_messenger::StarknetMessenger;
use tracing::{error, info};

use crate::backend::storage::transaction::L1HandlerTransaction;

pub(crate) const MSGING_TARGET: &str = "messaging";

type MessengerResult<T> = Result<T, MessengerError>;

#[derive(Debug, thiserror::Error)]
pub enum MessengerError {
    #[error("Error initializing messaging, please check messaging args")]
    InitError,
    #[error("Error gathering messages")]
    GatherError,
    #[error("Error sending messages")]
    SendError,
    #[error("Error ethereum provider")]
    EthereumProviderError(ProviderError),
}

#[derive(Debug, Default, Deserialize, Clone)]
pub struct MessagingConfig {
    // The RPC-URL of the settlement chain.
    pub rpc_url: String,
    // The messaging-contract address on the settlement chain.
    pub contract_address: String,
    // The sender address associated to the private key.
    pub sender_address: String,
    // The private key to send transaction on the settlement chain.
    pub private_key: String,
    // The interval at which Katana will fetch messages from settlement chain.
    pub fetch_interval: u64,
    // The block on settlement chain from where Katana will start fetching messages.
    pub from_block: u64,
}

impl MessagingConfig {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, std::io::Error> {
        let buf = std::fs::read(path)?;
        serde_json::from_slice(&buf).map_err(|e| e.into())
    }

    /// This is used as the clap `value_parser` implementation
    pub fn parse(path: &str) -> Result<Self, String> {
        Self::load(path).map_err(|e| e.to_string())
    }
}

#[async_trait]
pub trait Messenger {
    /// The type of the message hash.
    type MessageHash;

    /// Gathers messages emitted on the settlement chain and returns the
    /// list of transaction (L1HanlderTx) to be executed and the last fetched block.
    ///
    /// # Arguments
    ///
    /// * `from_block` - From which block the messages should be gathered.
    /// * `max_block` - The number of block fetched in the event/log filter. A too big value can
    ///   cause the RPC node to reject the query.
    async fn gather_messages(
        &self,
        from_block: u64,
        max_blocks: u64,
    ) -> MessengerResult<(u64, Vec<L1HandlerTransaction>)>;

    /// Computes the hash of the given messages and sends them to the settlement chain.
    ///
    /// Once message's hash is settled, one must send a transaction (with the message content)
    /// on the settlement chain to actually consume it.
    ///
    /// # Arguments
    ///
    /// * `messages` - Messages to settle.
    async fn settle_messages(
        &self,
        messages: &[MsgToL1],
    ) -> MessengerResult<Vec<Self::MessageHash>>;
}

pub enum AnyMessenger {
    Ethereum(EthereumMessenger),
    Starknet(StarknetMessenger),
}

impl AnyMessenger {
    pub async fn from_config(config: MessagingConfig) -> MessengerResult<Self> {
        if config.contract_address.len() < 50 {
            match EthereumMessenger::new(config.clone()).await {
                Ok(m_eth) => {
                    info!(MSGING_TARGET, "Messaging enabled [Ethereum]");
                    Ok(AnyMessenger::Ethereum(m_eth))
                }
                Err(e_eth) => {
                    error!("Ethereum messenger init failed: {:?}", e_eth);
                    Err(MessengerError::InitError)
                }
            }
        } else {
            match StarknetMessenger::new(config.clone()).await {
                Ok(m_sn) => {
                    info!(target: MSGING_TARGET, "Messaging enabled [Starknet]");
                    Ok(AnyMessenger::Starknet(m_sn))
                }
                Err(e_sn) => {
                    error!(target: MSGING_TARGET,
                           "Starknet messenger init failed: {:?}", e_sn);
                    Err(MessengerError::InitError)
                }
            }
        }
    }
}

#[async_trait]
impl Messenger for AnyMessenger {
    type MessageHash = String;

    async fn gather_messages(
        &self,
        from_block: u64,
        max_blocks: u64,
    ) -> MessengerResult<(u64, Vec<L1HandlerTransaction>)> {
        match self {
            Self::Ethereum(inner) => inner.gather_messages(from_block, max_blocks).await,
            Self::Starknet(inner) => inner.gather_messages(from_block, max_blocks).await,
        }
    }

    async fn settle_messages(
        &self,
        messages: &[MsgToL1],
    ) -> MessengerResult<Vec<Self::MessageHash>> {
        match self {
            Self::Ethereum(inner) => inner
                .settle_messages(messages)
                .await
                .map(|hashes| hashes.into_iter().map(|hash| format!("{hash:#x}")).collect()),

            Self::Starknet(inner) => inner
                .settle_messages(messages)
                .await
                .map(|hashes| hashes.into_iter().map(|hash| format!("{hash:#x}")).collect()),
        }
    }
}
