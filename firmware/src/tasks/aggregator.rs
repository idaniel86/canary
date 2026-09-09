use crate::tasks::Quality;
use crate::tasks::ReadingChannel;
use crate::tasks::SharedState;

#[embassy_executor::task]
pub async fn aggregator_task(state: &'static SharedState, receiver: &'static ReadingChannel) {
    loop {
        let reading = receiver.receive().await;
        let mut lock = state.quality.lock().await;
        let Quality { subscores, score_config , ..} = &mut *lock;
        subscores.update(reading, &score_config);
    }
}
