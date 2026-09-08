use defmt::*;

const CAP: usize = 2;
const SUBSCRIBERS: usize = 2;
const AUDIO_SAMPLE_SIZE: usize = 1024;

pub type AudioSample = [u32; AUDIO_SAMPLE_SIZE];
pub type AudioChannel = embassy_sync::pubsub::PubSubChannel<
    embassy_sync::blocking_mutex::raw::NoopRawMutex,
    AudioSample,
    CAP,
    SUBSCRIBERS,
    1,
>;
pub type AudioSai = embassy_stm32::sai::Sai<'static, embassy_stm32::peripherals::SAI1, u32>;

#[embassy_executor::task]
pub async fn sai_task(mut sai: AudioSai, channel: &'static AudioChannel) {
    let mut sample: AudioSample = [0u32; AUDIO_SAMPLE_SIZE];

    loop {
        if let Err(e) = sai.read(&mut sample).await {
            error!("SAI read error: {:?}", e);
        } else {
            if let Err(e) = channel.immediate_publisher().try_publish(sample) {
                error!("Failed to publish audio sample: {:?}", e);
            }
        }
    }
}
