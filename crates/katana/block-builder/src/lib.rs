use std::{
    future::Future,
    marker::PhantomData,
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

struct Permit {}

impl Permit {
    fn lock(&self) {}
}

struct TxStream {}

impl TxStream {
    fn drain(&self) {}
}

struct Executor {}

impl Executor {
    fn execute(&self, txs: ()) {}
}

// creates a new block every time a new transaction comes in
struct AutoMining {
    permit: Permit,
    executor: Executor,
    pending_txs_receiver: TxStream,
}

impl AutoMining {
    fn mine(permit: Permit, stream: TxStream, executor: Executor) {
        // lock the permit; prevent certain operations from running when a block is being produced
        let lock = permit.lock();

        // drain the pending txs stream
        let txs = stream.drain();

        // create a new block
        let block = executor.execute(txs);
    }
}

struct BlockProductionTask<T, TM> {
    stream: TxStream,
    task_manager: TM,
    block_producer: Option<T>,
}

impl<T, TM> Future for BlockProductionTask<T, TM>
where
    T: BlockProducer,
    TM: Unpin,
{
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();

        if this.block_producer.poll_task_start(cx).is_ready() {}

        todo!()
    }
}

trait BlockProducer: Unpin {
    // determines if a new block should be produced
    fn poll_task_start(&mut self, cx: &mut Context<'_>) -> Poll<()>;
}

struct IntervalMining {
    timer: tokio::time::Interval,
}

// impl BlockProducer for IntervalMining {
//     fn poll_task_start(&mut self, cx: &mut Context<'_>) {
//         self.timer.poll_tick(cx);
//     }
// }
