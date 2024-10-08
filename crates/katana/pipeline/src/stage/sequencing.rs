use std::sync::Arc;

use anyhow::Result;
use futures::future;
use katana_core::backend::Backend;
use katana_core::service::block_producer::BlockProducer;
use katana_core::service::messaging::{MessagingConfig, MessagingService, MessagingTask};
use katana_core::service::{BlockProductionTask, TransactionMiner};
use katana_executor::ExecutorFactory;
use katana_pool::{TransactionPool, TxPool};
use katana_tasks::TaskManager;
use tracing::trace;

use super::{StageId, StageResult};
use crate::Stage;

/// The sequencing stage is responsible for advancing the chain state.
#[allow(missing_debug_implementations)]
pub struct Sequencing<EF: ExecutorFactory> {
    pool: TxPool,
    backend: Arc<Backend<EF>>,
    task_manager: TaskManager,
    block_producer: BlockProducer<EF>,
    messaging_config: Option<MessagingConfig>,
}

impl<EF: ExecutorFactory> Sequencing<EF> {
    pub fn new(
        pool: TxPool,
        backend: Arc<Backend<EF>>,
        task_manager: TaskManager,
        block_producer: BlockProducer<EF>,
        messaging_config: Option<MessagingConfig>,
    ) -> Self {
        Self { pool, backend, task_manager, block_producer, messaging_config }
    }

    async fn run_messaging(&self) -> Result<()> {
        if let Some(config) = &self.messaging_config {
            let config = config.clone();
            let pool = self.pool.clone();
            let backend = self.backend.clone();

            let service = MessagingService::new(config, pool, backend).await?;
            let task = MessagingTask::new(service);

            self.task_manager.build_task().critical().name("Messaging").spawn(task);
            trace!(target: "pipeline", "Messaging task started");
        } else {
            trace!(target: "pipeline", "No messaging configuration provided, skipping messaging task");
        }
        Ok(())
    }

    async fn run_block_production(&self) {
        let pool = self.pool.clone();
        let miner = TransactionMiner::new(pool.add_listener());
        let block_producer = self.block_producer.clone();

        let service = BlockProductionTask::new(pool, miner, block_producer);
        self.task_manager.build_task().critical().name("Block production").spawn(service);
        trace!(target: "pipeline", "Block production task started");
    }
}

#[async_trait::async_trait]
impl<EF: ExecutorFactory> Stage for Sequencing<EF> {
    fn id(&self) -> StageId {
        StageId::Sequencing
    }

    #[tracing::instrument(skip_all, name = "Stage", fields(id = %self.id()))]
    async fn execute(&mut self) -> StageResult {
        let _ = self.run_messaging().await?;
        let _ = self.run_block_production().await;
        future::pending::<StageResult>().await
    }
}
