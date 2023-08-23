use async_trait::async_trait;
use starknet::core::types::MsgToL1;

use super::ethereum_messenger::EthereumMessenger;
use super::starknet_messenger::StarknetMessenger;
use crate::backend::storage::transaction::L1HandlerTransaction;
use crate::messaging::{Messenger, MessengerError, MessengerResult};
use crate::sequencer::SequencerMessagingConfig;

pub enum AnyMessenger {
    Ethereum(EthereumMessenger),
    Starknet(StarknetMessenger),
}

pub async fn from_config(config: SequencerMessagingConfig) -> MessengerResult<AnyMessenger> {
    // TODO: instead of trying the init of both, how can we easily
    // determine the chain from the config? Messaging contract address size?
    match EthereumMessenger::new(config.clone()).await {
        Ok(m_eth) => {
            tracing::debug!("Messaging enabled [Ethereum]");
            Ok(AnyMessenger::Ethereum(m_eth))
        }
        Err(e_eth) => {
            tracing::debug!("Ethereum messenger init failed: {:?}", e_eth);
            match StarknetMessenger::new(config.clone()).await {
                Ok(m_sn) => {
                    tracing::debug!("Messaging enabled [Starknet]");
                    Ok(AnyMessenger::Starknet(m_sn))
                }
                Err(e_sn) => {
                    tracing::debug!("Starknet messenger init failed: {:?}", e_sn);
                    Err(MessengerError::InitError)
                }
            }
        }
    }
}

#[async_trait]
impl Messenger for AnyMessenger {
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

    async fn settle_messages(&self, messages: &[MsgToL1]) -> MessengerResult<Vec<String>> {
        match self {
            Self::Ethereum(inner) => inner.settle_messages(messages).await,
            Self::Starknet(inner) => inner.settle_messages(messages).await,
        }
    }
}
