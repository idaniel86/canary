use crate::filters;
use crate::quality::Reading;
use crate::tasks::Quality;
use crate::tasks::ReadingChannel;
use crate::tasks::SharedState;

#[embassy_executor::task]
pub async fn aggregator_task(state: &'static SharedState, receiver: &'static ReadingChannel) {
    let lock = state.quality.lock().await;
    let config = &((*lock).score_config);
    let mut temperature_filter =
        filters::LowPassFilter::new(config.temperature.filter_tau_seconds, None);
    let mut humidity_filter = filters::LowPassFilter::new(config.humidity.filter_tau_seconds, None);
    let mut co2_filter = filters::LowPassFilter::new(config.co2.filter_tau_seconds, None);
    let mut illuminance_filter =
        filters::LowPassFilter::new(config.illuminance.filter_tau_seconds, None);
    let mut noise_filter = filters::LowPassFilter::new(config.noise.filter_tau_seconds, None);
    drop(lock);

    loop {
        let reading = receiver.receive().await;
        match reading {
            Reading::Temperature(value) => {
                let filtered = temperature_filter.process(value);
                let mut lock = state.quality.lock().await;
                let Quality {
                    score,
                    score_config,
                } = &mut *lock;
                score.update_temperature(filtered, score_config);
            }
            Reading::Humidity(value) => {
                let filtered = humidity_filter.process(value);
                let mut lock = state.quality.lock().await;
                let Quality {
                    score,
                    score_config,
                } = &mut *lock;
                score.update_humidity(filtered, score_config);
            }
            Reading::Co2(value) => {
                let filtered = co2_filter.process(value);
                let mut lock = state.quality.lock().await;
                let Quality {
                    score,
                    score_config,
                } = &mut *lock;
                score.update_co2(filtered, score_config);
            }
            Reading::Illuminance(value) => {
                let filtered = illuminance_filter.process(value);
                let mut lock = state.quality.lock().await;
                let Quality {
                    score,
                    score_config,
                } = &mut *lock;
                score.update_illuminance(filtered, score_config);
            }
            Reading::Noise(value) => {
                let filtered = noise_filter.process(value);
                let mut lock = state.quality.lock().await;
                let Quality {
                    score,
                    score_config,
                } = &mut *lock;
                score.update_noise(filtered, score_config);
            }
        }
    }
}
