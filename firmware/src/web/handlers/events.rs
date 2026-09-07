use defmt::{error, info};
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, mutex::Mutex};
use embassy_time::Timer;
use picoserve::{
    extract::State,
    response::{
        EventStream, IntoResponse, Json,
        sse::{EventSource, EventWriter},
    },
};

use crate::{quality::QualityScore, web::state::QualityScoreState};

struct QualityScoreEvents<'d> {
    quality_score: &'d Mutex<NoopRawMutex, QualityScore>,
}

impl<'d> EventSource for QualityScoreEvents<'d> {
    async fn write_events<W: picoserve::io::Write>(
        self,
        mut writer: EventWriter<'_, W>,
    ) -> Result<(), W::Error> {
        loop {
            let quality_score = self.quality_score.lock().await.clone();
            if let Err(_) = writer
                .write_event("quality_score", Json(&quality_score))
                .await
            {
                error!("Failed to write quality_score event");
                writer.write_keepalive().await?;
            }

            Timer::after_secs(10).await;
        }
    }
}

pub async fn get_events(
    State(QualityScoreState(state)): State<QualityScoreState<'_>>,
) -> impl IntoResponse {
    EventStream(QualityScoreEvents {
        quality_score: state,
    })
}
