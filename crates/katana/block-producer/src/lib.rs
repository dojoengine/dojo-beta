//! katana new pipeline to support syncing from da layer.
//!
//! ## Modes:-
//!
//! 1. sequencer
//!
//! - sync from underlying DA layer.
//!   - what to sync exactly ??? blocks ? state update ? or just veryfing the published state root ?
//! - once synced, start producing blocks.
//!   - transactions will not be added to the pool until the sync stage is finished.
//!
//! 2. full node
//!
//! - sync from underlying DA layer indefinitely.
//! - never transition to block production stage.

use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

// when a block production task is about to create a new block:
// 1. when there's no ongoing block production task
// 2. when the precondition for block production is met (eg block production interval or txs in mempool)
//
// - signal to other components that a new block is being produced (ie acquire a lock)
// - drain the pending txs streams
// - create a new block
// - update the state of the validator???
// - release the lock
//

use futures::StreamExt;
use katana_pool::{ordering::PoolOrd, tx::PoolTransaction, PoolSubscription, TransactionPool};
use katana_primitives::transaction::ExecutableTxWithHash;

type PoolSubs<P> =
    PoolSubscription<<P as TransactionPool>::Transaction, <P as TransactionPool>::Ordering>;

struct SequencingTask<T>
where
    T: BlockProducer,
{
    block_producer: T,
    pool_subscription: Option<PoolSubs<T::Pool>>,
}

impl<T> Future for SequencingTask<T>
where
    T: BlockProducer,
{
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();

        if this.block_producer.poll_block_open(cx).is_ready() {
            if let Some(mut sub) = this.pool_subscription.take() {
                if sub.poll_next_unpin(cx).is_ready() {
                    this.block_producer.on_transactions(&mut sub);
                }

                this.pool_subscription = Some(sub);
            }
        }

        if this.block_producer.poll_block_close(cx).is_ready() {
            this.block_producer.on_close();
        }

        Poll::Pending
    }
}

trait BlockProducer: Unpin {
    type Pool: TransactionPool;

    /// to indicate when a new block should be opened
    fn poll_block_open(&mut self, cx: &mut Context<'_>) -> Poll<()>;

    /// to indicate when a new block should be closed
    fn poll_block_close(&mut self, cx: &mut Context<'_>) -> Poll<()>;

    /// will be called when the call to poll_block_open returns ready
    fn on_open(&mut self);

    /// when there are transactions in the mempool
    fn on_transactions(&mut self, transactions: &mut PoolSubs<Self::Pool>);

    /// will be called when the call to poll_block_close returns ready
    fn on_close(&mut self);
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use katana_pool::{ordering::FiFo, pool::test_utils::TestPool};

    struct AutoMining {}

    impl BlockProducer for AutoMining {
        type Pool = TestPool;

        fn poll_block_open(&mut self, _cx: &mut Context<'_>) -> Poll<()> {
            Poll::Ready(())
        }

        fn poll_block_close(&mut self, _cx: &mut Context<'_>) -> Poll<()> {
            Poll::Ready(())
        }

        fn on_open(&mut self) {}

        fn on_transactions(
            &mut self,
            transactions: &mut PoolSubscription<
                <Self::Pool as TransactionPool>::Transaction,
                <Self::Pool as TransactionPool>::Ordering,
            >,
        ) {
            todo!()
        }

        fn on_close(&mut self) {}
    }

    #[tokio::test]
    async fn instant_mining() -> Result<()> {
        let pool = TestPool::test();

        Ok(())
    }
}
