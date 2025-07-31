use crate::{
    common::Task,
    field_sets::Ldswstatus,
    gpios::{Gpio, GpioMode, GpioPolarity},
    Ldsw1Activedischarge, Ldsw1Ldosel, Ldsw1Softstartdisable, Ldsw1Softstartsel,
    Ldsw2Activedischarge, Ldsw2Ldosel, Ldsw2Softstartdisable, Ldsw2Softstartsel,
};

mod types;

// Re-export everything in types.rs
pub use types::*;

/// Convert a GPIO enum value to its register index
///
/// GPIOs are 1-indexed in the nPM1300 so we subtract 1 from the GPIO number
/// to get the register index
fn gpio_to_register_index(gpio: Gpio) -> usize {
    usize::from(u8::from(gpio) - 1)
}

pub struct Config {
    /// GPIO to enable/disable LDSW regulators
    pub gpio_ldsw_enable_control: Gpio,
    /// GPIO enable/disable polarity
    pub gpio_ldsw_enable_control_polarity: GpioPolarity,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            gpio_ldsw_enable_control: Gpio::None,
            gpio_ldsw_enable_control_polarity: GpioPolarity::NotInverted,
        }
    }
}

impl<I2c: embedded_hal_async::i2c::I2c, Delay: embedded_hal_async::delay::DelayNs>
    crate::NPM1300<I2c, Delay>
{
    /// Enable LDSW1
    pub async fn enable_ldsw1(&mut self) -> Result<(), crate::NPM1300Error<I2c::Error>> {
        self.device
            .ldsw()
            .taskldsw_1_set()
            .dispatch_async(|command| command.set_taskldsw_1_set(Task::Trigger))
            .await
    }

    /// Disable LDSW1
    pub async fn disable_ldsw1(&mut self) -> Result<(), crate::NPM1300Error<I2c::Error>> {
        self.device
            .ldsw()
            .taskldsw_1_clr()
            .dispatch_async(|command| command.set_taskldsw_1_clr(Task::Trigger))
            .await
    }

    /// Enable LDSW2
    pub async fn enable_ldsw2(&mut self) -> Result<(), crate::NPM1300Error<I2c::Error>> {
        self.device
            .ldsw()
            .taskldsw_2_set()
            .dispatch_async(|command| command.set_taskldsw_2_set(Task::Trigger))
            .await
    }

    /// Disable LDSW2
    pub async fn disable_ldsw2(&mut self) -> Result<(), crate::NPM1300Error<I2c::Error>> {
        self.device
            .ldsw()
            .taskldsw_2_clr()
            .dispatch_async(|command| command.set_taskldsw_2_clr(Task::Trigger))
            .await
    }

    /// Get LDSW status
    pub async fn get_ldsw_status(&mut self) -> Result<Ldswstatus, crate::NPM1300Error<I2c::Error>> {
        self.device.ldsw().ldswstatus().read_async().await
    }

    /// Configure LDSW1 GPIO control
    ///
    /// # Arguments
    /// * `gpio` - GPIO to control LDSW1
    /// * `polarity` - Polarity of GPIO
    pub async fn set_ldsw1_gpio_control(
        &mut self,
        gpio: Gpio,
        polarity: GpioPolarity,
    ) -> Result<(), crate::NPM1300Error<I2c::Error>> {
        if gpio != Gpio::None {
            // Configure GPIO mode as input
            self.device
                .gpios()
                .gpiomode(gpio_to_register_index(gpio))
                .write_async(|reg| reg.set_gpiomode(GpioMode::GpiInput))
                .await?;
        }

        // Configure GPIO and its polarity
        self.device
            .ldsw()
            .ldsw_1_gpisel()
            .write_async(|reg| {
                reg.set_ldsw_1_gpisel(gpio);
                reg.set_ldsw_1_gpiinv(polarity);
            })
            .await
    }

    /// Configure LDSW2 GPIO control
    ///
    /// # Arguments
    /// * `gpio` - GPIO to control LDSW2
    /// * `polarity` - Polarity of GPIO
    pub async fn set_ldsw2_gpio_control(
        &mut self,
        gpio: Gpio,
        polarity: GpioPolarity,
    ) -> Result<(), crate::NPM1300Error<I2c::Error>> {
        if gpio != Gpio::None {
            // Configure GPIO mode as input
            self.device
                .gpios()
                .gpiomode(gpio_to_register_index(gpio))
                .write_async(|reg| reg.set_gpiomode(GpioMode::GpiInput))
                .await?;
        }

        // Configure GPIO and its polarity
        self.device
            .ldsw()
            .ldsw_2_gpisel()
            .write_async(|reg| {
                reg.set_ldsw_2_gpisel(gpio);
                reg.set_ldsw_2_gpiinv(polarity);
            })
            .await
    }

    /// Set LDSW1 mode (Load Switch or LDO)
    ///
    /// # Arguments
    /// * `mode` - The mode to set for LDSW1
    pub async fn set_ldsw1_mode(
        &mut self,
        mode: Ldsw1Ldosel,
    ) -> Result<(), crate::NPM1300Error<I2c::Error>> {
        self.device
            .ldsw()
            .ldsw_1_ldosel()
            .write_async(|reg| reg.set_ldsw_1_ldosel(mode))
            .await
    }

    /// Set LDSW2 mode (Load Switch or LDO)
    ///
    /// # Arguments
    /// * `mode` - The mode to set for LDSW2
    pub async fn set_ldsw2_mode(
        &mut self,
        mode: Ldsw2Ldosel,
    ) -> Result<(), crate::NPM1300Error<I2c::Error>> {
        self.device
            .ldsw()
            .ldsw_2_ldosel()
            .write_async(|reg| reg.set_ldsw_2_ldosel(mode))
            .await
    }

    /// Set LDSW1 LDO output voltage
    ///
    /// # Arguments
    /// * `voltage` - The voltage to set for LDO1
    pub async fn set_ldsw1_ldo_voltage(
        &mut self,
        voltage: LdoVoltage,
    ) -> Result<(), crate::NPM1300Error<I2c::Error>> {
        self.device
            .ldsw()
            .ldsw_1_voutsel()
            .write_async(|reg| reg.set_ldsw_1_voutsel(voltage))
            .await
    }

    /// Set LDSW2 LDO output voltage
    ///
    /// # Arguments
    /// * `voltage` - The voltage to set for LDO2
    pub async fn set_ldsw2_ldo_voltage(
        &mut self,
        voltage: LdoVoltage,
    ) -> Result<(), crate::NPM1300Error<I2c::Error>> {
        self.device
            .ldsw()
            .ldsw_2_voutsel()
            .write_async(|reg| reg.set_ldsw_2_voutsel(voltage))
            .await
    }

    /// Configure LDSW1 soft start
    ///
    /// # Arguments
    /// * `disable` - Whether to disable soft start
    /// * `current_limit` - Soft start current limit
    pub async fn configure_ldsw1_soft_start(
        &mut self,
        disable: Ldsw1Softstartdisable,
        current_limit: Ldsw1Softstartsel,
    ) -> Result<(), crate::NPM1300Error<I2c::Error>> {
        self.device
            .ldsw()
            .ldswconfig()
            .modify_async(|reg| {
                reg.set_ldsw_1_softstartdisable(disable);
                reg.set_ldsw_1_softstartsel(current_limit);
            })
            .await
    }

    /// Configure LDSW2 soft start
    ///
    /// # Arguments
    /// * `disable` - Whether to disable soft start
    /// * `current_limit` - Soft start current limit
    pub async fn configure_ldsw2_soft_start(
        &mut self,
        disable: Ldsw2Softstartdisable,
        current_limit: Ldsw2Softstartsel,
    ) -> Result<(), crate::NPM1300Error<I2c::Error>> {
        self.device
            .ldsw()
            .ldswconfig()
            .modify_async(|reg| {
                reg.set_ldsw_2_softstartdisable(disable);
                reg.set_ldsw_2_softstartsel(current_limit);
            })
            .await
    }

    /// Configure LDSW1 active discharge
    ///
    /// # Arguments
    /// * `enable` - Whether to enable active discharge
    pub async fn set_ldsw1_active_discharge(
        &mut self,
        enable: Ldsw1Activedischarge,
    ) -> Result<(), crate::NPM1300Error<I2c::Error>> {
        self.device
            .ldsw()
            .ldswconfig()
            .modify_async(|reg| reg.set_ldsw_1_activedischarge(enable))
            .await
    }

    /// Configure LDSW2 active discharge
    ///
    /// # Arguments
    /// * `enable` - Whether to enable active discharge
    pub async fn set_ldsw2_active_discharge(
        &mut self,
        enable: Ldsw2Activedischarge,
    ) -> Result<(), crate::NPM1300Error<I2c::Error>> {
        self.device
            .ldsw()
            .ldswconfig()
            .modify_async(|reg| reg.set_ldsw_2_activedischarge(enable))
            .await
    }
}
