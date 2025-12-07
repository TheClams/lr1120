# LR1120 Driver

[![Crates.io](https://img.shields.io/crates/v/lr1120.svg)](https://crates.io/crates/lr1120)
[![Documentation](https://docs.rs/lr1120/badge.svg)](https://docs.rs/lr1120)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/TheClams/lr1120)

An async, no_std Rust driver for the Semtech LR2021 dual-band transceiver, supporting many different radio protocols including LoRa, BLE, ZigBee, Z-Wave, and more.

## Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
lr1120 = "0.1.0"
embassy-time = "0.5"
```

Basic usage:

```rust
use lr1120_driver::Lr1120;

let mut radio = Lr1120::new(reset_pin, busy_pin, spi_device, nss_pin);
radio.reset().await?;
// Configure and use your preferred protocol
```

## Hardware Requirements

- Semtech LR2021 transceiver module
- SPI-capable microcontroller
- 3 GPIO pins: Reset (output), Busy (input), NSS/CS (output) (not counting SPI SCK/MISO/MOSI)
- Embassy-compatible async runtime

## Documentation & Examples

- **[API Documentation](https://docs.rs/lr1120-driver)** - Complete API reference
- **[Example Applications](https://github.com/TheClams/lr1120-apps)** - Real-world usage examples on Nucleo boards
