#![no_std]
#![no_main]

//! Example demonstrating control of the NPM1300 PMIC's TIMER block
//! This example shows how to configure and use the various timer modes:
//! - Boot monitor
//! - Watchdog timer
//! - General purpose timer
//! - Wake-up timer

use embassy_executor::Spawner;
use embassy_nrf::{
    bind_interrupts,
    peripherals,
    twim::{self, Twim},
};
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

use npm1300::{
    timer::{TimerMode, TimerPrescaler, TimerValue},
    NPM1300,
};

bind_interrupts!(struct Irqs {
    SPIM0_SPIS0_TWIM0_TWIS0_SPI0_TWI0 => twim::InterruptHandler<peripherals::TWISPI0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    let config = twim::Config::default();

    let twi = Twim::new(p.TWISPI0, Irqs, p.P0_07, p.P0_12, config);

    let mut npm1300 = NPM1300::new(twi, embassy_time::Delay);
    
    defmt::info!("NPM1300 Timer Configuration Example");

    // Example 1: Configure boot monitor
    defmt::info!("Configuring boot monitor with 10 second timeout...");
    let _ = npm1300.configure_boot_monitor(true, Some(10000)).await;
    
    let timer_status = npm1300.get_timer_status().await;
    defmt::info!("Timer status after boot monitor config: {:?}", timer_status);
    
    let is_boot_active = npm1300.is_boot_monitor_active().await;
    defmt::info!("Boot monitor active: {:?}", is_boot_active);

    Timer::after_millis(2000).await;

    // Example 2: Configure general purpose timer
    defmt::info!("Configuring general purpose timer for 5 seconds...");
    let _ = npm1300.configure_general_purpose_timer(5000, TimerPrescaler::Slow).await;
    
    let timer_status = npm1300.get_timer_status().await;
    defmt::info!("Timer status after GP timer config: {:?}", timer_status);

    Timer::after_millis(2000).await;

    // Example 3: Configure watchdog timer
    defmt::info!("Configuring watchdog timer for 30 seconds with warning mode...");
    let _ = npm1300.configure_watchdog(TimerMode::WatchdogWarning, 30000, TimerPrescaler::Slow).await;
    
    Timer::after_millis(2000).await;

    // Example 4: Kick the watchdog
    defmt::info!("Kicking watchdog timer...");
    let _ = npm1300.kick_watchdog().await;
    
    Timer::after_millis(2000).await;

    // Example 5: Configure wake-up timer (for hibernate mode)
    defmt::info!("Configuring wake-up timer for 60 seconds...");
    let _ = npm1300.configure_wakeup_timer(60000, TimerPrescaler::Slow).await;
    
    Timer::after_millis(2000).await;

    // Example 6: Manual timer configuration
    defmt::info!("Manual timer configuration example...");
    
    // Configure timer mode and prescaler separately
    let _ = npm1300.configure_timer(TimerMode::GeneralPurposeTimer, TimerPrescaler::Fast).await;
    
    // Create a custom timer value (2 seconds with fast prescaler)
    let timer_value = TimerValue::from_duration_ms(2000, TimerPrescaler::Fast);
    defmt::info!("Timer value: hi={}, mid={}, low={}", 
                 timer_value.hi_byte, timer_value.mid_byte, timer_value.low_byte);
    
    // Set the timer target
    let _ = npm1300.set_timer_target(timer_value).await;
    
    // Start the timer
    let _ = npm1300.start_timer().await;
    
    defmt::info!("Timer started, waiting for completion...");
    Timer::after_millis(3000).await;

    // Example 7: Read current timer value
    defmt::info!("Reading current timer value...");
    let current_value = npm1300.read_timer_value().await;
    match current_value {
        Ok(value) => {
            defmt::info!("Current timer value: hi={}, mid={}, low={}", 
                         value.hi_byte, value.mid_byte, value.low_byte);
            let duration = value.to_duration_ms(TimerPrescaler::Fast);
            defmt::info!("Duration in ms: {}", duration);
        }
        Err(e) => defmt::error!("Failed to read timer value: {:?}", e),
    }

    // Example 8: Check timer configuration status
    let is_configured = npm1300.is_timer_configured().await;
    defmt::info!("Timer configured: {:?}", is_configured);

    // Stop the timer
    defmt::info!("Stopping timer...");
    let _ = npm1300.stop_timer().await;

    defmt::info!("Timer example completed!");

    loop {
        Timer::after_millis(1000).await;
    }
}