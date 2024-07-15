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

// TODO: Unify provider implementations (in-memory and on-disk) to be transactional
// and conform to the general `Database` trait. This would allow us to:
//
// 1. Make only the underlying database generic instead of the whole provider.
// 2. Maintain only a single type (`DbProvider<Db: Database>`) that implements all the provider
//    traits and make the `Database` pluggable.
// 3. Remove the need for defining independent provider types for each storage implementations
//    (in-memory/on-disk).
// 4. Potentially simplify the provider hierarchy by leveraging common database operations.
// 5. Improve consistency between different storage backends.
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

    /// Creates a [ProviderMut] instance to allow performing write operations on the blockchain
    /// data.
    fn provider_mut(&self) -> ProviderResult<BlockchainProvider<Box<dyn ProviderMut>>>;
}
