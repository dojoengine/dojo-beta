use block::{BlockProvider, BlockWriter};
use contract::ContractClassWriter;
use env::BlockEnvProvider;
use state::{StateFactoryProvider, StateRootProvider, StateWriter};
use state_update::StateUpdateProvider;
use transaction::{
    ReceiptProvider, TransactionProvider, TransactionStatusProvider, TransactionTraceProvider,
    TransactionsProviderExt,
};

use crate::{BlockchainProvider, ProviderResult};

pub mod block;
pub mod contract;
pub mod env;
pub mod state;
pub mod state_update;
pub mod transaction;

// TODO: placeholder traits, we should unify all provider/db (in-memory or on disk) implementations to be transactional
pub trait Database:
    BlockProvider
    + ReceiptProvider
    + BlockEnvProvider
    + StateRootProvider
    + TransactionProvider
    + StateUpdateProvider
    + StateFactoryProvider
    + TransactionsProviderExt
    + TransactionTraceProvider
    + TransactionStatusProvider
    + Send
    + Sync
    + 'static
{
}

pub trait DatabaseMut: Database + BlockWriter + StateWriter + ContractClassWriter {}

impl<T> Database for T where
    T: BlockProvider
        + ReceiptProvider
        + BlockEnvProvider
        + StateRootProvider
        + TransactionProvider
        + StateUpdateProvider
        + StateFactoryProvider
        + TransactionsProviderExt
        + TransactionTraceProvider
        + TransactionStatusProvider
        + Send
        + Sync
        + 'static
{
}

impl<T> DatabaseMut for T where T: Database + BlockWriter + StateWriter + ContractClassWriter {}

pub trait ProviderFactory: Send + Sync {
    fn provider(&self) -> ProviderResult<BlockchainProvider<Box<dyn Database>>>;

    fn provider_mut(&self) -> ProviderResult<BlockchainProvider<Box<dyn DatabaseMut>>> {
        todo!()
    }
}
