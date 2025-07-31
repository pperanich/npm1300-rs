#![no_std]
#![no_main]

//! Example demonstrating the NPM1300 PMIC's LDSW/LDO features

use embassy_executor::Spawner;
use embassy_nrf::{
    peripherals,
    bind_interrupts,
    twim::{self, Twim},
};

use {defmt_rtt as _, panic_probe as _};

use npm1300::{
    NPM1300,
    ldsw::{LdoVoltage},
    gpios::{Gpio, GpioPolarity},
    Ldsw1Ldosel, Ldsw2Ldosel,
    Ldsw1Softstartdisable, Ldsw1Softstartsel,
    Ldsw1Activedischarge,
};

bind_interrupts!(struct Irqs {
    SPIM0_SPIS0_TWIM0_TWIS0_SPI0_TWI0 => twim::InterruptHandler<peripherals::TWISPI0>;
});

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_nrf::init(Default::default());
    
    let sdapin = p.P0_28;
    let sclpin = p.P0_29;
    let mut config = twim::Config::default();

    // Modify the configuration fields
    config.sda_pullup = true;
    config.scl_pullup = true;

    defmt::info!("Configuring TWIM...");
    let twi = Twim::new(p.TWISPI0, Irqs, sdapin, sclpin, config);
    
    let mut npm1300 = NPM1300::new(twi, embassy_time::Delay);
    
    defmt::info!("Configuring LDSW1 as LDO with 3.3V output...");
    // Configure LDSW1 as LDO mode
    let _ = npm1300.set_ldsw1_mode(Ldsw1Ldosel::Ldo).await;
    // Set LDO1 output voltage to 3.3V
    let _ = npm1300.set_ldsw1_ldo_voltage(LdoVoltage::V3_3).await;
    // Configure soft start
    let _ = npm1300.configure_ldsw1_soft_start(
        Ldsw1Softstartdisable::Noeffect,
        Ldsw1Softstartsel::Ma35
    ).await;
    // Enable active discharge
    let _ = npm1300.set_ldsw1_active_discharge(Ldsw1Activedischarge::Active).await;
    // Enable LDSW1
    let _ = npm1300.enable_ldsw1().await;
    
    defmt::info!("Configuring LDSW2 as Load Switch...");
    // Configure LDSW2 as Load Switch mode
    let _ = npm1300.set_ldsw2_mode(Ldsw2Ldosel::Ldsw).await;
    // Configure GPIO control for LDSW2 (using GPIO2)
    let _ = npm1300.set_ldsw2_gpio_control(Gpio::Gpio2, GpioPolarity::NotInverted).await;
    // Enable LDSW2
    let _ = npm1300.enable_ldsw2().await;
    
    defmt::info!("Checking LDSW status...");
    let status = npm1300.get_ldsw_status().await.unwrap();
    defmt::info!("LDSW status: {:?}", status);
    
    defmt::info!("LDSW configuration complete!");
}