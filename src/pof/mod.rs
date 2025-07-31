mod types;

// Re-export everything in types.rs
pub use types::*;

use crate::{Pofena, Pofwarnpolarity, VsysThreshold};

impl<I2c: embedded_hal_async::i2c::I2c, Delay: embedded_hal_async::delay::DelayNs>
    crate::NPM1300<I2c, Delay>
{
    /// Enable or disable power-failure detection
    ///
    /// # Arguments
    ///
    /// * `enable` - true to enable power-failure detection, false to disable it
    pub async fn enable_power_failure_detection(
        &mut self,
        enable: bool,
    ) -> Result<(), crate::NPM1300Error<I2c::Error>> {
        self.device
            .pof()
            .pofconfig()
            .modify_async(|reg| reg.set_pofena(if enable { Pofena::Enabled } else { Pofena::Off }))
            .await
    }

    /// Check if power failure detection is enabled
    ///
    /// # Returns
    ///
    /// Returns `true` if power failure detection is enabled, `false` if disabled, or
    /// Err(NPM1300Error::I2c) if there was an error communicating with the device.
    pub async fn is_power_failure_detection_enabled(
        &mut self,
    ) -> Result<bool, crate::NPM1300Error<I2c::Error>> {
        Ok(self.device.pof().pofconfig().read_async().await?.pofena() == Pofena::Enabled)
    }

    pub async fn set_power_failure_warning_gpio_polarity(
        &mut self,
        polarity: Pofwarnpolarity,
    ) -> Result<(), crate::NPM1300Error<I2c::Error>> {
        self.device
            .pof()
            .pofconfig()
            .modify_async(|reg| reg.set_pofwarnpolarity(polarity))
            .await
    }

    /// Get the polarity of the power failure warning GPIO
    ///
    /// This function does not check if a GPIO is configured as a power failure warning
    /// GPIO. It is the caller's responsibility to ensure that the GPIO is configured
    /// correctly.
    ///
    /// # Returns
    ///
    /// Returns the current polarity of the power failure warning GPIO, or
    /// Err(NPM1300Error::I2c) if there was an error communicating with the device.
    pub async fn get_power_failure_warning_gpio_polarity(
        &mut self,
    ) -> Result<Pofwarnpolarity, crate::NPM1300Error<I2c::Error>> {
        Ok(self
            .device
            .pof()
            .pofconfig()
            .read_async()
            .await?
            .pofwarnpolarity())
    }

    /// Set the VSYS (System Voltage) threshold for power failure detection
    ///
    /// # Arguments
    ///
    /// * `threshold` - The VSYS threshold voltage level to trigger power failure detection
    ///
    /// # Safety
    ///
    /// 1. The VSYS threshold must be lower than the current VSYS voltage, otherwise it will
    ///    immediately trigger a power failure event and device reset. This is checked by the driver
    ///    with a 50mV safety margin.
    ///
    /// 2. The VSYS threshold must be higher than the battery undervoltage protection level to
    ///    prevent the protection circuit from triggering. This is NOT checked by the driver
    ///    and must be ensured by the caller.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if the threshold was set successfully, or `Err(NPM1300Error::InvalidPofVsysThreshold)`
    /// if the requested threshold is higher than the current VSYS voltage (with safety margin), or
    /// Err(NPM1300Error::I2c) if there was an error communicating with the device.
    pub async fn set_vsys_threshold(
        &mut self,
        threshold: VsysThreshold,
    ) -> Result<(), crate::NPM1300Error<I2c::Error>> {
        // Check if the threshold is safe with improved validation
        let vsys = self.measure_vsys().await?;
        let threshold_voltage = self.vsys_threshold_to_voltage(threshold);

        // Add 50mV safety margin to prevent immediate POF trigger
        const SAFETY_MARGIN_V: f32 = 0.05;

        // Use epsilon comparison for floating-point values
        const EPSILON: f32 = 0.001;

        if vsys < (threshold_voltage + SAFETY_MARGIN_V + EPSILON) {
            return Err(crate::NPM1300Error::InvalidPofVsysThreshold);
        }

        // Configure the threshold
        self.device
            .pof()
            .pofconfig()
            .modify_async(|reg| reg.set_pofvsysthreshsel(threshold))
            .await
    }

    /// Get the VSYS (System Voltage) threshold for power failure detection
    ///
    /// # Returns
    ///
    /// Returns the current VSYS threshold voltage level, or
    /// Err(NPM1300Error::I2c) if there was an error communicating with the device.
    pub async fn get_vsys_threshold(
        &mut self,
    ) -> Result<VsysThreshold, crate::NPM1300Error<I2c::Error>> {
        // We trust the PMIC to return a valid value
        // We can safely unwrap here because we know the register is valid
        Ok(self
            .device
            .pof()
            .pofconfig()
            .read_async()
            .await?
            .pofvsysthreshsel())
    }

    /// Get current POF status including warning state and VSYS voltage
    ///
    /// This function provides comprehensive POF status monitoring by checking:
    /// - Whether POF detection is enabled
    /// - Current VSYS voltage level
    /// - Whether POF warning would be active based on current conditions
    ///
    /// # Returns
    ///
    /// Returns `Ok(PofStatus)` containing the current POF status, or
    /// `Err(NPM1300Error::I2c)` if there was an error communicating with the device.
    pub async fn get_pof_status(&mut self) -> Result<PofStatus, crate::NPM1300Error<I2c::Error>> {
        // Read POF configuration
        let pof_config = self.device.pof().pofconfig().read_async().await?;
        let pof_enabled = pof_config.pofena() == Pofena::Enabled;
        let threshold = pof_config.pofvsysthreshsel();

        // Measure current VSYS voltage
        let vsys_voltage = self.measure_vsys().await?;

        // Determine if POF warning would be active
        let threshold_voltage = self.vsys_threshold_to_voltage(threshold);
        let warning_active = pof_enabled && vsys_voltage <= threshold_voltage;

        Ok(PofStatus {
            warning_active,
            threshold_configured: pof_enabled,
            vsys_voltage: Some(vsys_voltage),
        })
    }

    /// Check if POF warning is currently active
    ///
    /// This function checks whether a POF warning condition is currently present
    /// by comparing the current VSYS voltage against the configured threshold.
    ///
    /// # Returns
    ///
    /// Returns `Ok(true)` if POF warning is active, `Ok(false)` if not active,
    /// or `Err(NPM1300Error::I2c)` if there was an error communicating with the device.
    pub async fn is_pof_warning_active(&mut self) -> Result<bool, crate::NPM1300Error<I2c::Error>> {
        let status = self.get_pof_status().await?;
        Ok(status.warning_active)
    }

    /// Get the POF threshold voltage in volts
    ///
    /// This function returns the currently configured POF threshold as a voltage value.
    ///
    /// # Returns
    ///
    /// Returns `Ok(f32)` with the threshold voltage in volts, or
    /// `Err(NPM1300Error::I2c)` if there was an error communicating with the device.
    pub async fn get_pof_threshold_voltage(
        &mut self,
    ) -> Result<f32, crate::NPM1300Error<I2c::Error>> {
        let threshold = self.get_vsys_threshold().await?;
        Ok(self.vsys_threshold_to_voltage(threshold))
    }

    /// Check if current VSYS voltage is within safe operating range
    ///
    /// This function verifies that the current VSYS voltage is sufficiently above
    /// the POF threshold to ensure stable operation.
    ///
    /// # Arguments
    ///
    /// * `safety_margin_v` - Additional safety margin in volts (default: 0.1V)
    ///
    /// # Returns
    ///
    /// Returns `Ok(true)` if VSYS is in safe range, `Ok(false)` if approaching threshold,
    /// or `Err(NPM1300Error::I2c)` if there was an error communicating with the device.
    pub async fn is_vsys_voltage_safe(
        &mut self,
        safety_margin_v: Option<f32>,
    ) -> Result<bool, crate::NPM1300Error<I2c::Error>> {
        let margin = safety_margin_v.unwrap_or(0.1);
        let vsys = self.measure_vsys().await?;
        let threshold_voltage = self.get_pof_threshold_voltage().await?;

        Ok(vsys > (threshold_voltage + margin))
    }

    /// Convert VSYS threshold enum to voltage value
    ///
    /// # Arguments
    ///
    /// * `threshold` - The VSYS threshold enum value
    ///
    /// # Returns
    ///
    /// Returns the voltage value in volts corresponding to the threshold
    fn vsys_threshold_to_voltage(&self, threshold: VsysThreshold) -> f32 {
        match threshold {
            VsysThreshold::V28 => 2.8,
            VsysThreshold::V26 => 2.6,
            VsysThreshold::V27 => 2.7,
            VsysThreshold::V29 => 2.9,
            VsysThreshold::V30 => 3.0,
            VsysThreshold::V31 => 3.1,
            VsysThreshold::V32 => 3.2,
            VsysThreshold::V33 => 3.3,
            VsysThreshold::V34 => 3.4,
            VsysThreshold::V35 => 3.5,
            _ => 2.8, // Default fallback for unused values
        }
    }
}
