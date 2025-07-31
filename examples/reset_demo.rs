//! nPM1300 Reset functionality demonstration
//!
//! This example shows how to use the reset-related functionality of the nPM1300,
//! including reading reset causes, managing error logs, and working with scratch registers.

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use npm1300::{reset::BootMonitorEnable, NPM1300};
use panic_halt as _;

// Mock I2C and Delay for demonstration
struct MockI2c;
struct MockDelay;

impl embedded_hal_async::i2c::I2c for MockI2c {
    async fn read(&mut self, _address: u8, _buffer: &mut [u8]) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn write(&mut self, _address: u8, _bytes: &[u8]) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn write_read(
        &mut self,
        _address: u8,
        _bytes: &[u8],
        _buffer: &mut [u8],
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl embedded_hal_async::i2c::ErrorType for MockI2c {
    type Error = ();
}

impl embedded_hal_async::delay::DelayNs for MockDelay {
    async fn delay_ns(&mut self, _ns: u32) {}
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let i2c = MockI2c;
    let delay = MockDelay;
    let mut npm1300 = NPM1300::new(i2c, delay);

    // Example 1: Check reset cause
    match npm1300.get_reset_cause().await {
        Ok(reset_cause) => {
            // Process reset cause information
            // In a real application, you would check specific reset causes
        }
        Err(_) => {
            // Handle error
        }
    }

    // Example 2: Get comprehensive reset status
    match npm1300.get_reset_status().await {
        Ok(status) => {
            if status.has_any_reset_cause() {
                // Handle reset causes
                if let Some(primary_cause) = status.get_primary_reset_cause() {
                    // Log or handle the primary reset cause
                    // In a real application: log::info!("Reset cause: {}", primary_cause);
                }
            }

            if status.has_any_charger_error() {
                // Handle charger errors
                if let Some(primary_error) = status.get_primary_charger_error() {
                    // Log or handle the primary charger error
                    // In a real application: log::warn!("Charger error: {}", primary_error);
                }
            }
        }
        Err(_) => {
            // Handle error
        }
    }

    // Example 3: Configure boot monitor
    if let Err(_) = npm1300
        .configure_boot_monitor_reset(BootMonitorEnable::Enabled)
        .await
    {
        // Handle configuration error
    }

    // Example 4: Use scratch registers for persistent data
    // Write some application state to scratch register 0
    let app_state = 0x42; // Example application state
    if let Err(_) = npm1300.write_scratch0(app_state).await {
        // Handle write error
    }

    // Read back the application state
    match npm1300.read_scratch0().await {
        Ok(scratch0) => {
            // Process the scratch register data
            // let stored_state = scratch0.scratch_0();
        }
        Err(_) => {
            // Handle read error
        }
    }

    // Example 5: Clear error logs after handling
    if let Err(_) = npm1300.clear_error_log().await {
        // Handle clear error
    }

    // Example 6: Check for charger errors specifically
    match npm1300.get_charger_error_reason().await {
        Ok(error_reason) => {
            // Check specific error flags
            // if error_reason.ntcsensorerr() == 1 { /* Handle NTC error */ }
            // if error_reason.vbatsensorerr() == 1 { /* Handle VBAT error */ }
        }
        Err(_) => {
            // Handle error
        }
    }

    // Example 7: Check charger sensor states during errors
    match npm1300.get_charger_error_sensor().await {
        Ok(sensor_states) => {
            // Check sensor states when errors occurred
            // if sensor_states.sensorntccold() == 1 { /* NTC was in cold region */ }
            // if sensor_states.sensorvbatlow() == 1 { /* VBAT was low */ }
        }
        Err(_) => {
            // Handle error
        }
    }

    // Main application loop
    loop {
        Timer::after(Duration::from_secs(1)).await;

        // Periodically check for new reset causes or errors
        if let Ok(status) = npm1300.get_reset_status().await {
            if status.has_any_reset_cause() || status.has_any_charger_error() {
                // Handle new issues
                // Clear error log after handling
                let _ = npm1300.clear_error_log().await;
            }
        }
    }
}