// Wifi commands API

use crate::status::Status;

/// 802.11 signal type: 0x01: Wi-Fi b, 0x02: Wi-Fi g, 0x03: Wi-Fi n, 0x04: All signals (b then g/n on same channel), 0x05-0xFF: RFU
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SignalType {
    WifiTypeB = 1,
    WifiTypeG = 2,
    WifiTypeN = 3,
    WifiTypeAll = 4,
}

/// Acquisition mode: 0x01: Beacon search, 0x02: Beacon and Packet search, 0x03: Full traffic, 0x04: Full beacon (until FCS), 0x05: SSID Beacon search (b/g only), other: RFU
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum AcqMode {
    BeaconSearch = 1,
    BeaconAndPacket = 2,
    FullTraffic = 3,
    FullBeacon = 4,
    SsidBeacon = 5,
}

/// Result format: 0x01: Basic Complete (22 or 79 bytes), 0x04: Basic MAC/Type/Channel (9 bytes), other: RFU
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Format {
    BasicComplete = 1,
    BasicMacTypeChannel = 4,
}

/// Captures Wi-Fi packets on RFIO_HF pin. BUSY signal high during scan (few hundred ms). IRQ signal high at end if WifiScanDone interrupt enabled.
pub fn wifi_scan_cmd(signal_type: SignalType, chan_mask: u16, acq_mode: AcqMode, nb_max_res: u8, nb_scan_per_chan: u8, timeout: u16, abort_on_timeout: u8) -> [u8; 11] {
    let mut cmd = [0u8; 11];
    cmd[0] = 0x03;
    cmd[1] = 0x00;

    cmd[2] |= signal_type as u8;
    cmd[3] |= ((chan_mask >> 8) & 0xFF) as u8;
    cmd[4] |= (chan_mask & 0xFF) as u8;
    cmd[5] |= acq_mode as u8;
    cmd[6] |= nb_max_res;
    cmd[7] |= nb_scan_per_chan;
    cmd[8] |= ((timeout >> 8) & 0xFF) as u8;
    cmd[9] |= (timeout & 0xFF) as u8;
    cmd[10] |= abort_on_timeout;
    cmd
}

/// Searches for Wi-Fi MAC addresses during configurable maximal time. Duration may be exceeded due to crystal drift and last signal detection. T_max = N_channel x ((1 + Xtal_precision) x Timeout + T_offset)
pub fn wifi_scan_time_limit_cmd(signal_type: SignalType, chan_mask: u16, acq_mode: AcqMode, nb_max_res: u8, scan_time_per_channel: u16, timeout_per_scan: u16) -> [u8; 11] {
    let mut cmd = [0u8; 11];
    cmd[0] = 0x03;
    cmd[1] = 0x01;

    cmd[2] |= signal_type as u8;
    cmd[3] |= ((chan_mask >> 8) & 0xFF) as u8;
    cmd[4] |= (chan_mask & 0xFF) as u8;
    cmd[5] |= acq_mode as u8;
    cmd[6] |= nb_max_res;
    cmd[7] |= ((scan_time_per_channel >> 8) & 0xFF) as u8;
    cmd[8] |= (scan_time_per_channel & 0xFF) as u8;
    cmd[9] |= ((timeout_per_scan >> 8) & 0xFF) as u8;
    cmd[10] |= (timeout_per_scan & 0xFF) as u8;
    cmd
}

/// Extracts Country code from Beacon or Probe Response. Only Wi-Fi b signals searched. Results filtered for duplicates by MAC address. Returns CMD_PERR if parameter range not respected, CMD_FAIL for radio config errors.
pub fn wifi_country_code_cmd(chan_mask: u16, nb_max_res: u8, nb_scan_per_channel: u8, timeout: u16, abort_on_timeout: u8) -> [u8; 9] {
    let mut cmd = [0u8; 9];
    cmd[0] = 0x03;
    cmd[1] = 0x02;

    cmd[2] |= ((chan_mask >> 8) & 0xFF) as u8;
    cmd[3] |= (chan_mask & 0xFF) as u8;
    cmd[4] |= nb_max_res;
    cmd[5] |= nb_scan_per_channel;
    cmd[6] |= ((timeout >> 8) & 0xFF) as u8;
    cmd[7] |= (timeout & 0xFF) as u8;
    cmd[8] |= abort_on_timeout;
    cmd
}

/// Searches for Wi-Fi MAC addresses during configurable maximal time for country code extraction. T_max = N_channel x ((1 + Xtal_precision) x Timeout + T_offset). T_offset always 9.59ms.
pub fn wifi_country_code_time_limit_cmd(chan_mask: u16, nb_max_res: u8, scan_time_per_channel: u16, timeout_per_scan: u16) -> [u8; 9] {
    let mut cmd = [0u8; 9];
    cmd[0] = 0x03;
    cmd[1] = 0x03;

    cmd[2] |= ((chan_mask >> 8) & 0xFF) as u8;
    cmd[3] |= (chan_mask & 0xFF) as u8;
    cmd[4] |= nb_max_res;
    cmd[5] |= ((scan_time_per_channel >> 8) & 0xFF) as u8;
    cmd[6] |= (scan_time_per_channel & 0xFF) as u8;
    cmd[7] |= ((timeout_per_scan >> 8) & 0xFF) as u8;
    cmd[8] |= (timeout_per_scan & 0xFF) as u8;
    cmd
}

/// Returns the number of Wi-Fi Scanning results (8 bits). Read at next SPI transaction.
pub fn wifi_get_nb_results_req() -> [u8; 2] {
    [0x03, 0x05]
}

/// Reads byte stream of Wi-Fi Passive Scanning results from given index in requested format. Must call WifiGetNbResults first. Issue NOP bytes (0x00) to read back. Max 1020 bytes per command - split into multiple requests if needed. Format 0x01: 22 bytes/MAC (modes 0x01, 0x02) or 79 bytes/MAC (mode 0x04). Format 0x04: 9 bytes/MAC.
pub fn wifi_read_results_req(index: u8, nb_results: u8, format: Format) -> [u8; 5] {
    let mut cmd = [0u8; 5];
    cmd[0] = 0x03;
    cmd[1] = 0x06;

    cmd[2] |= index;
    cmd[3] |= nb_results;
    cmd[4] |= format as u8;
    cmd
}

/// Resets Wi-Fi Passive Scanning cumulative timings. Must be called prior to executing Wi-Fi Passive Scanning if timings are to be read.
pub fn wifi_reset_cumul_timings_cmd() -> [u8; 2] {
    [0x03, 0x07]
}

/// Reads Wi-Fi Passive Scanning cumulative timings (16 bytes). Represents total time in various modes during WifiScan, summed for all acquisitions over different parameters. Times in microseconds. All 16 bytes must be read. Must be reset by host.
pub fn wifi_read_cumul_timings_req() -> [u8; 2] {
    [0x03, 0x08]
}

/// Returns number of results after Country Code scanning by WifiCountryCode or WifiCountryCodeTimeLimit
pub fn wifi_get_nb_country_code_results_req() -> [u8; 2] {
    [0x03, 0x09]
}

/// Reads byte stream of Wi-Fi Passive Scanning Country Code results from given index. Must call WifiGetNbCountryCodeResults first. Issue NOP bytes to read back. One result is 10 bytes.
pub fn wifi_read_country_code_results_req(index: u8, nb_results: u8) -> [u8; 4] {
    let mut cmd = [0u8; 4];
    cmd[0] = 0x03;
    cmd[1] = 0x0A;

    cmd[2] |= index;
    cmd[3] |= nb_results;
    cmd
}

/// Configures timestamp threshold to discriminate mobile access point from gateways. Default 1 day. If timestamp from beacon/probe response exceeds limit, MAC validation indicates probable gateway not mobile device.
pub fn wifi_cfg_timestamp_a_pphone_cmd(timestamp: u32) -> [u8; 6] {
    let mut cmd = [0u8; 6];
    cmd[0] = 0x03;
    cmd[1] = 0x0B;

    cmd[2] |= ((timestamp >> 24) & 0xFF) as u8;
    cmd[3] |= ((timestamp >> 16) & 0xFF) as u8;
    cmd[4] |= ((timestamp >> 8) & 0xFF) as u8;
    cmd[5] |= (timestamp & 0xFF) as u8;
    cmd
}

/// Returns internal Wi-Fi firmware version major and minor numbers
pub fn wifi_read_version_req() -> [u8; 2] {
    [0x03, 0x20]
}

// Response structs

/// Response for WifiGetNbResults command
#[derive(Default)]
pub struct WifiGetNbResultsRsp([u8; 2]);

impl WifiGetNbResultsRsp {
    /// Create a new response buffer
    pub fn new() -> Self {
        Self::default()
    }

    /// Return Status
    pub fn status(&mut self) -> Status {
        self.0[0].into()
    }

    /// Number of Wi-Fi scan results (0-32)
    pub fn nb_results(&self) -> u8 {
        self.0[1]
    }
}

impl AsMut<[u8]> for WifiGetNbResultsRsp {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

/// Response for WifiReadResults command
#[derive(Default)]
pub struct WifiReadResultsRsp([u8; 2]);

impl WifiReadResultsRsp {
    /// Create a new response buffer
    pub fn new() -> Self {
        Self::default()
    }

    /// Return Status
    pub fn status(&mut self) -> Status {
        self.0[0].into()
    }
    // TODO: Implement accessor for variable length field 'results'
}

impl AsMut<[u8]> for WifiReadResultsRsp {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

/// Response for WifiReadCumulTimings command
#[derive(Default)]
pub struct WifiReadCumulTimingsRsp([u8; 17]);

impl WifiReadCumulTimingsRsp {
    /// Create a new response buffer
    pub fn new() -> Self {
        Self::default()
    }

    /// Return Status
    pub fn status(&mut self) -> Status {
        self.0[0].into()
    }

    /// Reserved for future use
    pub fn rfu(&self) -> u32 {
        (self.0[4] as u32) |
        ((self.0[3] as u32) << 8) |
        ((self.0[2] as u32) << 16) |
        ((self.0[1] as u32) << 24)
    }

    /// Total duration in preamble detection mode (microseconds)
    pub fn preamble_detection_time(&self) -> u32 {
        (self.0[8] as u32) |
        ((self.0[7] as u32) << 8) |
        ((self.0[6] as u32) << 16) |
        ((self.0[5] as u32) << 24)
    }

    /// Total duration in capture mode (microseconds)
    pub fn capture_time(&self) -> u32 {
        (self.0[12] as u32) |
        ((self.0[11] as u32) << 8) |
        ((self.0[10] as u32) << 16) |
        ((self.0[9] as u32) << 24)
    }

    /// Total duration in demodulation mode (microseconds)
    pub fn demodulation_time(&self) -> u32 {
        (self.0[16] as u32) |
        ((self.0[15] as u32) << 8) |
        ((self.0[14] as u32) << 16) |
        ((self.0[13] as u32) << 24)
    }
}

impl AsMut<[u8]> for WifiReadCumulTimingsRsp {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

/// Response for WifiGetNbCountryCodeResults command
#[derive(Default)]
pub struct WifiGetNbCountryCodeResultsRsp([u8; 2]);

impl WifiGetNbCountryCodeResultsRsp {
    /// Create a new response buffer
    pub fn new() -> Self {
        Self::default()
    }

    /// Return Status
    pub fn status(&mut self) -> Status {
        self.0[0].into()
    }

    /// Number of Country Code results (0-32)
    pub fn nb_results(&self) -> u8 {
        self.0[1]
    }
}

impl AsMut<[u8]> for WifiGetNbCountryCodeResultsRsp {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

/// Response for WifiReadCountryCodeResults command
#[derive(Default)]
pub struct WifiReadCountryCodeResultsRsp([u8; 2]);

impl WifiReadCountryCodeResultsRsp {
    /// Create a new response buffer
    pub fn new() -> Self {
        Self::default()
    }

    /// Return Status
    pub fn status(&mut self) -> Status {
        self.0[0].into()
    }
    // TODO: Implement accessor for variable length field 'results'
}

impl AsMut<[u8]> for WifiReadCountryCodeResultsRsp {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

/// Response for WifiReadVersion command
#[derive(Default)]
pub struct WifiReadVersionRsp([u8; 3]);

impl WifiReadVersionRsp {
    /// Create a new response buffer
    pub fn new() -> Self {
        Self::default()
    }

    /// Return Status
    pub fn status(&mut self) -> Status {
        self.0[0].into()
    }

    /// Wi-Fi firmware major version
    pub fn version_major(&self) -> u8 {
        self.0[1]
    }

    /// Wi-Fi firmware minor version
    pub fn version_minor(&self) -> u8 {
        self.0[2]
    }
}

impl AsMut<[u8]> for WifiReadVersionRsp {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}
