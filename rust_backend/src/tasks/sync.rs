use loco_rs::prelude::*;
use crate::workers::product_sync::Worker;
use loco_rs::bgworker::BackgroundWorker;
pub struct Sync;
#[async_trait]
impl task::Task for Sync {
    fn task(&self) -> TaskInfo {
        TaskInfo {
            name: "sync".to_string(),
            detail: "Task generator".to_string(),
        }
    }
    async fn run(&self, ctx: &AppContext, _vars: &task::Vars) -> Result<()> {
        println!("📡 Enviando orden de sincronización a la cola...");

        
        Worker::perform_later(ctx, crate::workers::product_sync::WorkerArgs {})
            .await?;

        println!("✅ Orden enviada satisfactoriamente.");
        Ok(())
    }

}
