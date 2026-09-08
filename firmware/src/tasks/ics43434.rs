use super::sai::AudioChannel;
use crate::ics43434;
use crate::quality::Reading;
use crate::tasks::ReadingChannel;
use embassy_futures::select::{Either, select};
use embassy_time::{Duration, Ticker};

#[embassy_executor::task]
pub async fn ics43434_task(
    period: Duration,
    channel: &'static AudioChannel,
    sender: &'static ReadingChannel,
) {
    let sender = sender.sender();
    let mut ics_43434 = ics43434::Ics43434::new();
    let mut subscriber = channel.subscriber().unwrap();
    let mut ticker = Ticker::every(period);

    loop {
        // The pattern is cancel safe, no future is dropped
        match select(subscriber.next_message_pure(), ticker.next()).await {
            Either::First(frame) => {
                for sample in &frame {
                    ics_43434.process(*sample);
                }
            }
            Either::Second(_) => {
                sender.send(Reading::Noise(ics_43434.get_spl())).await;
            }
        }
    }
}
