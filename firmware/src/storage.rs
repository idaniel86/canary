use core::ops::Range;
use embassy_embedded_hal::adapter;
use embassy_stm32::flash;
use sequential_storage::{
    cache::{Cache, Uncached, page_pointers::ArrayPagePointers, page_states::ArrayPageStates},
    map::{MapConfig, MapStorage, PostcardValue},
};

use crate::quality::QualityScoreConfig;
impl<'a> PostcardValue<'a> for QualityScoreConfig {}

const FLASH_RANGE: Range<u32> = 0x001FC000..0x00200000;
type Flash<'d> = adapter::BlockingAsync<flash::Flash<'d, flash::Blocking>>;

/// Generic storage for different configurations
pub struct Storage<'d> {
    inner:
        MapStorage<u32, Flash<'d>, Cache<ArrayPageStates<2>, ArrayPagePointers<2>, Uncached, u32>>,
}

impl<'d> Storage<'d> {
    /// Creates a new instance of the storage with the given flash memory.
    /// 
    /// # Arguments
    /// * `flash` - The flash memory to be used for storage.
    /// 
    /// # Returns
    /// * `Self` - A new instance of the storage.
    pub fn new(flash: Flash<'d>) -> Self {
        let inner = MapStorage::new(
            flash,
            const { MapConfig::new(FLASH_RANGE) },
            Cache::new(
                ArrayPageStates::<2>::new(),
                ArrayPagePointers::<2>::new(),
                Uncached,
            ),
        );
        Self { inner }
    }

    /// Sets the quality score configuration in the storage.
    ///
    /// # Arguments
    /// * `config` - A reference to the quality score configuration to be stored.
    ///
    /// # Returns
    /// * `Result<(), sequential_storage::Error<flash::Error>>` - Ok if the operation was successful, Err otherwise.
    pub async fn set_score_config(
        &mut self,
        config: &QualityScoreConfig,
    ) -> Result<(), sequential_storage::Error<flash::Error>> {
        // TODO: verify the size of the buffer is sufficient for the serialized data
        let mut buffer = [0u8; 512];
        self.inner.store_item(&mut buffer, &0, config).await
    }

    /// Retrieves the quality score configuration from the storage.
    ///
    /// # Returns
    /// * `Result<Option<QualityScoreConfig>, sequential_storage::Error<flash::Error>>` - Ok with the configuration if it exists, Err otherwise.
    pub async fn get_score_config(
        &mut self,
    ) -> Result<Option<QualityScoreConfig>, sequential_storage::Error<flash::Error>> {
        // TODO: verify the size of the buffer is sufficient for the serialized data
        let mut buffer = [0u8; 512];
        self.inner.fetch_item(&mut buffer, &0).await
    }
}
