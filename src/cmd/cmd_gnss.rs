// Gnss commands API

use crate::status::Status;

/// 0x00: Legacy (single) scanning, 0x03: Advanced (multiple) scanning, other: RFU
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum GnssMode {
    LegacySingle = 0,
    AdvancedMultiple = 3,
}

/// 0x00: Low Power (stop if no strong satellite), other: RFU
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum EffortMode {
    LowPower = 0,
    BestEffort = 1,
}

/// Demodulation sensitivity: 0: Low (GPS -134dBm, BeiDou -130dBm), 1: Mid (GPS -137dBm, BeiDou -134dBm)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum DemodSensi {
    Low = 0,
    Mid = 1,
}

/// Configures GNSS scanning for selected constellation (GPS/BeiDou). If both selected, GPS scans first, then BeiDou after delay (4s fixed for FW ≤01.02, variable 1s steps for FW 02.01+). Requires 32.768kHz clock for dual constellation. BUSY high until both scans complete.
pub fn gnss_set_constellation_to_use_cmd(constellation_bit_mask: u8) -> [u8; 3] {
    let mut cmd = [0u8; 3];
    cmd[0] = 0x04;
    cmd[1] = 0x00;

    cmd[2] |= constellation_bit_mask;
    cmd
}

/// Reads selected constellation (GPS/BeiDou)
pub fn gnss_read_constellation_to_use_req() -> [u8; 2] {
    [0x04, 0x01]
}

/// Reads supported constellations
pub fn gnss_read_supported_constellations_req() -> [u8; 2] {
    [0x04, 0x07]
}

/// Configures GNSS for Legacy (single) or Advanced (multiple) scanning. Advanced performs multiple captures and averages them for increased precision, at expense of longer duration and higher energy. FW 02.01+ NAV message format differs from earlier versions.
pub fn gnss_set_mode_cmd(gnss_mode: GnssMode) -> [u8; 3] {
    let mut cmd = [0u8; 3];
    cmd[0] = 0x04;
    cmd[1] = 0x08;

    cmd[2] |= gnss_mode as u8;
    cmd
}

/// Captures GNSS signals in autonomous mode (no assistance info available). NOT supported in FW 02.01+, replaced by GnssScan. Resets previous GNSS results. BUSY high during scan, GNSSDone IRQ when complete.
pub fn gnss_autonomous_cmd(time: u32, effort_mode: EffortMode, result_mask: u8, nb_sv_max: u8) -> [u8; 9] {
    let mut cmd = [0u8; 9];
    cmd[0] = 0x04;
    cmd[1] = 0x09;

    cmd[2] |= ((time >> 24) & 0xFF) as u8;
    cmd[3] |= ((time >> 16) & 0xFF) as u8;
    cmd[4] |= ((time >> 8) & 0xFF) as u8;
    cmd[5] |= (time & 0xFF) as u8;
    cmd[6] |= effort_mode as u8;
    cmd[7] |= result_mask;
    cmd[8] |= nb_sv_max;
    cmd
}

/// Captures GNSS signals using assistance data (time, position, almanac). NOT supported in FW 02.01+, replaced by GnssScan. Resets previous GNSS results. BUSY high during scan, GNSSDone IRQ when complete.
pub fn gnss_assisted_cmd(time: u32, effort_mode: EffortMode, result_mask: u8, nb_sv_max: u8) -> [u8; 9] {
    let mut cmd = [0u8; 9];
    cmd[0] = 0x04;
    cmd[1] = 0x0A;

    cmd[2] |= ((time >> 24) & 0xFF) as u8;
    cmd[3] |= ((time >> 16) & 0xFF) as u8;
    cmd[4] |= ((time >> 8) & 0xFF) as u8;
    cmd[5] |= (time & 0xFF) as u8;
    cmd[6] |= effort_mode as u8;
    cmd[7] |= result_mask;
    cmd[8] |= nb_sv_max;
    cmd
}

/// Captures GNSS signals independent of assistance data availability. FW 02.01+ only. Two types: Cold start (no time/position, determines via demod/2D solving) or Assisted (time+position known, searches 12 strongest visible satellites). Use sleep with retention to preserve assistance data. Can be aborted by sending 0x0 on SPI while BUSY active (max 2.9s abort delay).
pub fn gnss_scan_cmd(effort_mode: EffortMode, result_mask: u8, nb_sv_max: u8) -> [u8; 5] {
    let mut cmd = [0u8; 5];
    cmd[0] = 0x04;
    cmd[1] = 0x0B;

    cmd[2] |= effort_mode as u8;
    cmd[3] |= result_mask;
    cmd[4] |= nb_sv_max;
    cmd
}

/// Configures approximate position for GNSS assisted mode. FW 02.01+ uses integrated 2D solving to determine Assistance Position, replacing this configured value.
pub fn gnss_set_assistance_position_cmd(latitude: u16, longitude: u16) -> [u8; 6] {
    let mut cmd = [0u8; 6];
    cmd[0] = 0x04;
    cmd[1] = 0x10;

    cmd[2] |= ((latitude >> 8) & 0xFF) as u8;
    cmd[3] |= (latitude & 0xFF) as u8;
    cmd[4] |= ((longitude >> 8) & 0xFF) as u8;
    cmd[5] |= (longitude & 0xFF) as u8;
    cmd
}

/// Reads assistance position
pub fn gnss_read_assistance_position_req() -> [u8; 2] {
    [0x04, 0x11]
}

/// Reads GNSS context status including firmware version, almanac CRC, error codes, and frequency search space
pub fn gnss_get_context_status_req() -> [u8; 2] {
    [0x04, 0x16]
}

/// Returns internal GNSS firmware version and almanac version
pub fn gnss_read_version_req() -> [u8; 2] {
    [0x04, 0x06]
}

/// Configures constellation almanac information to be updated. By default both constellations activated.
pub fn gnss_set_almanac_update_cmd(almanac_update_bit_mask: u8) -> [u8; 3] {
    let mut cmd = [0u8; 3];
    cmd[0] = 0x04;
    cmd[1] = 0x02;

    cmd[2] |= almanac_update_bit_mask;
    cmd
}

/// Reads almanac update information
pub fn gnss_read_almanac_update_req() -> [u8; 2] {
    [0x04, 0x03]
}

/// Reads size in bytes of byte stream containing available GNSS results. Must call before GnssReadResults.
pub fn gnss_get_result_size_req() -> [u8; 2] {
    [0x04, 0x0C]
}

/// Retrieves last GNSS results. Variable length message: DestinationID (1 byte) + Payload. DestinationID: 0x00=Status to Host, 0x01=NAV to GNSS Solver, 0x02=Almanac update to DM service. Must call GnssGetResultSize first.
pub fn gnss_read_results_req() -> [u8; 2] {
    [0x04, 0x0D]
}

/// Returns number of Satellite Vehicles detected during last GNSS scan
pub fn gnss_get_nb_sv_detected_req() -> [u8; 2] {
    [0x04, 0x17]
}

/// Returns ID, SNR and Doppler of Satellite Vehicles detected during last GNSS scan. SNR in dB, add 31dB to convert to C/N0.
pub fn gnss_get_sv_detected_req() -> [u8; 2] {
    [0x04, 0x18]
}

/// Reads duration of Radio capture and CPU processing phases of GNSS scan in microseconds. Used to determine GNSS power consumption.
pub fn gnss_get_consumption_req() -> [u8; 2] {
    [0x04, 0x19]
}

/// Returns number of visible satellites for given time, position, and constellation
pub fn gnss_get_sv_visible_req(time: u32, latitude: u16, longitude: u16, constellation: u8) -> [u8; 11] {
    let mut cmd = [0u8; 11];
    cmd[0] = 0x04;
    cmd[1] = 0x1F;

    cmd[2] |= ((time >> 24) & 0xFF) as u8;
    cmd[3] |= ((time >> 16) & 0xFF) as u8;
    cmd[4] |= ((time >> 8) & 0xFF) as u8;
    cmd[5] |= (time & 0xFF) as u8;
    cmd[6] |= ((latitude >> 8) & 0xFF) as u8;
    cmd[7] |= (latitude & 0xFF) as u8;
    cmd[8] |= ((longitude >> 8) & 0xFF) as u8;
    cmd[9] |= (longitude & 0xFF) as u8;
    cmd[10] |= constellation;
    cmd
}

/// Configures delay after which LR1120 resets Assistance Position and switches from assisted to cold start scan (if time elapsed since last AP update exceeds delay AND no SV detected). FW 02.01+ only.
pub fn gnss_config_delay_reset_ap_cmd(delay: u32) -> [u8; 5] {
    let mut cmd = [0u8; 5];
    cmd[0] = 0x04;
    cmd[1] = 0x65;

    cmd[2] |= ((delay >> 16) & 0xFF) as u8;
    cmd[3] |= ((delay >> 8) & 0xFF) as u8;
    cmd[4] |= (delay & 0xFF) as u8;
    cmd
}

/// Returns type of scan launched during last scan. FW 02.01+ only.
pub fn gnss_read_last_scan_mode_launched_req() -> [u8; 2] {
    [0x04, 0x26]
}

/// Configures LR1120 to determine time by demodulating satellite signals. FW 02.01+ only. Can be aborted.
pub fn gnss_fetch_time_cmd(demod_sensi: DemodSensi, options: u8) -> [u8; 4] {
    let mut cmd = [0u8; 4];
    cmd[0] = 0x04;
    cmd[1] = 0x32;

    cmd[2] |= demod_sensi as u8;
    cmd[3] |= options;
    cmd
}

/// Returns GPS time. FW 02.01+ only.
pub fn gnss_read_time_req() -> [u8; 2] {
    [0x04, 0x34]
}

/// Resets GPS time. FW 02.01+ only.
pub fn gnss_reset_time_cmd() -> [u8; 2] {
    [0x04, 0x35]
}

/// Resets Assistance Position. FW 02.01+ only.
pub fn gnss_reset_position_cmd() -> [u8; 2] {
    [0x04, 0x37]
}

/// Allows Host MCU to set LR1120 GPS time. FW 02.01+ only.
pub fn gnss_set_time_cmd(gps_time: u32, time_accuracy: u16) -> [u8; 8] {
    let mut cmd = [0u8; 8];
    cmd[0] = 0x04;
    cmd[1] = 0x4B;

    cmd[2] |= ((gps_time >> 24) & 0xFF) as u8;
    cmd[3] |= ((gps_time >> 16) & 0xFF) as u8;
    cmd[4] |= ((gps_time >> 8) & 0xFF) as u8;
    cmd[5] |= (gps_time & 0xFF) as u8;
    cmd[6] |= ((time_accuracy >> 8) & 0xFF) as u8;
    cmd[7] |= (time_accuracy & 0xFF) as u8;
    cmd
}

/// Reads Assistance Position calculated by LR1120 2D Solver during GnssScan/GnssAlmanacUpdateFromSat or updated by GnssComputeAssistancePosition. All 18 bytes must be read. FW 02.01+ only.
pub fn gnss_read_doppler_solver_res_req() -> [u8; 2] {
    [0x04, 0x4F]
}

/// Reads delay before Assistance Position reset configured in GnssConfigDelayResetAP. FW 02.01+ only.
pub fn gnss_read_delay_reset_ap_req() -> [u8; 2] {
    [0x04, 0x53]
}

// Response structs

/// Response for GnssReadConstellationToUse command
#[derive(Default)]
pub struct GnssReadConstellationToUseRsp([u8; 2]);

impl GnssReadConstellationToUseRsp {
    /// Create a new response buffer
    pub fn new() -> Self {
        Self::default()
    }

    /// Return Status
    pub fn status(&mut self) -> Status {
        self.0[0].into()
    }

    /// Bit 0=1: GPS, Bit 1=1: BeiDou, other: RFU
    pub fn constellation_bit_mask(&self) -> u8 {
        self.0[1]
    }
}

impl AsMut<[u8]> for GnssReadConstellationToUseRsp {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

/// Response for GnssReadSupportedConstellations command
#[derive(Default)]
pub struct GnssReadSupportedConstellationsRsp([u8; 2]);

impl GnssReadSupportedConstellationsRsp {
    /// Create a new response buffer
    pub fn new() -> Self {
        Self::default()
    }

    /// Return Status
    pub fn status(&mut self) -> Status {
        self.0[0].into()
    }

    /// Bit 0=1: GPS supported, Bit 1=1: BeiDou supported, other: RFU
    pub fn constellation_bit_mask(&self) -> u8 {
        self.0[1]
    }
}

impl AsMut<[u8]> for GnssReadSupportedConstellationsRsp {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

/// Response for GnssReadAssistancePosition command
#[derive(Default)]
pub struct GnssReadAssistancePositionRsp([u8; 5]);

impl GnssReadAssistancePositionRsp {
    /// Create a new response buffer
    pub fn new() -> Self {
        Self::default()
    }

    /// Return Status
    pub fn status(&mut self) -> Status {
        self.0[0].into()
    }

    /// Latitude (12-bit effective). Convert: latitude_degrees = Latitude * 90/2048
    pub fn latitude(&self) -> u16 {
        (self.0[2] as u16) |
        ((self.0[1] as u16) << 8)
    }

    /// Longitude (12-bit effective). Convert: longitude_degrees = Longitude * 180/2048
    pub fn longitude(&self) -> u16 {
        (self.0[4] as u16) |
        ((self.0[3] as u16) << 8)
    }
}

impl AsMut<[u8]> for GnssReadAssistancePositionRsp {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

/// Response for GnssGetContextStatus command
#[derive(Default)]
pub struct GnssGetContextStatusRsp([u8; 10]);

impl GnssGetContextStatusRsp {
    /// Create a new response buffer
    pub fn new() -> Self {
        Self::default()
    }

    /// Return Status
    pub fn status(&mut self) -> Status {
        self.0[0].into()
    }

    /// GNSS firmware version
    pub fn gnss_fw_version(&self) -> u8 {
        self.0[3]
    }

    /// 32-bit CRC on all flash memory (128 satellites, 21 bytes each)
    pub fn global_almanac_crc(&self) -> u32 {
        (self.0[7] as u32) |
        ((self.0[6] as u32) << 8) |
        ((self.0[5] as u32) << 16) |
        ((self.0[4] as u32) << 24)
    }

    /// 0: No error, 1: Almanac too old, 2: Last almanac update CRC mismatch, 3: Flash integrity error, 4: Almanac update not allowed (GPS and BeiDou can't update in same request), 5-15: RFU
    pub fn error_code(&self) -> u8 {
        (self.0[8] >> 4) & 0xF
    }

    /// Bit 0: GPS, Bit 1: BeiDou, Bit 2: RFU
    pub fn almanac_update_bit_mask(&self) -> u8 {
        (self.0[8] >> 1) & 0x7
    }

    /// 0: 250Hz, 1: 500Hz, 2: 1kHz, 3: 2kHz
    pub fn freq_search_space(&self) -> u8 {
        (self.0[9] >> 7) |
        ((self.0[8] & 0x1) << 1)
    }
}

impl AsMut<[u8]> for GnssGetContextStatusRsp {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

/// Response for GnssReadVersion command
#[derive(Default)]
pub struct GnssReadVersionRsp([u8; 3]);

impl GnssReadVersionRsp {
    /// Create a new response buffer
    pub fn new() -> Self {
        Self::default()
    }

    /// Return Status
    pub fn status(&mut self) -> Status {
        self.0[0].into()
    }

    /// GNSS firmware version
    pub fn firmware_version(&self) -> u8 {
        self.0[1]
    }

    /// Almanac version
    pub fn almanac_version(&self) -> u8 {
        self.0[2]
    }
}

impl AsMut<[u8]> for GnssReadVersionRsp {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

/// Response for GnssReadAlmanacUpdate command
#[derive(Default)]
pub struct GnssReadAlmanacUpdateRsp([u8; 2]);

impl GnssReadAlmanacUpdateRsp {
    /// Create a new response buffer
    pub fn new() -> Self {
        Self::default()
    }

    /// Return Status
    pub fn status(&mut self) -> Status {
        self.0[0].into()
    }

    /// Bit 0: GPS, Bit 1: BeiDou, Bit 2: RFU
    pub fn almanac_update_bit_mask(&self) -> u8 {
        self.0[1]
    }
}

impl AsMut<[u8]> for GnssReadAlmanacUpdateRsp {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

/// Response for GnssGetResultSize command
#[derive(Default)]
pub struct GnssGetResultSizeRsp([u8; 3]);

impl GnssGetResultSizeRsp {
    /// Create a new response buffer
    pub fn new() -> Self {
        Self::default()
    }

    /// Return Status
    pub fn status(&mut self) -> Status {
        self.0[0].into()
    }

    /// Size of GNSS results in bytes
    pub fn result_size(&self) -> u16 {
        (self.0[2] as u16) |
        ((self.0[1] as u16) << 8)
    }
}

impl AsMut<[u8]> for GnssGetResultSizeRsp {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

/// Response for GnssReadResults command
#[derive(Default)]
pub struct GnssReadResultsRsp([u8; 2]);

impl GnssReadResultsRsp {
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

impl AsMut<[u8]> for GnssReadResultsRsp {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

/// Response for GnssGetNbSvDetected command
#[derive(Default)]
pub struct GnssGetNbSvDetectedRsp([u8; 2]);

impl GnssGetNbSvDetectedRsp {
    /// Create a new response buffer
    pub fn new() -> Self {
        Self::default()
    }

    /// Return Status
    pub fn status(&mut self) -> Status {
        self.0[0].into()
    }

    /// Number of detected satellites
    pub fn nb_sv(&self) -> u8 {
        self.0[1]
    }
}

impl AsMut<[u8]> for GnssGetNbSvDetectedRsp {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

/// Response for GnssGetSvDetected command
#[derive(Default)]
pub struct GnssGetSvDetectedRsp([u8; 2]);

impl GnssGetSvDetectedRsp {
    /// Create a new response buffer
    pub fn new() -> Self {
        Self::default()
    }

    /// Return Status
    pub fn status(&mut self) -> Status {
        self.0[0].into()
    }
    // TODO: Implement accessor for variable length field 'sv_data'
}

impl AsMut<[u8]> for GnssGetSvDetectedRsp {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

/// Response for GnssGetConsumption command
#[derive(Default)]
pub struct GnssGetConsumptionRsp([u8; 9]);

impl GnssGetConsumptionRsp {
    /// Create a new response buffer
    pub fn new() -> Self {
        Self::default()
    }

    /// Return Status
    pub fn status(&mut self) -> Status {
        self.0[0].into()
    }

    /// Radio capture duration in microseconds
    pub fn radio_ms(&self) -> u32 {
        (self.0[4] as u32) |
        ((self.0[3] as u32) << 8) |
        ((self.0[2] as u32) << 16) |
        ((self.0[1] as u32) << 24)
    }

    /// CPU processing duration in microseconds
    pub fn computation_ms(&self) -> u32 {
        (self.0[8] as u32) |
        ((self.0[7] as u32) << 8) |
        ((self.0[6] as u32) << 16) |
        ((self.0[5] as u32) << 24)
    }
}

impl AsMut<[u8]> for GnssGetConsumptionRsp {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

/// Response for GnssGetSvVisible command
#[derive(Default)]
pub struct GnssGetSvVisibleRsp([u8; 2]);

impl GnssGetSvVisibleRsp {
    /// Create a new response buffer
    pub fn new() -> Self {
        Self::default()
    }

    /// Return Status
    pub fn status(&mut self) -> Status {
        self.0[0].into()
    }

    /// Number of visible satellites
    pub fn nb_sv_visible(&self) -> u8 {
        self.0[1]
    }
}

impl AsMut<[u8]> for GnssGetSvVisibleRsp {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

/// Response for GnssReadLastScanModeLaunched command
#[derive(Default)]
pub struct GnssReadLastScanModeLaunchedRsp([u8; 2]);

impl GnssReadLastScanModeLaunchedRsp {
    /// Create a new response buffer
    pub fn new() -> Self {
        Self::default()
    }

    /// Return Status
    pub fn status(&mut self) -> Status {
        self.0[0].into()
    }

    /// 0-2: RFU, 3: Assisted (time+position known), 4: Cold start (unknown), 5: Cold start (time known), 6: Fetch time/integrated 2D, 7: Almanac update no flash, 8: Keep sync, 9: Almanac update 1 constellation flashed, 10: Almanac update 2 constellations flashed
    pub fn last_scan_mode(&self) -> u8 {
        self.0[1]
    }
}

impl AsMut<[u8]> for GnssReadLastScanModeLaunchedRsp {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

/// Response for GnssReadTime command
#[derive(Default)]
pub struct GnssReadTimeRsp([u8; 9]);

impl GnssReadTimeRsp {
    /// Create a new response buffer
    pub fn new() -> Self {
        Self::default()
    }

    /// Return Status
    pub fn status(&mut self) -> Status {
        self.0[0].into()
    }

    /// GPS time in seconds since 6 Jan 1980 00:00:00
    pub fn gps_time(&self) -> u32 {
        (self.0[4] as u32) |
        ((self.0[3] as u32) << 8) |
        ((self.0[2] as u32) << 16) |
        ((self.0[1] as u32) << 24)
    }

    /// Time accuracy in milliseconds
    pub fn time_accuracy(&self) -> u32 {
        (self.0[8] as u32) |
        ((self.0[7] as u32) << 8) |
        ((self.0[6] as u32) << 16) |
        ((self.0[5] as u32) << 24)
    }
}

impl AsMut<[u8]> for GnssReadTimeRsp {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

/// Response for GnssReadDopplerSolverRes command
#[derive(Default)]
pub struct GnssReadDopplerSolverResRsp([u8; 19]);

impl GnssReadDopplerSolverResRsp {
    /// Create a new response buffer
    pub fn new() -> Self {
        Self::default()
    }

    /// Return Status
    pub fn status(&mut self) -> Status {
        self.0[0].into()
    }

    /// 0: NoError, 1: ResidueHigh, 2: NotConverged, 3: NotEnoughSv, 4: IllMatrix, 5: Time error, 6: PartialAlmanacTooOld, 7: NotConsistentWithHistory, 8: AllAlmanacTooOld. If not NoError, latitude=0xFFFF, longitude=0xFFFF
    pub fn error_code(&self) -> u8 {
        self.0[1]
    }

    /// Number of dopplers used for oneshot solving
    pub fn nb_sv_used(&self) -> u8 {
        self.0[2]
    }

    /// Latitude from 2D solver
    pub fn latitude(&self) -> u16 {
        (self.0[4] as u16) |
        ((self.0[3] as u16) << 8)
    }

    /// Longitude from 2D solver
    pub fn longitude(&self) -> u16 {
        (self.0[6] as u16) |
        ((self.0[5] as u16) << 8)
    }

    /// Accuracy estimate
    pub fn accuracy(&self) -> u16 {
        (self.0[8] as u16) |
        ((self.0[7] as u16) << 8)
    }

    /// Crystal frequency offset
    pub fn xtal(&self) -> u16 {
        (self.0[10] as u16) |
        ((self.0[9] as u16) << 8)
    }

    /// Filtered latitude
    pub fn filtered_latitude(&self) -> u16 {
        (self.0[12] as u16) |
        ((self.0[11] as u16) << 8)
    }

    /// Filtered longitude
    pub fn filtered_longitude(&self) -> u16 {
        (self.0[14] as u16) |
        ((self.0[13] as u16) << 8)
    }

    /// Filtered accuracy
    pub fn filtered_accuracy(&self) -> u16 {
        (self.0[16] as u16) |
        ((self.0[15] as u16) << 8)
    }

    /// Filtered crystal offset
    pub fn filtered_xtal(&self) -> u16 {
        (self.0[18] as u16) |
        ((self.0[17] as u16) << 8)
    }
}

impl AsMut<[u8]> for GnssReadDopplerSolverResRsp {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

/// Response for GnssReadDelayResetAP command
#[derive(Default)]
pub struct GnssReadDelayResetAPRsp([u8; 4]);

impl GnssReadDelayResetAPRsp {
    /// Create a new response buffer
    pub fn new() -> Self {
        Self::default()
    }

    /// Return Status
    pub fn status(&mut self) -> Status {
        self.0[0].into()
    }

    /// Delay in seconds
    pub fn delay(&self) -> u32 {
        (self.0[3] as u32) |
        ((self.0[2] as u32) << 8) |
        ((self.0[1] as u32) << 16)
    }
}

impl AsMut<[u8]> for GnssReadDelayResetAPRsp {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

// Commands with variable length parameters (not implemented):
// - GnssPushSolverMsg
// - GnssPushDmMsg
