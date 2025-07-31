/// Timer mode selection for the nPM1300 TIMER block
#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
pub enum TimerMode {
    /// Boot monitor mode
    BootMonitor = 0,
    /// Watchdog warning mode
    WatchdogWarning = 1,
    /// Watchdog reset mode
    WatchdogReset = 2,
    /// General purpose timer mode
    GeneralPurposeTimer = 3,
    /// Wake-up timer mode
    WakeupTimer = 4,
}

impl From<TimerMode> for u8 {
    fn from(mode: TimerMode) -> Self {
        mode as u8
    }
}

impl From<u8> for TimerMode {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::BootMonitor,
            1 => Self::WatchdogWarning,
            2 => Self::WatchdogReset,
            3 => Self::GeneralPurposeTimer,
            4 => Self::WakeupTimer,
            _ => Self::BootMonitor, // Default fallback
        }
    }
}

/// Timer prescaler selection for the nPM1300 TIMER block
#[derive(Copy, Clone, Debug)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
pub enum TimerPrescaler {
    /// 16 ms prescaler (slow)
    Slow = 0,
    /// 2 ms prescaler (fast)
    Fast = 1,
}

impl From<TimerPrescaler> for u8 {
    fn from(prescaler: TimerPrescaler) -> Self {
        prescaler as u8
    }
}

impl From<u8> for TimerPrescaler {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::Slow,
            1 => Self::Fast,
            _ => Self::Slow, // Default fallback
        }
    }
}

/// Timer configuration structure for setting up timer values
#[derive(Copy, Clone, Debug, Default)]
#[cfg_attr(feature = "defmt-03", derive(defmt::Format))]
pub struct TimerValue {
    /// High byte (bits 23-16)
    pub hi_byte: u8,
    /// Middle byte (bits 15-8)
    pub mid_byte: u8,
    /// Low byte (bits 7-0)
    pub low_byte: u8,
}

impl TimerValue {
    /// Create a new timer value from a 24-bit value
    pub fn from_u32(value: u32) -> Self {
        Self {
            hi_byte: ((value >> 16) & 0xFF) as u8,
            mid_byte: ((value >> 8) & 0xFF) as u8,
            low_byte: (value & 0xFF) as u8,
        }
    }

    /// Convert timer value to a 24-bit u32
    pub fn to_u32(&self) -> u32 {
        ((self.hi_byte as u32) << 16) | ((self.mid_byte as u32) << 8) | (self.low_byte as u32)
    }

    /// Create a timer value for a specific time duration in milliseconds
    ///
    /// # Arguments
    /// * `duration_ms` - Duration in milliseconds
    /// * `prescaler` - Timer prescaler setting
    ///
    /// # Returns
    /// Timer value that represents the given duration
    pub fn from_duration_ms(duration_ms: u32, prescaler: TimerPrescaler) -> Self {
        let prescaler_ms = match prescaler {
            TimerPrescaler::Slow => 16, // 16 ms prescaler
            TimerPrescaler::Fast => 2,  // 2 ms prescaler
        };

        let timer_ticks = duration_ms / prescaler_ms;
        Self::from_u32(timer_ticks.min(0xFFFFFF)) // Limit to 24-bit max
    }

    /// Get the duration in milliseconds for this timer value
    ///
    /// # Arguments
    /// * `prescaler` - Timer prescaler setting
    ///
    /// # Returns
    /// Duration in milliseconds
    pub fn to_duration_ms(&self, prescaler: TimerPrescaler) -> u32 {
        let prescaler_ms = match prescaler {
            TimerPrescaler::Slow => 16, // 16 ms prescaler
            TimerPrescaler::Fast => 2,  // 2 ms prescaler
        };

        self.to_u32() * prescaler_ms
    }
}
