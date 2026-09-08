use crate::tasks::Quality;
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, mutex::Mutex, signal::Signal};

#[derive(Clone, Copy)]
pub struct AppState<'a> {
    pub quality: &'a Mutex<NoopRawMutex, Quality>,
    pub storage_signal: &'a Signal<NoopRawMutex, ()>,
}

impl<'a> AppState<'a> {
    pub fn new(
        quality: &'a Mutex<NoopRawMutex, Quality>,
        storage_signal: &'a Signal<NoopRawMutex, ()>,
    ) -> Self {
        Self {
            quality,
            storage_signal,
        }
    }
}

pub struct QualityState<'a>(pub &'a Mutex<NoopRawMutex, Quality>);

impl<'a> picoserve::extract::FromRef<AppState<'a>> for QualityState<'a> {
    fn from_ref(app_state: &AppState<'a>) -> Self {
        Self(app_state.quality)
    }
}
