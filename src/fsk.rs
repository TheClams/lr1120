//! # API related to FSK operations
//!
//! This module provides an API for configuring and operating the LR1120 chip for FSK (Frequency Shift Keying) modulation.
//! This API is for a generic FSK modulation using a packet structure used in previous Semtech chips
//! (SX126x, SX127x). It supports fixed or dynamic length packets up to 255 bytes, with configurable
//! CRC, address filtering, and whitening.
//!
//! ## Quick Start
//!
//! Here's a typical sequence to initialize the chip for FSK operations:
//!
//! ```rust,no_run
//! use lr1120::radio::PacketType;
//! use lr1120::fsk::{PblLenDetect, AddrComp, FskPktFormat, Crc, DcFree};
//! use lr1120::{PulseShape, RxBw};
//!
//! // Set packet type to FSK Legacy (compatible with SX126x/SX127x/LR11xx)
//! lr1120.set_packet_type(PacketType::FskLegacy).await.expect("Setting packet type");
//!
//! // Configure FSK modulation (250kbps, BT=0.5 pulse shaping, 444kHz bandwidth, 62.5kHz deviation)
//! lr1120.set_fsk_modulation(
//!     250_000,                // Bitrate: 250 kbps
//!     PulseShape::Bt0p5,     // Pulse shaping: BT=0.5 Gaussian filter
//!     RxBw::Bw444,           // RX bandwidth: 444 kHz
//!     62500                  // Frequency deviation: 62.5 kHz
//! ).await.expect("Setting FSK modulation");
//!
//! // Configure syncword (64-bit value, syncword length configured separately in packet params)
//! lr1120.set_fsk_syncword(0xCD05DEADC0FE1337).await.expect("Setting syncword");
//!
//! // Configure packet parameters
//! lr1120.set_fsk_packet(
//!     16,                     // TX preamble length: 16 bits (minimum recommended)
//!     PblLenDetect::Len16Bits, // Preamble detection length: 16 bits
//!     32,                     // Syncword length: 32 bits
//!     AddrComp::Off,          // No address filtering
//!     FskPktFormat::Variable8bit, // Variable length with 8-bit length field
//!     10,                     // Maximum payload length: 10 bytes
//!     Crc::Crc2Byte,          // 2-byte CRC
//!     DcFree::DcFreeWhitening // DC-free encoding enabled (whitening)
//! ).await.expect("Setting packet parameters");
//! ```
//!
//! ## Available Methods
//!
//! - [`set_fsk_modulation`](Lr1120::set_fsk_modulation) - Configure bitrate, pulse shaping, bandwidth, and frequency deviation
//! - [`set_fsk_packet`](Lr1120::set_fsk_packet) - Set packet parameters (preamble, length format, CRC, addressing, whitening)
//! - [`set_fsk_syncword`](Lr1120::set_fsk_syncword) - Configure synchronization word value
//! - [`get_fsk_packet_status`](Lr1120::get_fsk_packet_status) - Read FSK packet status: RSSI, packet length, error source (address, CRC, length, ...)

use embedded_hal::digital::OutputPin;
use embedded_hal_async::spi::SpiBus;

pub use super::cmd::cmd_fsk::*;
use super::{BusyPin, Lr1120, Lr1120Error};

impl<O,SPI, M> Lr1120<O,SPI, M> where
    O: OutputPin, SPI: SpiBus<u8>, M: BusyPin
{

    /// Set Modulation parameters: raw bitrate, pulse shaping, Bandwidth and fdev
    pub async fn set_fsk_modulation(&mut self, bitrate: u32, pulse_shape: PulseShape, rx_bw: RxBw, fdev: u32) -> Result<(), Lr1120Error> {
        let req = set_fsk_modulation_params_cmd(Precision::Basic, bitrate, pulse_shape, rx_bw, fdev);
        self.cmd_wr(&req).await
    }

    // TODO: add dedicated struct and find a good default set of values
    #[allow(clippy::too_many_arguments)]
    /// Set packet parameters (preamble, length format, CRC, addressing, whitening)
    pub async fn set_fsk_packet(&mut self, pbl_len_tx: u16, pbl_len_detect: PblLenDetect, sw_len: u8, addr_comp: AddrComp, fsk_pkt_format: FskPktFormat, pld_len: u8, crc: Crc, dc_free: DcFree) -> Result<(), Lr1120Error> {
        let req = set_fsk_packet_params_cmd(pbl_len_tx, pbl_len_detect, sw_len, addr_comp, fsk_pkt_format, pld_len, crc, dc_free);
        self.cmd_wr(&req).await
    }

    /// Configure syncword
    pub async fn set_fsk_syncword(&mut self, syncword: u64) -> Result<(), Lr1120Error> {
        let req = set_fsk_sync_word_cmd(syncword);
        self.cmd_wr(&req).await
    }

    /// Read FSK packet status: RSSI, packet length, error source (address, CRC, length, ...)
    pub async fn get_fsk_packet_status(&mut self) -> Result<FskPacketStatusRsp, Lr1120Error> {
        let req = get_fsk_packet_status_req();
        let mut rsp = FskPacketStatusRsp::new();
        self.cmd_rd(&req, rsp.as_mut()).await?;
        Ok(rsp)
    }

}