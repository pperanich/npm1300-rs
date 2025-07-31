

/// Boot monitor enable configuration
///
/// Controls whether the boot monitor timer is enabled. The boot monitor
/// ensures the system boots properly within a specified time.
#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
pub enum BootMonitorEnable {
    /// Boot monitor is disabled
    Disabled = 0,
    /// Boot monitor is enabled
    Enabled = 1,
}



impl From<BootMonitorEnable> for u8 {
    fn from(enable: BootMonitorEnable) -> Self {
        enable as u8
    }
}

impl From<u8> for BootMonitorEnable {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Disabled,
            _ => Self::Enabled,
        }
    }
}

/// Reset cause enumeration
///
/// Represents the different types of reset causes that can occur.
#[derive(Copy, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
pub enum ResetCause {
    /// No reset occurred
    NoReset = 0,
    /// Reset was activated
    Reset = 1,
}



impl From<ResetCause> for u8 {
    fn from(cause: ResetCause) -> Self {
        cause as u8
    }
}

impl From<u8> for ResetCause {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::NoReset,
            _ => Self::Reset,
        }
    }
}

/// Comprehensive reset status information
///
/// Contains all reset-related status information including reset causes,
/// charger errors, and sensor states at the time of errors.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
pub struct ResetStatus {
    /// Reset was caused by Ship mode exit
    pub ship_mode_exit: bool,
    /// Reset was caused by boot monitor timeout
    pub boot_monitor_timeout: bool,
    /// Reset was caused by watchdog timeout
    pub watchdog_timeout: bool,
    /// Reset was caused by long press timeout
    pub long_press_timeout: bool,
    /// Reset was caused by thermal shutdown
    pub thermal_shutdown: bool,
    /// Reset was caused by VSYS voltage too low (power failure)
    pub vsys_low: bool,
    /// Reset was caused by software reset command
    pub software_reset: bool,
    /// Charger error information
    pub charger_errors: ChargerErrorInfo,
    /// Charger sensor states when errors occurred
    pub charger_sensor_states: ChargerSensorStates,
}

/// Charger error information
///
/// Contains information about various charger error conditions.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
pub struct ChargerErrorInfo {
    /// NTC thermistor sensor error occurred
    pub ntc_sensor_error: bool,
    /// VBAT sensor error occurred
    pub vbat_sensor_error: bool,
    /// VBAT voltage too low error occurred
    pub vbat_low_error: bool,
    /// Vtrickle error occurred
    pub vtrickle_error: bool,
    /// Measurement timeout error occurred
    pub measurement_timeout_error: bool,
    /// Charge timeout error occurred
    pub charge_timeout_error: bool,
    /// Trickle timeout error occurred
    pub trickle_timeout_error: bool,
}

/// Charger sensor states during error conditions
///
/// Contains the state of various sensors when charger errors occurred.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
pub struct ChargerSensorStates {
    /// NTC cold region was active when error occurred
    pub sensor_ntc_cold: bool,
    /// NTC cool region was active when error occurred
    pub sensor_ntc_cool: bool,
    /// NTC warm region was active when error occurred
    pub sensor_ntc_warm: bool,
    /// NTC hot region was active when error occurred
    pub sensor_ntc_hot: bool,
    /// Vterm status when error occurred
    pub sensor_vterm: bool,
    /// Recharge status when error occurred
    pub sensor_recharge: bool,
    /// Vtrickle status when error occurred
    pub sensor_vtrickle: bool,
    /// VBAT low status when error occurred
    pub sensor_vbat_low: bool,
}

impl ResetStatus {
    /// Check if any reset cause is active
    ///
    /// Returns true if any reset cause flag is set.
    pub fn has_any_reset_cause(&self) -> bool {
        self.ship_mode_exit
            || self.boot_monitor_timeout
            || self.watchdog_timeout
            || self.long_press_timeout
            || self.thermal_shutdown
            || self.vsys_low
            || self.software_reset
    }

    /// Check if any charger error occurred
    ///
    /// Returns true if any charger error flag is set.
    pub fn has_any_charger_error(&self) -> bool {
        self.charger_errors.ntc_sensor_error
            || self.charger_errors.vbat_sensor_error
            || self.charger_errors.vbat_low_error
            || self.charger_errors.vtrickle_error
            || self.charger_errors.measurement_timeout_error
            || self.charger_errors.charge_timeout_error
            || self.charger_errors.trickle_timeout_error
    }

    /// Get the most significant reset cause
    ///
    /// Returns the most significant reset cause that occurred, prioritizing
    /// more critical causes over less critical ones.
    pub fn get_primary_reset_cause(&self) -> Option<&'static str> {
        if self.thermal_shutdown {
            Some("Thermal shutdown")
        } else if self.vsys_low {
            Some("VSYS voltage low")
        } else if self.watchdog_timeout {
            Some("Watchdog timeout")
        } else if self.boot_monitor_timeout {
            Some("Boot monitor timeout")
        } else if self.long_press_timeout {
            Some("Long press timeout")
        } else if self.software_reset {
            Some("Software reset")
        } else if self.ship_mode_exit {
            Some("Ship mode exit")
        } else {
            None
        }
    }

    /// Get the most significant charger error
    ///
    /// Returns the most significant charger error that occurred.
    pub fn get_primary_charger_error(&self) -> Option<&'static str> {
        if self.charger_errors.ntc_sensor_error {
            Some("NTC sensor error")
        } else if self.charger_errors.vbat_sensor_error {
            Some("VBAT sensor error")
        } else if self.charger_errors.vbat_low_error {
            Some("VBAT low error")
        } else if self.charger_errors.vtrickle_error {
            Some("Vtrickle error")
        } else if self.charger_errors.measurement_timeout_error {
            Some("Measurement timeout error")
        } else if self.charger_errors.charge_timeout_error {
            Some("Charge timeout error")
        } else if self.charger_errors.trickle_timeout_error {
            Some("Trickle timeout error")
        } else {
            None
        }
    }
}