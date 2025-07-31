use crate::{common::Task, field_sets::Timerstatus};

mod types;

// Re-export everything in types.rs
pub use types::*;

impl<I2c: embedded_hal_async::i2c::I2c, Delay: embedded_hal_async::delay::DelayNs>
    crate::NPM1300<I2c, Delay>
{
    /// Start the timer
    pub async fn start_timer(&mut self) -> Result<(), crate::NPM1300Error<I2c::Error>> {
        self.device
            .timer()
            .timerset()
            .dispatch_async(|command| command.set_tasktimeren(Task::Trigger))
            .await
    }

    /// Stop the timer
    pub async fn stop_timer(&mut self) -> Result<(), crate::NPM1300Error<I2c::Error>> {
        self.device
            .timer()
            .timerclr()
            .dispatch_async(|command| command.set_tasktimerdis(Task::Trigger))
            .await
    }

    /// Configure timer mode and prescaler
    ///
    /// # Arguments
    /// * `mode` - Timer mode (boot monitor, watchdog, general purpose, etc.)
    /// * `prescaler` - Timer prescaler (16ms or 2ms)
    pub async fn configure_timer(
        &mut self,
        mode: TimerMode,
        prescaler: TimerPrescaler,
    ) -> Result<(), crate::NPM1300Error<I2c::Error>> {
        self.device
            .timer()
            .timerconfig()
            .write_async(|reg| {
                reg.set_timermodesel(mode);
                reg.set_timerprescaler(prescaler);
            })
            .await
    }

    /// Set timer target value
    ///
    /// # Arguments
    /// * `timer_value` - 24-bit timer value to set
    pub async fn set_timer_target(
        &mut self,
        timer_value: TimerValue,
    ) -> Result<(), crate::NPM1300Error<I2c::Error>> {
        // Set the timer bytes
        self.device
            .timer()
            .timerhibyte()
            .write_async(|reg| reg.set_timerhibyte(timer_value.hi_byte))
            .await?;

        self.device
            .timer()
            .timermidbyte()
            .write_async(|reg| reg.set_timermidbyte(timer_value.mid_byte))
            .await?;

        self.device
            .timer()
            .timerlobyte()
            .write_async(|reg| reg.set_timerlobyte(timer_value.low_byte))
            .await?;

        // Strobe to apply the timer target
        self.device
            .timer()
            .timertargetstrobe()
            .dispatch_async(|command| command.set_tasktimertargetstrobe(Task::Trigger))
            .await
    }

    /// Kick the watchdog timer to prevent timeout
    pub async fn kick_watchdog(&mut self) -> Result<(), crate::NPM1300Error<I2c::Error>> {
        self.device
            .timer()
            .watchdogkick()
            .dispatch_async(|command| command.set_taskwatchdogkick(Task::Trigger))
            .await
    }

    /// Get timer status
    pub async fn get_timer_status(
        &mut self,
    ) -> Result<Timerstatus, crate::NPM1300Error<I2c::Error>> {
        self.device.timer().timerstatus().read_async().await
    }

    /// Configure boot monitor
    ///
    /// # Arguments
    /// * `enable` - true to enable boot monitor, false to disable
    /// * `timeout_ms` - Boot monitor timeout in milliseconds (default is 10 seconds)
    pub async fn configure_boot_monitor(
        &mut self,
        enable: bool,
        timeout_ms: Option<u32>,
    ) -> Result<(), crate::NPM1300Error<I2c::Error>> {
        if enable {
            let timeout = timeout_ms.unwrap_or(10000); // Default 10 seconds
            let timer_value = TimerValue::from_duration_ms(timeout, TimerPrescaler::Slow);

            // Configure as boot monitor with slow prescaler
            self.configure_timer(TimerMode::BootMonitor, TimerPrescaler::Slow)
                .await?;

            // Set timer target
            self.set_timer_target(timer_value).await?;

            // Start the timer
            self.start_timer().await
        } else {
            // Stop the timer to disable boot monitor
            self.stop_timer().await
        }
    }

    /// Configure watchdog timer
    ///
    /// # Arguments
    /// * `mode` - Watchdog mode (warning or reset)
    /// * `timeout_ms` - Watchdog timeout in milliseconds
    /// * `prescaler` - Timer prescaler setting
    pub async fn configure_watchdog(
        &mut self,
        mode: TimerMode,
        timeout_ms: u32,
        prescaler: TimerPrescaler,
    ) -> Result<(), crate::NPM1300Error<I2c::Error>> {
        let timer_value = TimerValue::from_duration_ms(timeout_ms, prescaler);

        // Configure watchdog mode and prescaler
        self.configure_timer(mode, prescaler).await?;

        // Set timer target
        self.set_timer_target(timer_value).await?;

        // Start the timer
        self.start_timer().await
    }

    /// Configure wake-up timer for hibernate mode
    ///
    /// # Arguments
    /// * `timeout_ms` - Wake-up timeout in milliseconds
    /// * `prescaler` - Timer prescaler setting
    pub async fn configure_wakeup_timer(
        &mut self,
        timeout_ms: u32,
        prescaler: TimerPrescaler,
    ) -> Result<(), crate::NPM1300Error<I2c::Error>> {
        let timer_value = TimerValue::from_duration_ms(timeout_ms, prescaler);

        // Configure as wake-up timer
        self.configure_timer(TimerMode::WakeupTimer, prescaler)
            .await?;

        // Set timer target
        self.set_timer_target(timer_value).await
    }

    /// Configure general purpose timer
    ///
    /// # Arguments
    /// * `timeout_ms` - Timer timeout in milliseconds
    /// * `prescaler` - Timer prescaler setting
    pub async fn configure_general_purpose_timer(
        &mut self,
        timeout_ms: u32,
        prescaler: TimerPrescaler,
    ) -> Result<(), crate::NPM1300Error<I2c::Error>> {
        let timer_value = TimerValue::from_duration_ms(timeout_ms, prescaler);

        // Configure as general purpose timer
        self.configure_timer(TimerMode::GeneralPurposeTimer, prescaler)
            .await?;

        // Set timer target
        self.set_timer_target(timer_value).await?;

        // Start the timer
        self.start_timer().await
    }

    /// Read current timer value from registers
    pub async fn read_timer_value(
        &mut self,
    ) -> Result<TimerValue, crate::NPM1300Error<I2c::Error>> {
        let hi_reg = self.device.timer().timerhibyte().read_async().await?;
        let mid_reg = self.device.timer().timermidbyte().read_async().await?;
        let low_reg = self.device.timer().timerlobyte().read_async().await?;

        Ok(TimerValue {
            hi_byte: hi_reg.timerhibyte(),
            mid_byte: mid_reg.timermidbyte(),
            low_byte: low_reg.timerlobyte(),
        })
    }

    /// Check if boot monitor is active
    pub async fn is_boot_monitor_active(
        &mut self,
    ) -> Result<bool, crate::NPM1300Error<I2c::Error>> {
        let status = self.get_timer_status().await?;
        Ok(status.bootmonitoractive() == crate::Bootmonitoractive::Active)
    }

    /// Check if timer is configured and ready
    pub async fn is_timer_configured(&mut self) -> Result<bool, crate::NPM1300Error<I2c::Error>> {
        let status = self.get_timer_status().await?;
        Ok(status.slowdomainconfigured() == crate::Slowdomainconfigured::Config)
    }
}
