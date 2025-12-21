// Radio commands API

use crate::status::Status;

/// Device mode between TX and RX modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum IntermediaryMode {
    Sleep = 0,
    StdbyRc = 1,
    StdbyXosc = 2,
    Fs = 3,
}

/// Fallback mode after RX or TX
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum FallbackMode {
    StdbyRc = 1,
    StdbyXosc = 2,
    Fs = 3,
}

/// 0: RX Boosted mode deactivated, 1: RX Boosted mode activated, other values RFU
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum RxBoosted {
    Deactivated = 0,
    Activated = 1,
}

/// PA selection: Low0Power, High Power or High Frequency
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum PaSel {
    LpPa = 0,
    HpPa = 1,
    HfPa = 2,
}

/// PA power source: 0x00: Internal regulator (VREG), 0x01: VBAT. Must use 0x01 when TxPower > 14
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum RegPaSupply {
    Vreg = 0,
    Vbat = 1,
}

/// PA power ramping time
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum RampTime {
    Ramp16u = 0,
    Ramp32u = 1,
    Ramp48u = 2,
    Ramp64u = 3,
    Ramp80u = 4,
    Ramp96u = 5,
    Ramp112u = 6,
    Ramp128u = 7,
    Ramp144u = 8,
    Ramp160u = 9,
    Ramp176u = 10,
    Ramp192u = 11,
    Ramp208u = 12,
    Ramp240u = 13,
    Ramp272u = 14,
    Ramp304u = 15,
}

/// Modem selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum PacketType {
    None = 0,
    Gfsk = 1,
    Lora = 2,
    SigfoxUl = 3,
    LrFhss = 4,
    Ranging = 5,
    Ble = 6,
}

impl From<u8> for PacketType {
    fn from(value: u8) -> Self {
        match value {
            6 => PacketType::Ble,
            5 => PacketType::Ranging,
            4 => PacketType::LrFhss,
            3 => PacketType::SigfoxUl,
            2 => PacketType::Lora,
            1 => PacketType::Gfsk,
            _ => PacketType::None,
        }
    }
}

/// Sets the RF (PLL) frequency of the radio in Hz. Sub-GHz path selected for frequencies ≤1.50GHz, HF path for higher frequencies. All frequency dependent parameters automatically recomputed.
pub fn set_rf_frequency_cmd(rf_freq: u32) -> [u8; 6] {
    let mut cmd = [0u8; 6];
    cmd[0] = 0x02;
    cmd[1] = 0x0B;

    cmd[2] |= ((rf_freq >> 24) & 0xFF) as u8;
    cmd[3] |= ((rf_freq >> 16) & 0xFF) as u8;
    cmd[4] |= ((rf_freq >> 8) & 0xFF) as u8;
    cmd[5] |= (rf_freq & 0xFF) as u8;
    cmd
}

/// Sets radio in RX mode. Sub-GHz path for ≤1.50GHz, HF path above. After timeout, returns to Standby RC. BUSY goes low after entering RX mode. Fails if no packet type configured or packet type doesn't allow RX.
pub fn set_rx_cmd(rx_timeout: u32) -> [u8; 5] {
    let mut cmd = [0u8; 5];
    cmd[0] = 0x02;
    cmd[1] = 0x09;

    cmd[2] |= ((rx_timeout >> 16) & 0xFF) as u8;
    cmd[3] |= ((rx_timeout >> 8) & 0xFF) as u8;
    cmd[4] |= (rx_timeout & 0xFF) as u8;
    cmd
}

/// Sets radio in TX mode, triggers RF packet transmission with RTC timeout. After TX_DONE or TIMEOUT, returns to STBY_RC (default), STBY_XOSC or FS per FallBackMode config. BUSY goes low after entering TX mode. Fails if no packet type configured or packet type doesn't allow TX.
pub fn set_tx_cmd(tx_timeout: u32) -> [u8; 5] {
    let mut cmd = [0u8; 5];
    cmd[0] = 0x02;
    cmd[1] = 0x0A;

    cmd[2] |= ((tx_timeout >> 16) & 0xFF) as u8;
    cmd[3] |= ((tx_timeout >> 8) & 0xFF) as u8;
    cmd[4] |= (tx_timeout & 0xFF) as u8;
    cmd
}

/// Sets device in TX continuous wave mode (unmodulated carrier). Immediately enters TX CW mode. Operating frequency and PA configuration commands (including RF output power) must be called PRIOR to this command. Used for ETSI D-M1 test (unmodulated carrier) and FCC Part 15.247 compliance testing.
pub fn set_tx_cw_cmd() -> [u8; 2] {
    [0x02, 0x19]
}

/// Transmits infinite preamble sequence. Immediately starts transmission. Operating frequency, PA configuration commands (including RF output power), and packet type must be called PRIOR to this command. Used for ETSI D-M2 test (continuously modulated signal with greatest occupied RF bandwidth).
pub fn set_tx_infinite_preamble_cmd() -> [u8; 2] {
    [0x02, 0x1A]
}

/// Automatically performs transition to RX after TX or TX after RX. After second mode, returns to Standby RC. Not used if Rx Duty Cycle is started.
pub fn auto_tx_rx_cmd(delay: u32, intermediary_mode: IntermediaryMode, timeout: u32) -> [u8; 9] {
    let mut cmd = [0u8; 9];
    cmd[0] = 0x02;
    cmd[1] = 0x0C;

    cmd[2] |= ((delay >> 16) & 0xFF) as u8;
    cmd[3] |= ((delay >> 8) & 0xFF) as u8;
    cmd[4] |= (delay & 0xFF) as u8;
    cmd[5] |= intermediary_mode as u8;
    cmd[6] |= ((timeout >> 16) & 0xFF) as u8;
    cmd[7] |= ((timeout >> 8) & 0xFF) as u8;
    cmd[8] |= (timeout & 0xFF) as u8;
    cmd
}

/// Defines device mode after packet transmission or reception. Not used if Rx Duty Cycle started or AutoRxTx configured. After RX timeout, device returns to Standby RC regardless of fallback config.
pub fn set_rx_tx_fallback_mode_cmd(fallback_mode: FallbackMode) -> [u8; 3] {
    let mut cmd = [0u8; 3];
    cmd[0] = 0x02;
    cmd[1] = 0x13;

    cmd[2] |= fallback_mode as u8;
    cmd
}

/// Periodically opens RX windows. Between windows, device goes to Sleep mode with retention. Configure 32kHz clock source before entering Duty Cycle. AutoRxTx and SetRxTxFallback not used in this mode. Returns CMD_FAIL if packet type not set.
pub fn set_rx_duty_cycle_cmd(rx_period: u32, sleep_period: u32, use_lora_cad: bool) -> [u8; 9] {
    let mut cmd = [0u8; 9];
    cmd[0] = 0x02;
    cmd[1] = 0x14;

    cmd[2] |= ((rx_period >> 16) & 0xFF) as u8;
    cmd[3] |= ((rx_period >> 8) & 0xFF) as u8;
    cmd[4] |= (rx_period & 0xFF) as u8;
    cmd[5] |= ((sleep_period >> 16) & 0xFF) as u8;
    cmd[6] |= ((sleep_period >> 8) & 0xFF) as u8;
    cmd[7] |= (sleep_period & 0xFF) as u8;
    if use_lora_cad { cmd[8] |= 1; }
    cmd
}

/// Defines if RX timeout should be stopped on Syncword/Header detection or Preamble detection
pub fn stop_timeout_on_preamble_cmd(stop_on_preamble: bool) -> [u8; 3] {
    let mut cmd = [0u8; 3];
    cmd[0] = 0x02;
    cmd[1] = 0x17;

    if stop_on_preamble { cmd[2] |= 1; }
    cmd
}

/// Returns instantaneous RSSI value at the time command is sent. If no RF packet present, returns RF noise. Formula: RSSI (dBm) = -Rssi/2
pub fn get_rssi_inst_req() -> [u8; 2] {
    [0x02, 0x05]
}

/// Returns internal statistics of received RF packets. Statistics reset on Power On Reset, power down, or ResetStats command.
pub fn get_stats_req() -> [u8; 2] {
    [0x02, 0x01]
}

/// Resets the internal statistics of received RF packets
pub fn reset_stats_cmd() -> [u8; 2] {
    [0x02, 0x00]
}

/// Returns the length of last packet received and offset in RX buffer of first byte received
pub fn get_rx_buffer_status_req() -> [u8; 2] {
    [0x02, 0x03]
}

/// Sets device in RX Boosted mode, allowing ~2dB increased sensitivity at expense of ~2mA higher RX current consumption
pub fn set_rx_boosted_cmd(rx_boosted: RxBoosted) -> [u8; 3] {
    let mut cmd = [0u8; 3];
    cmd[0] = 0x02;
    cmd[1] = 0x27;

    cmd[2] |= rx_boosted as u8;
    cmd
}

/// Selects which PA to use and configures the PA supply. Must be called before SetTxParams. No automatic frequency limitation during PA selection - frequency must match external matching network capability.
pub fn set_pa_config_cmd(pa_sel: PaSel, reg_pa_supply: RegPaSupply, pa_duty_cycle: u8, pa_hp_sel: u8) -> [u8; 6] {
    let mut cmd = [0u8; 6];
    cmd[0] = 0x02;
    cmd[1] = 0x15;

    cmd[2] |= pa_sel as u8;
    cmd[3] |= reg_pa_supply as u8;
    cmd[4] |= pa_duty_cycle;
    cmd[5] |= pa_hp_sel;
    cmd
}

/// Sets TX power and ramp time of selected PA. SetPaConfig must be sent prior to this command. 48us ramp time recommended for best trade-off between fast RF establishment and minimum spurious emissions.
pub fn set_tx_params_cmd(tx_power: i8, ramp_time: RampTime) -> [u8; 4] {
    let mut cmd = [0u8; 4];
    cmd[0] = 0x02;
    cmd[1] = 0x11;

    cmd[2] |= (tx_power) as u8;
    cmd[3] |= ramp_time as u8;
    cmd
}

#[allow(clippy::too_many_arguments)]
/// Sets gain offset for on-chip power estimation. LR1120 internal LNA has predefined gains (G4-G13 with sub-gains). RSSI must be calibrated per hardware type, not per device. Default calibration for 868-915MHz on EVK.
pub fn set_rssi_calibration_cmd(tune_g4: u8, tune_g5: u8, tune_g6: u8, tune_g7: u8, tune_g8: u8, tune_g9: u8, tune_g10: u8, tune_g11: u8, tune_g12: u8, tune_g13: u8, tune_g13_hp1: u8, tune_g13_hp2: u8, tune_g13_hp3: u8, tune_g13_hp4: u8, tune_g13_hp5: u8, tune_g13_hp6: u8, tune_g13_hp7: u8, gain_offset: u16) -> [u8; 12] {
    let mut cmd = [0u8; 12];
    cmd[0] = 0x02;
    cmd[1] = 0x29;

    cmd[2] |= (tune_g4 & 0xF) << 4;
    cmd[2] |= tune_g5 & 0xF;
    cmd[3] |= (tune_g6 & 0xF) << 4;
    cmd[3] |= tune_g7 & 0xF;
    cmd[4] |= (tune_g8 & 0xF) << 4;
    cmd[4] |= tune_g9 & 0xF;
    cmd[5] |= (tune_g10 & 0xF) << 4;
    cmd[5] |= tune_g11 & 0xF;
    cmd[6] |= (tune_g12 & 0xF) << 4;
    cmd[6] |= tune_g13 & 0xF;
    cmd[7] |= (tune_g13_hp1 & 0xF) << 4;
    cmd[7] |= tune_g13_hp2 & 0xF;
    cmd[8] |= (tune_g13_hp3 & 0xF) << 4;
    cmd[8] |= tune_g13_hp4 & 0xF;
    cmd[9] |= (tune_g13_hp5 & 0xF) << 4;
    cmd[9] |= tune_g13_hp6 & 0xF;
    cmd[10] |= (tune_g13_hp7 & 0xF) << 4;
    cmd[10] |= ((gain_offset >> 8) & 0xFF) as u8;
    cmd[11] |= (gain_offset & 0xFF) as u8;
    cmd
}

/// Defines which modem to use for next RF transactions. First command to call before RX/TX and before defining modulation/packet parameters. Only works in Standby RC, Standby Xosc or FS mode, otherwise returns CMD_FAIL.
pub fn set_packet_type_cmd(packet_type: PacketType) -> [u8; 3] {
    let mut cmd = [0u8; 3];
    cmd[0] = 0x02;
    cmd[1] = 0x0E;

    cmd[2] |= packet_type as u8;
    cmd
}

/// Returns current protocol of the radio
pub fn get_packet_type_req() -> [u8; 2] {
    [0x02, 0x02]
}

// Response structs

/// Response for GetRssiInst command
#[derive(Default)]
pub struct RssiInstRsp([u8; 2]);

impl RssiInstRsp {
    /// Create a new response buffer
    pub fn new() -> Self {
        Self::default()
    }

    /// Return Status
    pub fn status(&mut self) -> Status {
        self.0[0].into()
    }

    /// Instantaneous RSSI value. Convert to dBm: RSSI(dBm) = -rssi/2
    pub fn rssi(&self) -> u8 {
        self.0[1]
    }
}

impl AsMut<[u8]> for RssiInstRsp {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

/// Response for GetStats command
#[derive(Default)]
pub struct StatsRsp([u8; 9]);

impl StatsRsp {
    /// Create a new response buffer
    pub fn new() -> Self {
        Self::default()
    }

    /// Return Status
    pub fn status(&mut self) -> Status {
        self.0[0].into()
    }

    /// Total number of received packets
    pub fn pkt_rx(&self) -> u16 {
        (self.0[2] as u16) |
        ((self.0[1] as u16) << 8)
    }

    /// Total number of received packets with CRC error
    pub fn crc_error(&self) -> u16 {
        (self.0[4] as u16) |
        ((self.0[3] as u16) << 8)
    }

    /// Header error. In LoRa, a header error is detect when the checksum fails, while in FSK, the header is in error only when the packet length is higher than the programmed length.
    pub fn header_error(&self) -> u16 {
        (self.0[6] as u16) |
        ((self.0[5] as u16) << 8)
    }

    /// False synchronisation counter (LoRa only)
    pub fn false_sync(&self) -> u16 {
        (self.0[8] as u16) |
        ((self.0[7] as u16) << 8)
    }
}

impl AsMut<[u8]> for StatsRsp {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

/// Response for GetRxBufferStatus command
#[derive(Default)]
pub struct RxBufferStatusRsp([u8; 3]);

impl RxBufferStatusRsp {
    /// Create a new response buffer
    pub fn new() -> Self {
        Self::default()
    }

    /// Return Status
    pub fn status(&mut self) -> Status {
        self.0[0].into()
    }

    /// Payload length of last packet received in bytes
    pub fn pld_len(&self) -> u8 {
        self.0[1]
    }

    /// Offset in RX buffer of first byte received
    pub fn offset(&self) -> u8 {
        self.0[2]
    }
}

impl AsMut<[u8]> for RxBufferStatusRsp {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

/// Response for GetPacketType command
#[derive(Default)]
pub struct PacketTypeRsp([u8; 2]);

impl PacketTypeRsp {
    /// Create a new response buffer
    pub fn new() -> Self {
        Self::default()
    }

    /// Return Status
    pub fn status(&mut self) -> Status {
        self.0[0].into()
    }

    /// Current modem
    pub fn packet_type(&self) -> PacketType {
        self.0[1].into()
    }
}

impl AsMut<[u8]> for PacketTypeRsp {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}
