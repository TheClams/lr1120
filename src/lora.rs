//! # API related to LoRa operations
//!
//! This module provides an API for configuring and operating the LR1120 chip for LoRa (Long Range) communication.
//! LoRa is a Semtech proprietary modulation using chirp spread-spectrum, providing the highest sensitivity modulation
//! supported by the LR1120, ideal for communication over multiple kilometers at low bit-rate.
//!
//! ## Quick Start
//!
//! Here's a typical sequence to initialize the chip for LoRa operations:
//!
//! ```rust,no_run
//! use lr1120::radio::PacketType;
//! use lr1120::lora::{Sf, LoraBw};
//!
//! // Set packet type to LoRa
//! lr1120.set_packet_type(PacketType::Lora).await.expect("Setting packet type");
//!
//! // Configure LoRa parameters: modulation & packet format (10 bytes with header and CRC)
//! let modulation = LoraModulationParams::basic(Sf::Sf5, LoraBw::Bw1000);
//! let packet_params = LoraPacketParams::basic(10, &modulation);
//!
//! lr1120.set_lora_modulation(&modulation).await.expect("Setting LoRa modulation");
//! lr1120.set_lora_packet(&packet_params).await.expect("Setting packet parameters");
//! ```
//!
//! ## Available Methods
//!
//! ### Core LoRa Methods
//! - [`set_lora_modulation`](Lr1120::set_lora_modulation) - Configure spreading factor, bandwidth, coding rate, and LDRO
//! - [`set_lora_packet`](Lr1120::set_lora_packet) - Set packet parameters (preamble, payload length, header type, CRC)
//! - [`set_lora_syncword`](Lr1120::set_lora_syncword) - Set syncword using legacy 1-byte format
//! - [`set_lora_syncword_ext`](Lr1120::set_lora_syncword_ext) - Set syncword using extended 2-byte format
//! - [`set_lora_synch_timeout`](Lr1120::set_lora_synch_timeout) - Configure synchronization timeout
//!
//! ### Status and Statistics
//! - [`get_lora_rx_header_info`](Lr1120::get_lora_rx_header_info) - Get RX header information (CRC and coding rate)
//! - [`get_lora_packet_status`](Lr1120::get_lora_packet_status) - Get RSSI/SNR on the last received packet
//!
//! ### Channel Activity Detection (CAD)
//! - [`set_lora_cad_params`](Lr1120::set_lora_cad_params) - Configure CAD parameters for listen-before-talk
//! - [`set_lora_cad`](Lr1120::set_lora_cad) - Start channel activity detection
//!
//! ### Misc Features
//! - [`comp_sx127x_sf6`](Lr1120::comp_sx127x_sf6) - Enable SX127x compatibility for SF6
//!
//! ### Side-Detection (Multi-SF receiver)
//! - [`set_lora_sidedet_cfg`](Lr1120::set_lora_sidedet_cfg) - Configure side-detector for multiple SF detection
//! - [`set_lora_sidedet_syncword`](Lr1120::set_lora_sidedet_syncword) - Configure side-detector syncwords
//!
//! ### Ranging Operations
//! - [`set_ranging_dev_addr`](Lr1120::set_ranging_dev_addr) - Set device address for ranging
//! - [`set_ranging_req_addr`](Lr1120::set_ranging_req_addr) - Set request address for ranging
//! - [`set_ranging_txrx_delay`](Lr1120::set_ranging_txrx_delay) - Set ranging calibration delay
//! - [`get_ranging_base_delay`](Lr1120::get_ranging_base_delay) - Get base delay for ranging based on bandwidth and SF
//! - [`set_ranging_params`](Lr1120::set_ranging_params) - Configure ranging parameters
//! - [`get_ranging_result`](Lr1120::get_ranging_result) - Get basic ranging results (distance)
//! - [`get_ranging_rssi`](Lr1120::get_ranging_rssi) - Get RSSI measured during ranging

use embedded_hal::digital::OutputPin;
use embedded_hal_async::spi::SpiBus;

pub use super::cmd::cmd_lora::*;
pub use super::cmd::cmd_regmem::*;
use super::{BusyPin, Lr1120, Lr1120Error};

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// LoRa Modulation parameters: SF, Bandwidth, Code-rate, LDRO
pub struct LoraModulationParams {
    /// Spreading factor
    pub sf: Sf,
    /// Bandwidth
    pub bw: LoraBw,
    /// Coding Rate
    pub cr: LoraCr,
    /// Low Data-Rate Optimisation
    pub ldro: Ldro,
}

impl LoraModulationParams {
    /// Modulation with default coderate (4/5) and LDRO based on SF/BW
    pub fn basic(sf: Sf, bw: LoraBw) -> Self {
        let ldro_en = (sf==Sf::Sf12 && !matches!(bw,LoraBw::Bw1000|LoraBw::Bw500))
                    || (sf==Sf::Sf11 && !matches!(bw,LoraBw::Bw1000|LoraBw::Bw500|LoraBw::Bw250) );
        Self {
            sf, bw,
            cr: LoraCr::Cr1Ham45Si,
            ldro: if ldro_en {Ldro::On} else {Ldro::Off},
        }
    }

    /// Modulation with default coderate (4/5) and LDRO based on SF/BW
    pub fn new(sf: Sf, bw: LoraBw, cr: LoraCr, ldro: Ldro) -> Self {
        Self {sf, bw, cr, ldro}
    }
}

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// LoRa Modulation parameters: SF, Bandwidth, Code-rate, LDRO
pub struct LoraPacketParams {
    /// Preamble length (in symbol)
    pub pbl_len: u16,
    /// Payload length (in byte)
    pub payload_len: u8,
    /// Explicit or implicit header
    pub header_type: HeaderType,
    /// CRC Enable
    pub crc_en: bool,
    /// Chirp direction
    pub invert_iq: bool,
}

impl LoraPacketParams {
    /// Default Packet parameters (Explicit header with CRC and standard direction)
    pub fn basic(payload_len: u8, modulation: &LoraModulationParams) -> Self {
        Self {
            pbl_len: if modulation.sf < Sf::Sf7 {12} else {8},
            payload_len,
            header_type: HeaderType::Explicit,
            crc_en: true,
            invert_iq: false
        }
    }

    /// Modulation with default coderate (4/5) and LDRO based on SF/BW
    pub fn new(pbl_len: u16, payload_len: u8, header_type: HeaderType, crc_en: bool, invert_iq: bool) -> Self {
        Self {pbl_len, payload_len, header_type, crc_en, invert_iq}
    }
}

#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// LoRa CAD parameters: SF, Bandwidth, Code-rate, LDRO
pub struct LoraCadParams {
    /// Number of symbols (1 to 15)
    nb_symbols: u8,
    /// Exit Mode: Idle, RX (low power detect on long preamble) or TX (for Listen-Before-Talk)
    pub exit_mode: ExitMode,
    /// timeout for the following RX or TX if exit mode is not CAD_ONLY
    pub timeout: u32,
    /// Detection threshold
    pub thr: u8,
}

/// Recommended CAD threshold for a given SF and number of symbols
pub fn lora_cad_thr(sf: Sf, nb_symbols: u8) -> u8 {
    let base_symb = match nb_symbols {
        0 | 1 => 60,
        2 => 56,
        3 => 52,
        _ => 51
    };
    let offset_sf = match sf {
        Sf::Sf5 => 0,
        Sf::Sf6 => 1,
        Sf::Sf7 => 2,
        Sf::Sf8 => 3,
        Sf::Sf9 => 5,
        Sf::Sf10 => 6,
        Sf::Sf11 => 8,
        Sf::Sf12 => 10,
    };
    base_symb + offset_sf
}

impl LoraCadParams {

    /// Create CAD parameter for a CAD only operation
    pub fn new_cad_only(sf: Sf, nb_symbols: u8) -> Self {
        let nb_symbols = nb_symbols.clamp(1,15);
        let thr = lora_cad_thr(sf, nb_symbols);
        LoraCadParams {
            nb_symbols,
            exit_mode: ExitMode::CadOnly,
            timeout: 0,
            thr
        }
    }

    /// Create CAD parameter with automatic detection threshold
    pub fn new_auto(sf: Sf, nb_symbols: u8, exit_mode: ExitMode, timeout: u32) -> Self {
        let nb_symbols = nb_symbols.clamp(1,15);
        let thr = lora_cad_thr(sf, nb_symbols);
        LoraCadParams {nb_symbols,exit_mode,timeout,thr}
    }

    /// Create CAD parameter with manual detection threshold
    pub fn new(nb_symbols: u8, thr: u8, exit_mode: ExitMode, timeout: u32) -> Self {
        let nb_symbols = nb_symbols.clamp(1,15);
        LoraCadParams {nb_symbols,exit_mode,timeout,thr}
    }
}

// Recommneded delay for ranging
// One line per bandwidth: 500, 250, 125
const RANGING_DELAY : [u32; 24] = [
    19115, 19113, 19121, 19127, 19141, 19178, 19242, 19370,
    20265, 20266, 20279, 20292, 20236, 20305, 20433, 20689,
    20154, 20268, 20298, 20319, 20323, 20314, 20570, 21082,
];

#[derive(Debug, Clone, Copy)]
pub struct SidedetCfg(u8);
impl SidedetCfg {
    pub fn new(sf: Sf, ldro: Ldro, inv: bool) -> Self{
        let b = ((sf as u8) << 4) |
            (ldro as u8) << 2 |
            if inv {1} else {0};
        Self(b)
    }

    pub fn to_byte(&self) -> u8 {
        self.0
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// LoRa Blanking configuration
pub struct BlankingCfg {
    /// Threshold on SNR margin (0.5dB) to enable symbol domain blanking (0-15)
    pub snr_thr     : u8,
    /// Gain (0-3) to adapt threshold based on average SNR
    pub thr_gain: u8,
    /// Symbol domain blanking coefficient (0 to 3, with 0 being hard-blanking)
    pub symb_gain: u8,
    /// Threshold on RSSI (0.5dB) for time domain blanking (0-15)
    pub rssi_thr: u8,
    /// Enable Time domain blanking during detection
    pub detect  : bool,
}

impl BlankingCfg {

    /// Blanking disabled
    pub fn off() -> Self {
        Self {
            thr_gain: 0,
            snr_thr : 0,
            symb_gain: 0,
            detect  : false,
            rssi_thr: 0,
        }
    }

    /// Blanking enabled at symbol domain only
    pub fn symbol() -> Self {
        Self {
            thr_gain: 2,
            snr_thr : 8,
            symb_gain: 2,
            detect  : false,
            rssi_thr: 0,
        }
    }

    /// Blanking enabled at time-Domain & symbol domain
    pub fn td_symb() -> Self {
        Self {
            thr_gain: 2,
            snr_thr : 8,
            symb_gain: 2,
            detect  : false,
            rssi_thr: 7,
        }
    }

    /// Blanking fully enabled including during detection
    pub fn full() -> Self {
        Self {
            thr_gain: 2,
            snr_thr : 8,
            symb_gain: 2,
            detect  : true,
            rssi_thr: 7,
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// Frequency estimation during ranging exchange (valid only on responder side)
pub struct RangingFei {
    /// Frequency estimation on first exchange
    pub fei1: i32,
    /// Frequency estimation on second exchange
    pub fei2: i32,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// Define duration of the TimingSync pulse of the responder
pub enum TimingSyncPulseWidth {
    W1 = 0, W5 = 1, W52 = 2, W520 = 3, W5200 = 4, W52k = 5, W260k = 6, W1024k = 7
}


impl<O,SPI, M> Lr1120<O,SPI, M> where
    O: OutputPin, SPI: SpiBus<u8>, M: BusyPin
{

    /// Set LoRa Modulation parameters
    pub async fn set_lora_modulation(&mut self, params: &LoraModulationParams) -> Result<(), Lr1120Error> {
        let req = set_lora_modulation_params_cmd(params.sf, params.bw, params.cr, params.ldro);
        self.cmd_wr(&req).await
    }

    /// Set LoRa Packet parameters
    pub async fn set_lora_packet(&mut self, params: &LoraPacketParams) -> Result<(), Lr1120Error> {
        let req = set_lora_packet_params_cmd(params.pbl_len, params.header_type, params.payload_len,  params.crc_en, params.invert_iq);
        self.cmd_wr(&req).await
    }

    /// Set LoRa Syncword using legacy (SX127x) 1B notation: 0x34 for public network, 0x12 for private
    pub async fn set_lora_syncword(&mut self, syncword: u8) -> Result<(), Lr1120Error> {
        let req = set_lora_syncword_cmd(syncword);
        self.cmd_wr(&req).await
    }

    /// Set LoRa Syncword, using 2B notation (2 values on 5b each)
    /// Public network is (6,8) and private network is (2,4)
    pub async fn set_lora_syncword_ext(&mut self, s1: i8, s2: i8) -> Result<(), Lr1120Error> {
        let reg_val = ((s1&0x1F) as u32) | (((s2&0x1F) as u32) << 8);
        let req =  write_reg_mem_mask32_cmd(0xF20460, 0x1FFF, reg_val);
        self.cmd_wr(&req).await
    }

    /// Set synchronisation timeout
    /// Timeout is given in number of symbol
    pub async fn set_lora_synch_timeout(&mut self, timeout: u8) -> Result<(), Lr1120Error> {
        let req = set_lora_synch_timeout_cmd(timeout);
        self.cmd_wr(&req).await
    }

    /// Return RX Header information: CRC On/Off and Coding Rate
    pub async fn get_lora_rx_header_info(&mut self) -> Result<LoraRxHeaderInfosRsp, Lr1120Error> {
        let req = get_lora_rx_header_infos_req();
        let mut rsp = LoraRxHeaderInfosRsp::new();
        self.cmd_rd(&req, rsp.as_mut()).await?;
        Ok(rsp)
    }

    /// Read LoRa RX stats (RSSI/SNR)
    pub async fn get_lora_packet_status(&mut self) -> Result<LoraPacketStatusRsp, Lr1120Error> {
        let req = get_lora_packet_status_req();
        let mut rsp = LoraPacketStatusRsp::new();
        self.cmd_rd(&req, rsp.as_mut()).await?;
        Ok(rsp)
    }

    /// Set LoRa Channel Activity Detection parameters
    pub async fn set_lora_cad_params(&mut self, params: LoraCadParams) -> Result<(), Lr1120Error> {
        let req = set_lora_cad_params_cmd(params.nb_symbols, params.thr, 10, params.exit_mode, params.timeout);
        self.cmd_wr(&req).await
    }

    /// Start a LoRa Channel Activity Detection (CAD)
    pub async fn set_lora_cad(&mut self) -> Result<(), Lr1120Error> {
        let req = set_lora_cad_cmd();
        self.cmd_wr(&req).await
    }

    /// Enable compatibility with SX127x for SF6 communication
    /// Must be called after each SetLoraModulation
    pub async fn comp_sx127x_sf6(&mut self, en: bool) -> Result<(), Lr1120Error> {
        let req =  write_reg_mem_mask32_cmd(0xF20414, 0x00040000, en as u32);
        self.cmd_wr(&req).await
    }

    #[allow(clippy::get_first)]
    /// Configure Side-Detector allowing multiple SF to be detected
    /// Must be called after set_lora_modulation
    /// If cfg is an empty slice, this disabled all side-detector
    pub async fn set_lora_sidedet_cfg(&mut self, cfg: &[SidedetCfg]) -> Result<(), Lr1120Error> {
        let req = [
            0x02, 0x24,
            cfg.get(0).map(|c| c.to_byte()).unwrap_or(0),
            cfg.get(1).map(|c| c.to_byte()).unwrap_or(0),
            cfg.get(2).map(|c| c.to_byte()).unwrap_or(0),
        ];
        let len = cfg.len() + 2;
        self.cmd_wr(&req[..len]).await
    }

    #[allow(clippy::get_first)]
    /// Configure Side-Detector Syncword using basic syncword format
    pub async fn set_lora_sidedet_syncword(&mut self, sw: &[u8]) -> Result<(), Lr1120Error> {
        let req = [
            0x02, 0x25,
            sw.get(0).copied().unwrap_or(0x24),
            sw.get(1).copied().unwrap_or(0x24),
            sw.get(2).copied().unwrap_or(0x24),
        ];
        let len = sw.len() + 2;
        self.cmd_wr(&req[..len]).await
    }

    /// Set the device address for ranging operation
    /// The device will answer to ranging request only if the request address matches the device address
    /// The length allows to define how many bytes from the address are checked (starting from LSB)
    pub async fn set_ranging_dev_addr(&mut self, addr: u32, length: Option<CheckLength>) -> Result<(), Lr1120Error> {
         let req = set_ranging_addr_cmd(addr, length.unwrap_or(CheckLength::Addr32b));
        self.cmd_wr(&req).await
   }

    /// Set the request address for ranging operation
    pub async fn set_ranging_req_addr(&mut self, addr: u32) -> Result<(), Lr1120Error> {
         let req = set_ranging_req_addr_cmd(addr);
        self.cmd_wr(&req).await
    }

    /// Set the ranging calibration value
    pub async fn set_ranging_txrx_delay(&mut self, delay: u32) -> Result<(), Lr1120Error> {
         let req = set_ranging_tx_rx_delay_cmd(delay);
        self.cmd_wr(&req).await
   }

    /// Get the base delay for ranging depdending on bandwidth and SF
    /// Delay was calibrated only for bandwidth 125kHz, 250kHz and 500kHz.
    pub fn get_ranging_base_delay(&self, modulation: &LoraModulationParams) -> u32 {
        let offset = match modulation.bw {
            LoraBw::Bw500 =>  0,
            LoraBw::Bw250 =>  8,
            LoraBw::Bw125 =>  16,
            _ => 32,
        };
        let idx = offset + (modulation.sf as usize - 5);
        RANGING_DELAY.get(idx).copied().unwrap_or(18000 - (5600 >> (12 - modulation.sf as u32)))
    }

    /// Set the ranging parameters: number of symbols
    /// Number of symbols should typically be between 8 to 16 symbols, with 12 being close to optimal performances
    pub async fn set_ranging_params(&mut self, nb_symbols: u8) -> Result<(), Lr1120Error> {
         let req = set_ranging_parameter_cmd(0, nb_symbols);
        self.cmd_wr(&req).await
   }

    /// Return the result of last ranging exchange (round-trip time of flight and RSSI)
    /// The distance is provided
    pub async fn get_ranging_result(&mut self) -> Result<RangingResultRsp, Lr1120Error> {
        let req = get_ranging_result_req(RangingResKind::Distance);
        let mut rsp = RangingResultRsp::new();
        self.cmd_rd(&req, rsp.as_mut()).await?;
        Ok(rsp)
    }

    /// Return the RSSI measured during ranging
    pub async fn get_ranging_rssi(&mut self) -> Result<RangingRssiRsp, Lr1120Error> {
        let req = get_ranging_result_req(RangingResKind::Rssi);
        let mut rsp = RangingRssiRsp::new();
        self.cmd_rd(&req, rsp.as_mut()).await?;
        Ok(rsp)
    }

}
