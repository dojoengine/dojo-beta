#![cfg_attr(not(test), warn(unused_crate_dependencies))]

pub mod ordering;
pub mod pool;
pub mod tx;
pub mod validation;

use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use futures::channel::mpsc::Receiver;
use futures::StreamExt;
use katana_primitives::transaction::{ExecutableTxWithHash, TxHash};
use ordering::{FiFo, PoolOrd};
use pool::Pool;
use tx::{PendingTx, PoolTransaction};
use validation::stateful::TxValidator;
use validation::{InvalidTransactionError, Validator};

/// Katana default transacstion pool type.
pub type TxPool = Pool<ExecutableTxWithHash, TxValidator, FiFo<ExecutableTxWithHash>>;

pub type PoolResult<T> = Result<T, PoolError>;

#[derive(Debug, thiserror::Error)]
pub enum PoolError {
    #[error("Invalid transaction: {0}")]
    InvalidTransaction(Box<InvalidTransactionError>),
    #[error("Internal error: {0}")]
    Internal(Box<dyn std::error::Error>),
}

/// Represents a subscription to the transaction pool.
pub struct PoolSubscription<T, O>(Receiver<PendingTx<T, O>>)
where
    T: PoolTransaction,
    O: PoolOrd<Transaction = T>;

impl<T, O> futures::Stream for PoolSubscription<T, O>
where
    T: PoolTransaction,
    O: PoolOrd<Transaction = T>,
{
    type Item = PendingTx<T, O>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.0.poll_next_unpin(cx)
    }
}

/// Represents a complete transaction pool.
pub trait TransactionPool {
    /// The pool's transaction type.
    type Transaction: PoolTransaction;

    /// The ordering mechanism to use. This is used to determine
    /// how transactions are being ordered within the pool.
    type Ordering: PoolOrd<Transaction = Self::Transaction>;

    /// Transaction validation before adding to the pool.
    type Validator: Validator<Transaction = Self::Transaction>;

    /// Add a new transaction to the pool.
    fn add_transaction(&self, tx: Self::Transaction) -> PoolResult<TxHash>;

    fn take_transactions(
        &self,
    ) -> impl Iterator<Item = PendingTx<Self::Transaction, Self::Ordering>>;

    fn subscribe(&self) -> PoolSubscription<Self::Transaction, Self::Ordering> {
        todo!()
    }

    /// Check if the pool contains a transaction with the given hash.
    fn contains(&self, hash: TxHash) -> bool;

    /// Get a transaction from the pool by its hash.
    fn get(&self, hash: TxHash) -> Option<Arc<Self::Transaction>>;

    fn add_listener(&self) -> Receiver<TxHash>;

    /// Get the total number of transactions in the pool.
    fn size(&self) -> usize;

    /// Get a reference to the pool's validator.
    fn validator(&self) -> &Self::Validator;
}
