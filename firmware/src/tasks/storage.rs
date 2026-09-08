use crate::storage::Storage;
use crate::tasks::SharedState;
use defmt::*;

#[embassy_executor::task]
pub async fn storage_task(mut storage: Storage<'static>, state: &'static SharedState) {
    loop {
        let _ = state.storage_signal.wait().await;
        let lock = state.quality.lock().await;
        let config = &(*lock).score_config;
        if let Err(e) = storage.set_score_config(config).await {
            error!("Failed to set score config: {:?}", e);
        }
    }
}
