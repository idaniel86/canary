#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror_no_std::Error)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Error<E>
where
    E: embedded_hal_async::i2c::Error,
{
    #[error("I2C communication error")]
    I2c(#[from] E),
    #[error("Unexpected chip ID: {0:#X}")]
    UnexpectedChipId(u8),
    #[error("Unexpected variant ID: {0:#X}")]
    UnexpectedVariantId(u8),
    #[error("Sleep mode timeout")]
    SleepModeTimeout,
}
