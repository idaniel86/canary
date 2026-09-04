use crate::quality::{QualityScore, QualityScoreConfig};
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, mutex::Mutex};

#[derive(Debug, Clone, Copy)]
pub struct AppState<'a> {
    pub quality_score: &'a Mutex<NoopRawMutex, QualityScore>,
    pub quality_score_config: &'a Mutex<NoopRawMutex, QualityScoreConfig>,
}

impl<'a> AppState<'a> {
    pub fn new(
        quality_score: &'a Mutex<NoopRawMutex, QualityScore>,
        quality_score_config: &'a Mutex<NoopRawMutex, QualityScoreConfig>,
    ) -> Self {
        Self {
            quality_score,
            quality_score_config,
        }
    }
}

pub struct QualityScoreConfigState<'a>(pub &'a Mutex<NoopRawMutex, QualityScoreConfig>);

impl<'a> picoserve::extract::FromRef<AppState<'a>> for QualityScoreConfigState<'a> {
    fn from_ref(app_state: &AppState<'a>) -> Self {
        Self(app_state.quality_score_config)
    }
}

pub struct QualityScoreState<'a>(pub &'a Mutex<NoopRawMutex, QualityScore>);

impl<'a> picoserve::extract::FromRef<AppState<'a>> for QualityScoreState<'a> {
    fn from_ref(app_state: &AppState<'a>) -> Self {
        Self(app_state.quality_score)
    }
}
