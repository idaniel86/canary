use defmt::*;
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, mutex::Mutex};
use embassy_time::Timer;
use picoserve::{
    extract::State,
    response::{
        EventStream, IntoResponse, Json,
        sse::{EventSource, EventWriter},
    },
};

use crate::{tasks::Quality, web::state::QualityState};

struct QualityScoreEvents<'d> {
    quality_score: &'d Mutex<NoopRawMutex, Quality>,
}

impl<'d> EventSource for QualityScoreEvents<'d> {
    async fn write_events<W: picoserve::io::Write>(
        self,
        mut writer: EventWriter<'_, W>,
    ) -> Result<(), W::Error> {
        loop {
            let score = {
                let quality = self.quality_score.lock().await;
                quality
                    .subscores
                    .snapshot(&quality.model, &quality.score_config)
            };
            if let Err(_) = writer.write_event("quality_score", Json(&score)).await {
                error!("Failed to write quality_score event");
                writer.write_keepalive().await?;
            }

            Timer::after_secs(10).await;
        }
    }
}

pub async fn get_events(State(QualityState(state)): State<QualityState<'_>>) -> impl IntoResponse {
    EventStream(QualityScoreEvents {
        quality_score: state,
    })
}
