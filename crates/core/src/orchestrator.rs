use anyhow::Result;
use tokio::time::{sleep, Duration};
use tracing::{error, info};

/// Polling schedule: (iteration_count, interval).
const POLL_SCHEDULE: &[(u32, Duration)] = &[
    (8, Duration::from_secs(15)),  // first ~2 min at 15s
    (16, Duration::from_secs(30)), // next ~8 min at 30s
];

/// Async callback interface for the orchestrator so it stays decoupled from
/// the storage crate (avoids a circular dependency).
#[async_trait::async_trait]
pub trait RunBackend: Send + Sync {
    async fn poll_once(&self, run_id: i64) -> Result<()>;
    async fn all_matched(&self, run_id: i64) -> bool;
    async fn is_cancelled(&self, run_id: i64) -> bool;
    async fn finalize(&self, run_id: i64) -> Result<()>;
}

pub struct RunOrchestrator<B> {
    backend: B,
}

impl<B: RunBackend> RunOrchestrator<B> {
    pub fn new(backend: B) -> Self {
        Self { backend }
    }

    pub async fn run(&self, run_id: i64) -> Result<()> {
        info!("Orchestrator starting run {}", run_id);

        'outer: for &(count, interval) in POLL_SCHEDULE {
            for _ in 0..count {
                if self.backend.is_cancelled(run_id).await {
                    info!("Run {} cancelled", run_id);
                    return Ok(());
                }
                if let Err(e) = self.backend.poll_once(run_id).await {
                    error!("Poll error for run {}: {}", run_id, e);
                }
                if self.backend.all_matched(run_id).await {
                    break 'outer;
                }
                sleep(interval).await;
            }
        }

        self.backend.finalize(run_id).await?;
        info!("Run {} complete", run_id);
        Ok(())
    }
}
