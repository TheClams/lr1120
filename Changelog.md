# Change Log

## [0.5.0] - Unreleased

### Added
  - Add API for rx_stats and packet status (LoRa/FSK)
  - Add method to irq to check any rx error (length, address, header, crc)

### Changed
  - `clear_irqs` now accept an Option which default to all IRQs

### Fixed
  - Fix IRQ mask for GNNS abort


## [0.4.0] - 2025-12-19

### Fixed
  - Fix some defmt code not compiling


## [0.3.0] - 2025-12-14

### Added
  - Add API for GNSS and CryptoEngine

### Fixed
  - Fix wifi_get_result_* (were fully broken)

## [0.2.0] - 2025-12-12

### Added
  - Add API for WiFi scanning
  - Add API for TX/RX Buffer

## [0.1.0] - 2025-12-07

### Added
  - Basic driver to send command and receive
  - Generate all raw commands
  - Add API to call system/radio command and control LoRa/FSK modem

