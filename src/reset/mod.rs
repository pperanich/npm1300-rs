use crate::{
    common::Task,
    field_sets::{Chargererrreason, Chargererrsensor, Rstcause, Scratch0, Scratch1},
};

mod types;

// Re-export everything in types.rs
pub use types::*;

impl<I2c: embedded_hal_async::i2c::I2c, Delay: embedded_hal_async::delay::DelayNs>
    crate::NPM1300<I2c, Delay>
{
    /// Clear error log registers
    ///
    /// Clears the RSTCAUSE, CHARGERERRREASON, and CHARGERERRSENSOR registers.
    /// This is useful for clearing historical error information.
    pub async fn clear_error_log(&mut self) -> Result<(), crate::NPM1300Error<I2c::Error>> {
        self.device
            .errlog()
            .taskclrerrlog()
            .dispatch_async(|command| command.set_taskclrerrlog(Task::Trigger))
            .await
    }

    /// Configure boot monitor
    ///
    /// Enables or disables the boot monitor timer. The boot monitor ensures
    /// the system boots properly within a specified time.
    ///
    /// # Arguments
    ///
    /// * `enabled` - Whether to enable the boot monitor
    pub async fn configure_boot_monitor_reset(
        &mut self,
        enabled: BootMonitorEnable,
    ) -> Result<(), crate::NPM1300Error<I2c::Error>> {
        self.device
            .errlog()
            .scratch_0()
            .modify_async(|reg| reg.set_boottimeren(enabled))
            .await
    }

    /// Write to scratch register 0
    ///
    /// Writes data to the 7-bit scratch register 0. This register is only
    /// cleared by Power-On Reset (POR).
    ///
    /// # Arguments
    ///
    /// * `value` - 7-bit value to write (0-127)
    pub async fn write_scratch0(&mut self, value: u8) -> Result<(), crate::NPM1300Error<I2c::Error>> {
        self.device
            .errlog()
            .scratch_0()
            .modify_async(|reg| reg.set_scratch_0(value & 0x7F))
            .await
    }

    /// Read scratch register 0
    ///
    /// Reads the current value of scratch register 0.
    ///
    /// # Returns
    ///
    /// The current scratch register 0 contents
    pub async fn read_scratch0(&mut self) -> Result<Scratch0, crate::NPM1300Error<I2c::Error>> {
        self.device.errlog().scratch_0().read_async().await
    }

    /// Write to scratch register 1
    ///
    /// Writes data to the 8-bit scratch register 1. This register is only
    /// cleared by Power-On Reset (POR).
    ///
    /// # Arguments
    ///
    /// * `value` - 8-bit value to write
    pub async fn write_scratch1(&mut self, value: u8) -> Result<(), crate::NPM1300Error<I2c::Error>> {
        self.device
            .errlog()
            .scratch_1()
            .write_async(|reg| reg.set_scratch_1(value))
            .await
    }

    /// Read scratch register 1
    ///
    /// Reads the current value of scratch register 1.
    ///
    /// # Returns
    ///
    /// The current scratch register 1 contents
    pub async fn read_scratch1(&mut self) -> Result<Scratch1, crate::NPM1300Error<I2c::Error>> {
        self.device.errlog().scratch_1().read_async().await
    }

    /// Get reset cause
    ///
    /// Reads the reset cause register to determine what caused the last reset.
    /// This register is cleared with the clear_error_log() function.
    ///
    /// # Returns
    ///
    /// The reset cause register contents showing which reset sources were active
    pub async fn get_reset_cause(&mut self) -> Result<Rstcause, crate::NPM1300Error<I2c::Error>> {
        self.device.errlog().rstcause().read_async().await
    }

    /// Get charger error reason
    ///
    /// Reads the charger error reason register to determine what charger
    /// errors have occurred. This register is cleared with the clear_error_log() function.
    ///
    /// # Returns
    ///
    /// The charger error reason register contents
    pub async fn get_charger_error_reason(
        &mut self,
    ) -> Result<Chargererrreason, crate::NPM1300Error<I2c::Error>> {
        self.device.errlog().chargererrreason().read_async().await
    }

    /// Get charger error sensor values
    ///
    /// Reads the charger error sensor register to determine the sensor states
    /// when charger errors occurred. This register is cleared with the clear_error_log() function.
    ///
    /// # Returns
    ///
    /// The charger error sensor register contents
    pub async fn get_charger_error_sensor(
        &mut self,
    ) -> Result<Chargererrsensor, crate::NPM1300Error<I2c::Error>> {
        self.device.errlog().chargererrsensor().read_async().await
    }

    /// Get comprehensive reset status
    ///
    /// Returns a comprehensive status structure containing all reset-related information.
    ///
    /// # Returns
    ///
    /// A ResetStatus structure containing reset cause, charger errors, and sensor states
    pub async fn get_reset_status(
        &mut self,
    ) -> Result<ResetStatus, crate::NPM1300Error<I2c::Error>> {
        let reset_cause = self.get_reset_cause().await?;
        let charger_error_reason = self.get_charger_error_reason().await?;
        let charger_error_sensor = self.get_charger_error_sensor().await?;

        Ok(ResetStatus {
            ship_mode_exit: reset_cause.shipmodeexit() == ResetCause::Reset,
            boot_monitor_timeout: reset_cause.bootmonitortimeout() == ResetCause::Reset,
            watchdog_timeout: reset_cause.watchdogtimeout() == ResetCause::Reset,
            long_press_timeout: reset_cause.longpresstimeout() == ResetCause::Reset,
            thermal_shutdown: reset_cause.thermalshutdown() == ResetCause::Reset,
            vsys_low: reset_cause.vsyslow() == ResetCause::Reset,
            software_reset: reset_cause.swreset() == ResetCause::Reset,
            charger_errors: ChargerErrorInfo {
                ntc_sensor_error: charger_error_reason.ntcsensorerr() == 1,
                vbat_sensor_error: charger_error_reason.vbatsensorerr() == 1,
                vbat_low_error: charger_error_reason.vbatlow() == 1,
                vtrickle_error: charger_error_reason.vtrickle() == 1,
                measurement_timeout_error: charger_error_reason.meastimeout() == 1,
                charge_timeout_error: charger_error_reason.chargetimeout() == 1,
                trickle_timeout_error: charger_error_reason.trickletimeout() == 1,
            },
            charger_sensor_states: ChargerSensorStates {
                sensor_ntc_cold: charger_error_sensor.sensorntccold() == 1,
                sensor_ntc_cool: charger_error_sensor.sensorntccool() == 1,
                sensor_ntc_warm: charger_error_sensor.sensorntcwarm() == 1,
                sensor_ntc_hot: charger_error_sensor.sensorntchot() == 1,
                sensor_vterm: charger_error_sensor.sensorvterm() == 1,
                sensor_recharge: charger_error_sensor.sensorrecharge() == 1,
                sensor_vtrickle: charger_error_sensor.sensorvtrickle() == 1,
                sensor_vbat_low: charger_error_sensor.sensorvbatlow() == 1,
            },
        })
    }
}