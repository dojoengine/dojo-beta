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

//! Pool updates:-
//! - get pending transactions based on some filtering (necessary for hypervisor node)

use std::future::Future;
use std::pin::Pin;
use std::sync::atomic;
use std::task::{Context, Poll};

use futures::channel::mpsc::Receiver;
use futures::future::BoxFuture;
use futures::StreamExt;
// when a block production task is about to create a new block:
// 1. when there's no ongoing block production task
// 2. when the precondition for block production is met (eg block production interval or txs in
//    mempool)
//
// - signal to other components that a new block is being produced (ie acquire a lock)
// - drain the pending txs streams
// - create a new block
// - update the state of the validator???
// - release the lock
use katana_pool::TransactionPool;
use katana_primitives::transaction::TxHash;

#[derive(Debug, thiserror::Error)]
pub enum Error {}

type ExecutionTask = BoxFuture<'static, Result<(), Error>>;
type BlockCommitmentTask = BoxFuture<'static, Result<BlockCommitmentOutcome, Error>>;

type PendingTransactions<T> = katana_pool::pool::PendingTransactions<
    <T as TransactionPool>::Transaction,
    <T as TransactionPool>::Ordering,
>;

#[derive(Debug, Default)]
struct BlockCommitmentOutcome {}

trait BlockProducer: Unpin {
    type Pool: TransactionPool;

    /// to indicate when a new block should be opened
    fn poll_block_open(&mut self, cx: &mut Context<'_>) -> Poll<()>;

    /// to indicate when a new block should be closed
    fn poll_block_close(&mut self, cx: &mut Context<'_>) -> Poll<()>;

    /// will be called when the call to poll_block_open returns ready.
    ///
    /// this is where we can update the validator state if necessary.
    fn on_open(&mut self);

    /// when there are transactions in the mempool
    fn on_transactions(&mut self, transactions: PendingTransactions<Self::Pool>) -> ExecutionTask;

    /// will be called when the call to poll_block_close returns ready
    fn on_close(&mut self) -> BlockCommitmentTask;

    /// indicates whether a block is currently open
    fn is_open(&self) -> bool;
}

struct BlockProductionTask<BP, P> {
    miner: BP,
    pool: P,
    pool_listener: Receiver<TxHash>,
    execution_task: Option<ExecutionTask>,
    commitment_task: Option<BlockCommitmentTask>,
}

impl<BP, P> BlockProductionTask<BP, P>
where
    BP: BlockProducer<Pool = P>,
    P: TransactionPool,
{
    fn new(miner: BP, pool: P) -> Self {
        let pool_listener = pool.add_listener();
        Self { miner, pool, pool_listener, execution_task: None, commitment_task: None }
    }
}

impl<BP, P> Future for BlockProductionTask<BP, P>
where
    BP: BlockProducer<Pool = P> + Unpin,
    P: TransactionPool + Unpin,
{
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();

        // Only check for opening a new block if there are no ongoing tasks
        if this.execution_task.is_none() && this.commitment_task.is_none() {
            if this.miner.poll_block_open(cx).is_ready() {
                this.miner.on_open();
            }
        }

        // check if the miner has a block opened, if yes we can start processing transactions
        if this.miner.is_open() {
            // check if the pool has pending transactions
            if this.pool_listener.poll_next_unpin(cx).is_ready() {
                // get the pending transactions from the pool
                let transactions = this.pool.take_transactions();
                let fut = this.miner.on_transactions(transactions);
                this.execution_task = Some(Box::pin(fut));
            }
        }

        // for instant mining, after executing transactions, we lock
        // the provider to prevent new transactions from being added
        // to the pool while we're producing the block.
        if let Some(mut task) = this.execution_task.take() {
            match task.as_mut().poll(cx) {
                Poll::Ready(Ok(_)) => {
                    // Notify listeners here
                    // this.notify_listeners();
                }

                Poll::Ready(Err(_)) => {
                    // error handling logic here
                }

                // no need to continue the rest of the function if the execution task is pending
                Poll::Pending => {
                    this.execution_task = Some(task);
                    return Poll::Pending;
                }
            }
        }

        // we only poll for block closing if there's no ongoing tasks
        if this.execution_task.is_none() && this.commitment_task.is_none() {
            if this.miner.poll_block_close(cx).is_ready() {
                let fut = this.miner.on_close();
                this.commitment_task = Some(Box::pin(fut));
            }
        }

        // Poll the block production task
        if let Some(mut task) = this.commitment_task.take() {
            match task.as_mut().poll(cx) {
                Poll::Ready(Ok(_)) => {
                    // Block production completed
                    // Any post-production actions can be added here (eg metrics)
                }

                Poll::Ready(Err(_)) => {
                    // error handling logic here
                }

                Poll::Pending => this.commitment_task = Some(task),
            }
        }

        Poll::Pending
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Condvar, Mutex};

    use futures::task::ArcWake;
    use katana_pool::pool::test_utils::{PoolTx, TestPool};

    use super::*;

    use std::sync::atomic::AtomicBool;

    struct InstantMining {
        is_open: bool,
        transactions: Vec<PoolTx>,
        listener: Receiver<TxHash>,
        should_close: Arc<(Mutex<bool>, Condvar)>,
        waker_registered: AtomicBool,
    }

    impl InstantMining {
        fn new(listener: Receiver<TxHash>) -> Self {
            Self {
                is_open: false,
                listener,
                should_close: Arc::new((Mutex::new(false), Condvar::new())),
                transactions: Vec::new(),
                waker_registered: AtomicBool::new(false),
            }
        }
    }

    impl BlockProducer for InstantMining {
        type Pool = TestPool;

        fn poll_block_open(&mut self, cx: &mut Context<'_>) -> Poll<()> {
            match self.listener.poll_next_unpin(cx) {
                Poll::Pending => Poll::Pending,
                Poll::Ready(_) => {
                    self.is_open = true;
                    Poll::Ready(())
                }
            }
        }

        fn poll_block_close(&mut self, cx: &mut Context<'_>) -> Poll<()> {
            let should_close = self.should_close.clone();

            if *should_close.0.lock().unwrap() {
                self.waker_registered.store(false, atomic::Ordering::Relaxed);
                Poll::Ready(())
            } else {
                if !self.waker_registered.load(atomic::Ordering::Relaxed) {
                    if self
                        .waker_registered
                        .compare_exchange(
                            false,
                            true,
                            atomic::Ordering::Relaxed,
                            atomic::Ordering::Relaxed,
                        )
                        .is_ok()
                    {
                        let waker = cx.waker().clone();
                        std::thread::spawn(move || {
                            let (lock, cvar) = &*should_close;
                            let mut should_close = lock.lock().unwrap();

                            while !*should_close {
                                should_close = cvar.wait(should_close).unwrap();
                            }

                            waker.wake();
                        });
                    }
                }

                Poll::Pending
            }
        }

        fn is_open(&self) -> bool {
            self.is_open
        }

        fn on_open(&mut self) {}

        fn on_transactions(
            &mut self,
            pending_transactions: PendingTransactions<Self::Pool>,
        ) -> ExecutionTask {
            // close the validator to prevent new transactions from being added to the pool

            // drain all the relevant pending transactions from the pool
            self.transactions =
                pending_transactions.map(|tx| tx.tx.as_ref().clone()).collect::<Vec<_>>();

            // indicate that the block should be closed (ie dont take anymore transactions and
            // prepare for block production)
            let (lock, cvar) = &*self.should_close;
            let mut should_close = lock.lock().unwrap();
            *should_close = true;
            cvar.notify_one();

            Box::pin(async move { Ok(()) })
        }

        fn on_close(&mut self) -> BlockCommitmentTask {
            let transactions = std::mem::take(&mut self.transactions);
            let close_flag = self.should_close.clone();
            Box::pin(async move {
                println!("Producing block with total transactions: {}", transactions.len());

                // indicate that the block should be closed (ie dont take anymore transactions and
                // prepare for block production)
                let (lock, cvar) = &*close_flag;
                let mut should_close = lock.lock().unwrap();
                *should_close = false;
                cvar.notify_one();

                Ok(BlockCommitmentOutcome::default())
            })
        }
    }

    struct CustomWaker {
        flag: Arc<atomic::AtomicBool>,
    }

    impl ArcWake for CustomWaker {
        fn wake_by_ref(arc_self: &Arc<Self>) {
            arc_self.flag.store(true, atomic::Ordering::SeqCst);
        }
    }

    #[tokio::test]
    async fn instant_mining() {
        let pool = TestPool::test();

        let miner = InstantMining::new(pool.add_listener());
        let mut task = BlockProductionTask::new(miner, pool.clone());

        // Create a flag to track if the waker was called
        let is_awaken = Arc::new(atomic::AtomicBool::new(false));

        let waker = futures::task::waker(Arc::new(CustomWaker { flag: is_awaken.clone() }));
        let mut cx = Context::from_waker(&waker);

        // Waker should not be called initially
        assert!(!is_awaken.load(atomic::Ordering::SeqCst));
        // no incoming transaction, so should not open a block
        assert!(matches!(task.miner.poll_block_open(&mut cx), Poll::Pending));

        pool.add_transaction(PoolTx::new()).unwrap();
        // Incoming transactions should wake the waker
        assert!(is_awaken.load(atomic::Ordering::SeqCst));
        // Incoming transactions should initiate block opening
        assert!(matches!(task.miner.poll_block_open(&mut cx), Poll::Ready(())));

        // poll the future to completion as we dont care about the result here
        let _ = task.miner.on_transactions(pool.take_transactions()).await;

        // Waker should not be called before poll_block_close
        assert!(matches!(task.miner.poll_block_close(&mut cx), Poll::Ready(_)));
        // Waker should be called when poll_block_close returns Ready
        assert!(is_awaken.load(atomic::Ordering::SeqCst));

        // Test on_close
        let commitment_task = task.miner.on_close();
        assert!(matches!(futures::poll!(commitment_task), Poll::Ready(Ok(_))));
    }

    #[tokio::test]
    async fn test_block_production_task_flow() {
        let pool = TestPool::test();
        let miner = InstantMining::new(pool.add_listener());
        let mut task = BlockProductionTask::new(miner, pool.clone());

        let waker = futures::task::noop_waker();
        let mut cx = Context::from_waker(&waker);

        // Initially, no block should be open
        assert!(!task.miner.is_open());

        // Add a transaction to trigger block opening
        pool.add_transaction(PoolTx::new()).unwrap();

        // Poll the task
        assert!(matches!(Future::poll(Pin::new(&mut task), &mut cx), Poll::Pending));

        // Block should now be open
        assert!(task.miner.is_open());

        // Poll again to process transactions
        assert!(matches!(Future::poll(Pin::new(&mut task), &mut cx), Poll::Pending));

        // Block should be closed after processing
        assert!(!task.miner.is_open());

        // Poll one more time to complete the cycle
        assert!(matches!(Future::poll(Pin::new(&mut task), &mut cx), Poll::Pending));

        // Ensure the block is still closed
        assert!(!task.miner.is_open());
    }
}
