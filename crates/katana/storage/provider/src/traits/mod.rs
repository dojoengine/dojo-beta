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

// TODO: placeholder traits, we should unify all provider/db (in-memory or on disk) implementations to be transactional or at least
// to conform to the general [Database] trait so that we only need to make the underlying database generic instead of
// the whole provider. essentially, we can have only a single type that implements all of this provider traits, (Eg DbProvider<Db: Database>)
// and the `Database` will be pluggable to any database implementation (in-memory, on-disk, etc)
pub trait Provider:
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

pub trait ProviderMut: Provider + BlockWriter + StateWriter + ContractClassWriter {}

impl<T> Provider for T where
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

impl<T> ProviderMut for T where T: Provider + BlockWriter + StateWriter + ContractClassWriter {}

/// Types that implement this trait must be able to give read/write access to the underlying
/// database data through the use of the [Provider] and [ProviderMut] traits.
pub trait ProviderFactory: Send + Sync {
    /// Creates a [Provider] instance for reading the blockchain data.
    fn provider(&self) -> ProviderResult<BlockchainProvider<Box<dyn Provider>>>;

    /// Creates a [ProviderMut] instance to allow performing write operations on the blockchain data.
    fn provider_mut(&self) -> ProviderResult<BlockchainProvider<Box<dyn ProviderMut>>>;
}
