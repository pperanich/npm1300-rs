# Rust nPM1300 PMIC Driver

A `no_std`, async Rust driver for the [Nordic nPM1300](https://www.nordicsemi.com/Products/nPM1300) Power Management IC (PMIC).  
This crate provides both low-level register access and a high-level API for managing PMIC functions.

## Features

- `no_std` support for embedded environments
- Async design with comprehensive error handling
- Type-safe register access with validation
- [`defmt`](https://github.com/knurling-rs/defmt) support for logging (optional)
- Generated low-level API using [`device-driver`](https://docs.rs/device-driver/)
- Minimal dependencies

### Power Management Features

- **BUCK Converters**: Dual buck regulators with programmable voltage (1.0V-3.3V)
- **Battery Charging**: Complete Li-ion charging with JEITA temperature profile
- **Load Switches/LDOs**: Dual-mode operation (100mA load switch or 50mA LDO)
- **System Monitoring**: ADC measurements (VBAT, VSYS, VBUS, NTC, die temperature)
- **Power Sequencing**: Ship/hibernate modes with configurable wake-up sources
- **Safety Features**: Thermal protection, current limiting, power-fail detection

### System Control Features

- **Timer/Watchdog**: Boot monitor, watchdog, general purpose, and wake-up timers
- **GPIO Control**: 5 configurable GPIO pins with interrupt support
- **LED Drivers**: 3 LED drivers with programmable current (1mA-5mA)
- **Reset Management**: Software reset, reset cause detection, error logging
- **Status Monitoring**: Comprehensive system status and error reporting

## API Coverage

> [!IMPORTANT]
> The driver is currently under development, and the API may change before reaching 1.0.

| **Component**                             | **Low-Level API** | **High-Level API** |
| ----------------------------------------- | :---------------: | :----------------: |
| SYSREG — System regulator                 |        ✅         |         ✅         |
| CHARGER — Battery charger                 |        ✅         |         ✅         |
| BUCK — Buck regulators                    |        ✅         |         ✅         |
| LDSW/LDO — Load switches/LDO regulators   |        ✅         |         ✅         |
| LEDDRV — LED drivers                      |        ✅         |         ✅         |
| GPIO — General-purpose I/O                |        ✅         |         ✅         |
| ADC - System Monitor                      |        ✅         |         ✅         |
| POF - Power-fail comparator               |        ✅         |         ✅         |
| TIMER — Timer/monitor                     |        ✅         |         ✅         |
| Ship and hibernate modes                  |        ✅         |         ✅         |
| Reset and error                           |        ✅         |         ✅         |
| Event and interrupt                       |        ❌         |         ❌         |
| Fuel gauge                                |        ❌         |         ❌         |

Legend:

- ✅ Fully implemented and production-ready
- ❌ Not implemented (may not be available in nPM1300 hardware)

**Note:** LDSW/LDO represents a single hardware block where each load switch (LDSW1/LDSW2) can operate in either Load Switch mode (100mA) or LDO mode (50mA) with programmable output voltage (1.0V-3.3V). Both modes are fully supported with GPIO control, soft start, and active discharge features.

> [!NOTE]
> The driver is now production-ready for most use cases with comprehensive error handling, validation, and safety features. All major power management functions are fully implemented and tested.

> [!NOTE]
> This crate is async-only and there are no plans to add synchronous APIs. Contributions are welcome!

## Usage Example

Here's a minimal example using the Embassy framework on an nRF52840:

```rust
#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_nrf::{
    bind_interrupts,
    peripherals,
    twim::{self, Twim},
};
use {defmt_rtt as _, panic_probe as _};

use npm1300_rs::{
    types::{BuckVoltage, LdoVoltage, LdswMode},
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

    let mut npm1300 = NPM1300::new(twi);
    
    // Configure BUCK converter
    let _ = npm1300.set_buck2_normal_voltage(BuckVoltage::V1_8).await;
    let _ = npm1300.enable_buck2().await;
    
    // Configure LDSW1 as LDO regulator
    let _ = npm1300.set_ldsw1_mode(LdswMode::Ldo).await;
    let _ = npm1300.set_ldsw1_ldo_voltage(LdoVoltage::V3_3).await;
    let _ = npm1300.enable_ldsw1().await;
    
    // Read system status
    let vbat = npm1300.measure_vbat().await.unwrap_or(0.0);
    defmt::info!("Battery voltage: {:.2}V", vbat);
}
```

More examples can be found in the [`examples`](examples) directory.

## Support

- [GitHub Issues](https://github.com/thermigo/npm1300-rs/issues) - Bug reports, feature requests, and questions
- [Examples](examples/) - Comprehensive examples for all major features
- [API Documentation](https://docs.rs/npm1300-rs) - Complete API reference

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT License ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

## Contributing

We welcome contributions! Please:

1. Fork the repository
2. Create a feature branch
3. Follow [conventional commits](https://www.conventionalcommits.org) for commit messages
4. Submit a Pull Request

By contributing, you agree that your work will be dual-licensed as above, without additional terms or conditions.
