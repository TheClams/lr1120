// Lora commands API

use crate::status::Status;

/// Spreading factor
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Sf {
    Sf5 = 5,
    Sf6 = 6,
    Sf7 = 7,
    Sf8 = 8,
    Sf9 = 9,
    Sf10 = 10,
    Sf11 = 11,
    Sf12 = 12,
}

/// LoRa bandwidth
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum LoraBw {
    Bw7 = 0,
    Bw15 = 1,
    Bw31 = 2,
    Bw62 = 3,
    Bw125 = 4,
    Bw250 = 5,
    Bw500 = 6,
    Bw1000 = 7,
    Bw10 = 8,
    Bw20 = 9,
    Bw41 = 10,
    Bw83 = 11,
    Bw101 = 12,
    Bw203 = 13,
    Bw406 = 14,
    Bw812 = 15,
}

impl LoraBw {
    /// Return Bandwidth in Hz
    pub fn to_hz(&self) -> u32 {
        match self {
            LoraBw::Bw1000 => 1_000_000,
            LoraBw::Bw812  =>   812_500,
            LoraBw::Bw500  =>   500_000,
            LoraBw::Bw406  =>   406_250,
            LoraBw::Bw250  =>   250_000,
            LoraBw::Bw203  =>   203_125,
            LoraBw::Bw125  =>   125_000,
            LoraBw::Bw101  =>   101_562,
            LoraBw::Bw83   =>    83_333,
            LoraBw::Bw62   =>    62_500,
            LoraBw::Bw41   =>    41_666,
            LoraBw::Bw31   =>    31_250,
            LoraBw::Bw20   =>    20_833,
            LoraBw::Bw15   =>    15_625,
            LoraBw::Bw10   =>    10_416,
            LoraBw::Bw7    =>     7_812,
        }
    }

    /// Flag Fractional bandwidth
    /// Corresponds to band used in SX1280
    pub fn is_fractional(&self) -> bool {
        use LoraBw::*;
        matches!(self, Bw812 | Bw406 | Bw203 | Bw101)
    }
}

impl PartialOrd for LoraBw {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for LoraBw {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.to_hz().cmp(&other.to_hz())
    }
}

/// Coding rate. Note that for Long interleaver (LI) minimum payload is 8 bytes and max is 253 bytes (CRC on) or 255 bytes (CRC off)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum LoraCr {
    NoCoding = 0,
    Cr1Ham45Si = 1,
    Cr2Ham23Si = 2,
    Cr3Ham47Si = 3,
    Cr4Ham12Si = 4,
    Cr5Ham45Li = 5,
    Cr6Ham23Li = 6,
    Cr7Ham12Li = 7,
}

impl LoraCr {
    /// Return if Code-rate uses long interleaving
    pub fn is_li(&self) -> bool {
        use LoraCr::*;
        matches!(self, Cr5Ham45Li|Cr6Ham23Li|Cr7Ham12Li)
    }
    /// Return denominator for the coding rate, supposing a numerator equal to 4
    pub fn denominator(&self) -> u8 {
        match self {
            LoraCr::NoCoding   => 4,
            // Code rate 4/5
            LoraCr::Cr1Ham45Si |
            LoraCr::Cr5Ham45Li => 5,
            // Code rate 2/3 -> 4/6
            LoraCr::Cr2Ham23Si |
            LoraCr::Cr6Ham23Li => 6,
            // Code rate 4/7
            LoraCr::Cr3Ham47Si => 7,
            // Code rate 1/2 -> 4/8
            LoraCr::Cr4Ham12Si |
            LoraCr::Cr7Ham12Li => 8,
        }
    }
}

impl From<u8> for LoraCr {
    fn from(value: u8) -> Self {
        match value {
            7 => LoraCr::Cr7Ham12Li,
            6 => LoraCr::Cr6Ham23Li,
            5 => LoraCr::Cr5Ham45Li,
            4 => LoraCr::Cr4Ham12Si,
            3 => LoraCr::Cr3Ham47Si,
            2 => LoraCr::Cr2Ham23Si,
            1 => LoraCr::Cr1Ham45Si,
            _ => LoraCr::NoCoding,
        }
    }
}

/// Low Data Rate Optimisation. Enable for high Spreading factor to increase tolerance to clock drift (mandatory for SF11/SF12 at BW125, and SF12 at BW250)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Ldro {
    Off = 0,
    On = 1,
}

/// 0x00: Explicit header (default), 0x01: Implicit header
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum HeaderType {
    Explicit = 0,
    Implicit = 1,
}

/// 0x00: CAD_ONLY (return to STBY_RC), 0x01: CAD_RX (stay in RX if activity detected), 0x10: CAD_LBT (go to TX if no activity)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ExitMode {
    CadOnly = 0,
    CadRx = 1,
    CadLbt = 16,
}

/// Defines how many of the 4 bytes of the address are checked against the request address sent by the initiator. Checked bytes are the LSB if check_length<4
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum CheckLength {
    Addr8b = 1,
    Addr16b = 2,
    Addr24b = 3,
    Addr32b = 4,
}

/// Result type: 0: Last ranging distance result, 1: Last ranging RSSI result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum RangingResKind {
    Distance = 0,
    Rssi = 1,
}

/// Configures LoRa modulation parameters (SF, BW, CR, LDRO). Returns CMD_FAIL if packet type is not LoRa. SF5/SF6 compatible with SX126x. SF6 can be made compatible with SX127x in implicit mode via register setting.
pub fn set_lora_modulation_params_cmd(sf: Sf, lora_bw: LoraBw, lora_cr: LoraCr, ldro: Ldro) -> [u8; 6] {
    let mut cmd = [0u8; 6];
    cmd[0] = 0x02;
    cmd[1] = 0x0F;

    cmd[2] |= sf as u8;
    cmd[3] |= lora_bw as u8;
    cmd[4] |= (lora_cr as u8) & 0x7;
    cmd[5] |= (ldro as u8) & 0x3;
    cmd
}

/// Configures LoRa RF packet parameters. Fails if no packet type set.
pub fn set_lora_packet_params_cmd(pbl_len: u16, header_type: HeaderType, pld_len: u8, crc_en: bool, invert_iq: bool) -> [u8; 8] {
    let mut cmd = [0u8; 8];
    cmd[0] = 0x02;
    cmd[1] = 0x10;

    cmd[2] |= ((pbl_len >> 8) & 0xFF) as u8;
    cmd[3] |= (pbl_len & 0xFF) as u8;
    cmd[4] |= header_type as u8;
    cmd[5] |= pld_len;
    if crc_en { cmd[6] |= 1; }
    if invert_iq { cmd[7] |= 1; }
    cmd
}

/// Activates Channel Activity Detection (CAD) feature. Searches for LoRa preamble signal in both preamble and payload fields. Triggers CADdone IRQ when complete, CadDetected IRQ if valid signal found. Returns to STDBY_RC after completion. Minimum 2 symbols recommended.
pub fn set_lora_cad_cmd() -> [u8; 2] {
    [0x02, 0x18]
}

/// Defines LoRa CAD parameters. DetPeak/DetMin depend on SF, BW, and symbol count - must be carefully tested for good sensitivity and minimal false detections.
pub fn set_lora_cad_params_cmd(nb_symbols: u8, det_peak: u8, det_min: u8, exit_mode: ExitMode, timeout: u32) -> [u8; 9] {
    let mut cmd = [0u8; 9];
    cmd[0] = 0x02;
    cmd[1] = 0x0D;

    cmd[2] |= nb_symbols;
    cmd[3] |= det_peak;
    cmd[4] |= det_min;
    cmd[5] |= exit_mode as u8;
    cmd[6] |= ((timeout >> 16) & 0xFF) as u8;
    cmd[7] |= ((timeout >> 8) & 0xFF) as u8;
    cmd[8] |= (timeout & 0xFF) as u8;
    cmd
}

/// Configures LoRa modem to issue RX timeout after exactly SymbolNum symbols if no packet detected
pub fn set_lora_synch_timeout_cmd(symbol_num: u8) -> [u8; 3] {
    let mut cmd = [0u8; 3];
    cmd[0] = 0x02;
    cmd[1] = 0x1B;

    cmd[2] |= symbol_num;
    cmd
}

/// Sets the LoRa syncword. Valid for all spreading factors.
pub fn set_lora_syncword_cmd(syncword: u8) -> [u8; 3] {
    let mut cmd = [0u8; 3];
    cmd[0] = 0x02;
    cmd[1] = 0x2B;

    cmd[2] |= syncword;
    cmd
}

/// Returns information coded in last received packet header (explicit header mode) or configured coding_rate and crc_type settings
pub fn get_lora_rx_header_infos_req() -> [u8; 2] {
    [0x02, 0x30]
}

/// Sets the ranging ID for this slave device. Defines which address bytes are checked against master's request.
pub fn set_ranging_addr_cmd(addr: u32, check_length: CheckLength) -> [u8; 7] {
    let mut cmd = [0u8; 7];
    cmd[0] = 0x02;
    cmd[1] = 0x1C;

    cmd[2] |= ((addr >> 24) & 0xFF) as u8;
    cmd[3] |= ((addr >> 16) & 0xFF) as u8;
    cmd[4] |= ((addr >> 8) & 0xFF) as u8;
    cmd[5] |= (addr & 0xFF) as u8;
    cmd[6] |= (check_length as u8) & 0x7;
    cmd
}

/// Sets the address requested by the Master in the ranging request. Must match receiving Slave's ranging address.
pub fn set_ranging_req_addr_cmd(req_addr: u32) -> [u8; 6] {
    let mut cmd = [0u8; 6];
    cmd[0] = 0x02;
    cmd[1] = 0x1D;

    cmd[2] |= ((req_addr >> 24) & 0xFF) as u8;
    cmd[3] |= ((req_addr >> 16) & 0xFF) as u8;
    cmd[4] |= ((req_addr >> 8) & 0xFF) as u8;
    cmd[5] |= (req_addr & 0xFF) as u8;
    cmd
}

/// Reads ranging results in Master based on Type. Distance formula: Round Trip Distance (m) = Res * 3e8 / (2^12 * BW), where BW is LoRa bandwidth in Hz. RSSI formula: RSSI (dB) = Res / 2
pub fn get_ranging_result_req(ranging_res_kind: RangingResKind) -> [u8; 3] {
    let mut cmd = [0u8; 3];
    cmd[0] = 0x02;
    cmd[1] = 0x1E;

    cmd[2] |= ranging_res_kind as u8;
    cmd
}



/// Sets TxRx delay for ranging calibration. Compensates for deterministic fixed delay in processing ranging response/request for accurate range estimation. Same value must be written in both Master and Slave. Value depends on LoRa BW/SF used.
pub fn set_ranging_tx_rx_delay_cmd(delay: u32) -> [u8; 6] {
    let mut cmd = [0u8; 6];
    cmd[0] = 0x02;
    cmd[1] = 0x1F;

    cmd[2] |= ((delay >> 24) & 0xFF) as u8;
    cmd[3] |= ((delay >> 16) & 0xFF) as u8;
    cmd[4] |= ((delay >> 8) & 0xFF) as u8;
    cmd[5] |= (delay & 0xFF) as u8;
    cmd
}

/// Defines number of symbols used during synchronization. Value of 15 recommended for good compromise between accuracy and time on air/energy. Increasing symbols improves accuracy at expense of longer time on air.
pub fn set_ranging_parameter_cmd(reserved: u8, symb_nb: u8) -> [u8; 4] {
    let mut cmd = [0u8; 4];
    cmd[0] = 0x02;
    cmd[1] = 0x28;

    cmd[2] |= reserved;
    cmd[3] |= symb_nb;
    cmd
}

// Response structs

/// Response for GetLoraRxHeaderInfos command
#[derive(Default)]
pub struct LoraRxHeaderInfosRsp([u8; 2]);

impl LoraRxHeaderInfosRsp {
    /// Create a new response buffer
    pub fn new() -> Self {
        Self::default()
    }

    /// Return Status
    pub fn status(&mut self) -> Status {
        self.0[0].into()
    }

    /// CRC status from header (explicit mode) or configured setting (implicit mode). 1=CRC_ON, 0=CRC_OFF
    pub fn crc(&self) -> bool {
        (self.0[1] >> 4) & 0x1 != 0
    }

    /// Coding rate from header (explicit mode) or configured setting (implicit mode)
    pub fn lora_cr(&self) -> LoraCr {
        (self.0[1] & 0x7).into()
    }
}

impl AsMut<[u8]> for LoraRxHeaderInfosRsp {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

/// Response for GetRangingResult command
#[derive(Default)]
pub struct RangingResultRsp([u8; 4]);

impl RangingResultRsp {
    /// Create a new response buffer
    pub fn new() -> Self {
        Self::default()
    }

    /// Return Status
    pub fn status(&mut self) -> Status {
        self.0[0].into()
    }

    /// Distance in meter is given by rng*150/(2^12*Bandwidth)
    pub fn rng(&self) -> u32 {
        (self.0[3] as u32) |
        ((self.0[2] as u32) << 8) |
        ((self.0[1] as u32) << 16)
    }
}

impl AsMut<[u8]> for RangingResultRsp {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

/// Response for GetRangingRssi command
#[derive(Default)]
pub struct RangingRssiRsp([u8; 2]);

impl RangingRssiRsp {
    /// Create a new response buffer
    pub fn new() -> Self {
        Self::default()
    }

    /// Return Status
    pub fn status(&mut self) -> Status {
        self.0[0].into()
    }

    /// RSSI value in -0,5 dBm
    pub fn rssi(&self) -> u8 {
        self.0[1]
    }
}

impl AsMut<[u8]> for RangingRssiRsp {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}
