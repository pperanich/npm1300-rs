#![no_std]
#![no_main]

//! Example demonstrating software reset functionality of the NPM1300 PMIC
//! This example shows how to:
//! - Check reset cause to determine what caused the last reset
//! - Trigger a software reset
//! - Clear error logs

use embassy_executor::Spawner;
use embassy_nrf::{
    bind_interrupts,
    peripherals,
    twim::{self, Twim},
};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

use npm1300::NPM1300;

bind_interrupts!(struct Irqs {
    SPIM0_SPIS0_TWIM0_TWIS0_SPI0_TWI0 => twim::InterruptHandler<peripherals::TWISPI0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    let config = twim::Config::default();

    let twi = Twim::new(p.TWISPI0, Irqs, p.P0_07, p.P0_12, config);

    let mut npm1300 = NPM1300::new(twi, embassy_time::Delay);
    
    defmt::info!("NPM1300 Software Reset Example");

    // Check what caused the last reset
    defmt::info!("Checking reset cause...");
    match npm1300.get_reset_status().await {
        Ok(status) => {
            defmt::info!("Reset Status:");
            defmt::info!("  Ship mode exit: {}", status.ship_mode_exit);
            defmt::info!("  Boot monitor timeout: {}", status.boot_monitor_timeout);
            defmt::info!("  Watchdog timeout: {}", status.watchdog_timeout);
            defmt::info!("  Long press timeout: {}", status.long_press_timeout);
            defmt::info!("  Thermal shutdown: {}", status.thermal_shutdown);
            defmt::info!("  VSYS low: {}", status.vsys_low);
            defmt::info!("  Software reset: {}", status.software_reset);
            
            if status.software_reset {
                defmt::info!("Last reset was caused by software reset!");
            }
        }
        Err(e) => defmt::error!("Failed to get reset status: {:?}", e),
    }

    // Wait a bit
    Timer::after_millis(2000).await;

    // Clear error logs
    defmt::info!("Clearing error logs...");
    match npm1300.clear_error_log().await {
        Ok(_) => defmt::info!("Error logs cleared successfully"),
        Err(e) => defmt::error!("Failed to clear error logs: {:?}", e),
    }

    Timer::after_millis(1000).await;

    // Demonstrate writing to scratch registers (these survive reset)
    defmt::info!("Writing to scratch registers...");
    let _ = npm1300.write_scratch0(0x42).await;
    let _ = npm1300.write_scratch1(0xAB).await;

    // Read back scratch registers
    if let Ok(scratch0) = npm1300.read_scratch0().await {
        defmt::info!("Scratch0 value: 0x{:02x}", scratch0.scratch_0());
    }
    if let Ok(scratch1) = npm1300.read_scratch1().await {
        defmt::info!("Scratch1 value: 0x{:02x}", scratch1.scratch_1());
    }

    Timer::after_millis(2000).await;

    // Trigger software reset after a countdown
    defmt::warn!("Software reset will be triggered in 5 seconds...");
    Timer::after_millis(1000).await;
    defmt::warn!("4...");
    Timer::after_millis(1000).await;
    defmt::warn!("3...");
    Timer::after_millis(1000).await;
    defmt::warn!("2...");
    Timer::after_millis(1000).await;
    defmt::warn!("1...");
    Timer::after_millis(1000).await;
    
    defmt::warn!("Triggering software reset NOW!");
    
    // This will cause the device to reset and restart
    match npm1300.software_reset().await {
        Ok(_) => {
            // This should not be reached as the device will reset
            defmt::info!("Software reset command sent successfully");
        }
        Err(e) => defmt::error!("Failed to trigger software reset: {:?}", e),
    }

    // This code should not be reached after reset
    loop {
        defmt::info!("Still running after reset attempt...");
        Timer::after_millis(1000).await;
    }
}