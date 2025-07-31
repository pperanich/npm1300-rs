#![no_std]
#![no_main]

//! Example demonstrating control of the NPM1300 PMIC's TIMER block on nRF9160
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
    reset::BootMonitorEnable,
    timer::{TimerMode, TimerPrescaler},
    NPM1300,
};

bind_interrupts!(struct Irqs {
    SERIAL0 => twim::InterruptHandler<peripherals::SERIAL0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    
    let sdapin = p.P0_28;
    let sclpin = p.P0_29;
    let mut config = twim::Config::default();

    // Modify the i2c configuration fields if you dont have external i2c pullups
    config.sda_pullup = true;
    config.scl_pullup = true;

    let twi = Twim::new(p.SERIAL0, Irqs, sdapin, sclpin, config);

    let mut npm1300 = NPM1300::new(twi, embassy_time::Delay);
    
    defmt::info!("NPM1300 Timer Configuration Example (nRF9160)");

    // Example 1: Configure watchdog timer for system monitoring
    defmt::info!("Configuring watchdog timer for 60 seconds...");
    let _ = npm1300.configure_watchdog(TimerMode::WatchdogReset, 60000, TimerPrescaler::Slow).await;
    
    // Simulate application work and periodic watchdog kicks
    for i in 0..5 {
        defmt::info!("Application work iteration {}, kicking watchdog...", i + 1);
        let _ = npm1300.kick_watchdog().await;
        Timer::after_millis(10000).await; // Wait 10 seconds
    }

    // Example 2: Configure wake-up timer for power management
    defmt::info!("Configuring wake-up timer for hibernate mode (120 seconds)...");
    let _ = npm1300.configure_wakeup_timer(120000, TimerPrescaler::Slow).await;
    
    let timer_status = npm1300.get_timer_status().await;
    defmt::info!("Timer status: {:?}", timer_status);

    // Example 3: Use general purpose timer for application timing
    defmt::info!("Using general purpose timer for precise timing...");
    let _ = npm1300.configure_general_purpose_timer(3000, TimerPrescaler::Fast).await;
    
    Timer::after_millis(4000).await;

    // Example 4: Boot monitor configuration
    defmt::info!("Configuring boot monitor...");
    let _ = npm1300.configure_boot_monitor_reset(BootMonitorEnable::Enabled).await;
    
    let is_boot_active = npm1300.is_boot_monitor_active().await;
    defmt::info!("Boot monitor active: {:?}", is_boot_active);

    defmt::info!("Timer example completed!");

    loop {
        Timer::after_millis(1000).await;
    }
}