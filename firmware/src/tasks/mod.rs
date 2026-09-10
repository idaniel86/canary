mod aggregator;
mod bme688;
mod ics43434;
mod net;
mod opt3001;
mod sai;
mod scd41;
mod storage;
mod web;

pub use aggregator::aggregator_task;
pub use bme688::bme688_task;
pub use ics43434::ics43434_task;
pub use net::net_task;
pub use opt3001::opt3001_task;
pub use sai::{AudioChannel, sai_task};
pub use scd41::scd41_task;
pub use storage::storage_task;
pub use web::{WEB_TASK_POOL_SIZE, web_task};

use crate::quality::AnyQualityModel;
use crate::quality::{ScoreConfig, Subscores};

pub struct Quality {
    pub subscores: Subscores,
    pub model: AnyQualityModel,
    pub score_config: ScoreConfig,
}

/// The capacity of the channel used for sending sensor readings between tasks.
const READING_CHANNEL_CAPACITY: usize = 8;

/// A channel for sending sensor readings between tasks.
pub type ReadingChannel = embassy_sync::channel::Channel<
    embassy_sync::blocking_mutex::raw::NoopRawMutex,
    crate::quality::Reading,
    READING_CHANNEL_CAPACITY,
>;

/// A signal used to notify tasks about updates to the storage.
pub type StorageSignal =
    embassy_sync::signal::Signal<embassy_sync::blocking_mutex::raw::NoopRawMutex, ()>;

/// Shared state containing the current score, score configuration, and a storage signal.
pub struct SharedState {
    /// The current quality, including the score and its configuration.
    pub quality:
        embassy_sync::mutex::Mutex<embassy_sync::blocking_mutex::raw::NoopRawMutex, Quality>,
    /// The signal used to notify tasks about updates to the storage.
    pub storage_signal: StorageSignal,
}
