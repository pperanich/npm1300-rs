#![cfg_attr(not(test), no_std)]

use device_driver::AsyncRegisterInterface;

pub mod common;

pub mod adc;
pub mod buck;
pub mod charger;
pub mod gpios;
pub mod ldsw;
pub mod leds;
pub mod pof;
pub mod reset;
pub mod ship;
pub mod sysreg;
pub mod timer;

const ADDR: u8 = 0x6B;

#[derive(thiserror::Error, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
pub enum NPM1300Error<I2cError> {
    #[error("i2c error: {0:?}")]
    I2c(I2cError),
    #[error("charger current {0:?} is too high")]
    ChargerCurrentTooHigh(u16),
    #[error("invalid NTC threshold")]
    InvalidNtcThreshold,
    #[error("invalid die temperature stop/resume threshold")]
    InvalidDieTemperatureThreshold,
    #[error("invalid NTC beta")]
    InvalidNtcBeta,
    #[error(
        "invalid VBAT measurement delay value, it must be between 4 and 514 and a multiple of 2"
    )]
    InvalidVbatMeasurementDelayValue,
    #[error("invalid VSYS threshold")]
    InvalidPofVsysThreshold,
    #[error("invalid timer configuration")]
    InvalidConfiguration,
    #[error("timer not configured")]
    TimerNotConfigured,
    #[error("invalid voltage range")]
    InvalidVoltageRange,
    #[error("invalid current range")]
    InvalidCurrentRange,
    #[error("threshold too low")]
    ThresholdTooLow,
    #[error("threshold too high")]
    ThresholdTooHigh,
    #[error("mode not supported")]
    ModeNotSupported,
    #[error("invalid GPIO pin number")]
    InvalidGpioPin,
    #[error("invalid timer value")]
    InvalidTimerValue,
    #[error("invalid register value")]
    InvalidRegisterValue,
}

#[derive(Debug)]
pub struct DeviceInterface<I2c: embedded_hal_async::i2c::I2c> {
    pub i2c: I2c,
}

pub struct NPM1300<I2c: embedded_hal_async::i2c::I2c, Delay: embedded_hal_async::delay::DelayNs> {
    device: Device<DeviceInterface<I2c>>,
    delay: Delay,
    ntc_beta: Option<f32>,
}

// Validation helper functions
impl<I2cError> NPM1300Error<I2cError> {
    /// Create a validation error for voltage range
    pub fn voltage_out_of_range(voltage: f32, min: f32, max: f32) -> Self {
        if voltage < min {
            Self::ThresholdTooLow
        } else if voltage > max {
            Self::ThresholdTooHigh
        } else {
            Self::InvalidVoltageRange
        }
    }

    /// Create a validation error for current range
    pub fn current_out_of_range(current: u16, max: u16) -> Self {
        if current > max {
            Self::InvalidCurrentRange
        } else {
            Self::InvalidCurrentRange
        }
    }
}

impl<I2c: embedded_hal_async::i2c::I2c, Delay: embedded_hal_async::delay::DelayNs>
    NPM1300<I2c, Delay>
{
    pub fn new(i2c: I2c, delay: Delay) -> Self {
        Self {
            device: Device::new(DeviceInterface { i2c }),
            delay,
            ntc_beta: None,
        }
    }
}

device_driver::create_device!(
    device_name: Device,
    manifest: "device.yaml"
);

impl<I2c: embedded_hal_async::i2c::I2c> device_driver::AsyncRegisterInterface
    for DeviceInterface<I2c>
{
    type AddressType = u16;

    type Error = NPM1300Error<I2c::Error>;

    async fn write_register(
        &mut self,
        address: Self::AddressType,
        _size_bits: u32,
        data: &[u8],
    ) -> Result<(), Self::Error> {
        let buf = [(address >> 8) as u8, address as u8, data[0]];
        self.i2c.write(ADDR, &buf).await.map_err(NPM1300Error::I2c)
    }

    async fn read_register(
        &mut self,
        address: Self::AddressType,
        _size_bits: u32,
        data: &mut [u8],
    ) -> Result<(), Self::Error> {
        self.i2c
            .write_read(ADDR, &[(address >> 8) as u8, address as u8], data)
            .await
            .map_err(NPM1300Error::I2c)
    }
}

impl<I2c: embedded_hal_async::i2c::I2c> device_driver::AsyncCommandInterface
    for DeviceInterface<I2c>
{
    type AddressType = u16;

    type Error = NPM1300Error<I2c::Error>;

    async fn dispatch_command(
        &mut self,
        address: Self::AddressType,
        size_bits_in: u32,
        input: &[u8],
        _size_bits_out: u32,
        _output: &mut [u8],
    ) -> Result<(), Self::Error> {
        self.write_register(address, size_bits_in, input).await
    }
}
