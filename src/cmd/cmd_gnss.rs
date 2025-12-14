// Gnss commands API

use crate::status::Status;

/// 0x00: Legacy (single) scanning, 0x03: Advanced (multiple) scanning, other: RFU
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum GnssScanMode {
    Single = 0,
    Multi = 3,
}

/// Source of error, if any
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ContextError {
    None = 0,
    AlmanacOld = 1,
    AlmanacCrc = 2,
    FlashIntegrity = 3,
    AlmanacLocked = 4,
}

impl From<u8> for ContextError {
    fn from(value: u8) -> Self {
        match value {
            4 => ContextError::AlmanacLocked,
            3 => ContextError::FlashIntegrity,
            2 => ContextError::AlmanacCrc,
            1 => ContextError::AlmanacOld,
            _ => ContextError::None,
        }
    }
}

/// Frequency search space in Heartz
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum FreqSearchSpace {
    F250 = 0,
    F500 = 1,
    F1000 = 2,
    F2000 = 3,
}

impl From<u8> for FreqSearchSpace {
    fn from(value: u8) -> Self {
        match value {
            3 => FreqSearchSpace::F2000,
            2 => FreqSearchSpace::F1000,
            1 => FreqSearchSpace::F500,
            _ => FreqSearchSpace::F250,
        }
    }
}

/// 0-2: RFU, 3: Assisted (time+position known), 4: Cold start (unknown), 5: Cold start (time known), 6: Fetch time/integrated 2D, 7: Almanac update no flash, 8: Keep sync, 9: Almanac update 1 constellation flashed, 10: Almanac update 2 constellations flashed
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum GnssScanType {
    Assisted = 3,
    ColdStart = 4,
    TimeKnown = 5,
    FetchTime = 6,
    AlmanacUpdt0 = 7,
    KeepSync = 8,
    AlmanacUpdt1 = 9,
    AlmanacUpdt2 = 10,
}

impl From<u8> for GnssScanType {
    fn from(value: u8) -> Self {
        match value {
            10 => GnssScanType::AlmanacUpdt2,
            9 => GnssScanType::AlmanacUpdt1,
            8 => GnssScanType::KeepSync,
            7 => GnssScanType::AlmanacUpdt0,
            6 => GnssScanType::FetchTime,
            5 => GnssScanType::TimeKnown,
            4 => GnssScanType::ColdStart,
            _ => GnssScanType::Assisted,
        }
    }
}

/// TOW_ONLY should only be used once the Week Number is known. ROLLOVER is for beidou constellation only.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum FetchTimeMode {
    TowOnly = 0,
    TowWn = 1,
    Rollover = 2,
}

/// Failure reason
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum SolverError {
    None = 0,
    ResidueHigh = 1,
    NotConverged = 2,
    NotEnoughSv = 3,
    IllegalMatrix = 4,
    TimeError = 5,
    PartialAlamanacOld = 6,
    Inconsistent = 7,
    FullAlamanacOld = 8,
}

impl From<u8> for SolverError {
    fn from(value: u8) -> Self {
        match value {
            8 => SolverError::FullAlamanacOld,
            7 => SolverError::Inconsistent,
            6 => SolverError::PartialAlamanacOld,
            5 => SolverError::TimeError,
            4 => SolverError::IllegalMatrix,
            3 => SolverError::NotEnoughSv,
            2 => SolverError::NotConverged,
            1 => SolverError::ResidueHigh,
            _ => SolverError::None,
        }
    }
}

/// Source of week number. Can be not set, set by GnssScan, or from GnssSetTime/GnssFetchTime.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum WnSource {
    NotSet = 0,
    Scan = 1,
    Time = 2,
}

impl From<u8> for WnSource {
    fn from(value: u8) -> Self {
        match value {
            2 => WnSource::Time,
            1 => WnSource::Scan,
            _ => WnSource::NotSet,
        }
    }
}

/// GPS almanac status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum AlmanacStatus {
    LowAccuracy = 252,
    NoTimeSet = 253,
    NextTimeUnknown = 254,
    PageIdUnknown = 255,
    NothingTodo = 0,
    Success = 1,
}

impl From<u8> for AlmanacStatus {
    fn from(value: u8) -> Self {
        match value {
            1 => AlmanacStatus::Success,
            0 => AlmanacStatus::NothingTodo,
            255 => AlmanacStatus::PageIdUnknown,
            254 => AlmanacStatus::NextTimeUnknown,
            253 => AlmanacStatus::NoTimeSet,
            _ => AlmanacStatus::LowAccuracy,
        }
    }
}

/// 0: MEO satellite, 1: IGSO satellite. No impact on GPS but must be ≤1
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum BeidouType {
    Meo = 0,
    Igso = 1,
}

/// Configures GNSS scanning for selected constellation (GPS/BeiDou). If both selected, GPS scans first, then BeiDou after delay (4s fixed for FW ≤01.02, variable 1s steps for FW 02.01+). Requires 32.768kHz clock for dual constellation. BUSY high until both scans complete.
pub fn gnss_set_constellation_to_use_cmd(gps_en: bool, beidou_en: bool) -> [u8; 3] {
    let mut cmd = [0u8; 3];
    cmd[0] = 0x04;
    cmd[1] = 0x00;

    if gps_en { cmd[2] |= 1; }
    if beidou_en { cmd[2] |= 2; }
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
pub fn gnss_set_mode_cmd(gnss_scan_mode: GnssScanMode) -> [u8; 3] {
    let mut cmd = [0u8; 3];
    cmd[0] = 0x04;
    cmd[1] = 0x08;

    cmd[2] |= gnss_scan_mode as u8;
    cmd
}

/// Captures GNSS signals in autonomous mode (no assistance info available). NOT supported in FW 02.01+, replaced by GnssScan. Resets previous GNSS results. BUSY high during scan, GNSSDone IRQ when complete.
pub fn gnss_autonomous_cmd(time: u32, best_effort: bool, pseudo_range_en: bool, doppler_info_en: bool, bit_changes_en: bool, nb_sv_max: u8) -> [u8; 9] {
    let mut cmd = [0u8; 9];
    cmd[0] = 0x04;
    cmd[1] = 0x09;

    cmd[2] |= ((time >> 24) & 0xFF) as u8;
    cmd[3] |= ((time >> 16) & 0xFF) as u8;
    cmd[4] |= ((time >> 8) & 0xFF) as u8;
    cmd[5] |= (time & 0xFF) as u8;
    if best_effort { cmd[6] |= 1; }
    if pseudo_range_en { cmd[7] |= 1; }
    if doppler_info_en { cmd[7] |= 2; }
    if bit_changes_en { cmd[7] |= 4; }
    cmd[8] |= nb_sv_max;
    cmd
}

/// Captures GNSS signals using assistance data (time, position, almanac). NOT supported in FW 02.01+, replaced by GnssScan. Resets previous GNSS results. BUSY high during scan, GNSSDone IRQ when complete.
pub fn gnss_assisted_cmd(time: u32, best_effort: bool, pseudo_range_en: bool, doppler_info_en: bool, bit_changes_en: bool, nb_sv_max: u8) -> [u8; 9] {
    let mut cmd = [0u8; 9];
    cmd[0] = 0x04;
    cmd[1] = 0x0A;

    cmd[2] |= ((time >> 24) & 0xFF) as u8;
    cmd[3] |= ((time >> 16) & 0xFF) as u8;
    cmd[4] |= ((time >> 8) & 0xFF) as u8;
    cmd[5] |= (time & 0xFF) as u8;
    if best_effort { cmd[6] |= 1; }
    if pseudo_range_en { cmd[7] |= 1; }
    if doppler_info_en { cmd[7] |= 2; }
    if bit_changes_en { cmd[7] |= 4; }
    cmd[8] |= nb_sv_max;
    cmd
}

/// Captures GNSS signals independent of assistance data availability. FW 02.01+ only. Two types: Cold start (no time/position, determines via demod/2D solving) or Assisted (time+position known, searches 12 strongest visible satellites). Use sleep with retention to preserve assistance data. Can be aborted by sending 0x0 on SPI while BUSY active (max 2.9s abort delay).
pub fn gnss_scan_cmd(best_effort: bool, pseudo_range_en: bool, doppler_info_en: bool, bit_changes_en: bool, nb_sv_max: u8) -> [u8; 5] {
    let mut cmd = [0u8; 5];
    cmd[0] = 0x04;
    cmd[1] = 0x0B;

    if best_effort { cmd[2] |= 1; }
    if pseudo_range_en { cmd[3] |= 1; }
    if doppler_info_en { cmd[3] |= 2; }
    if bit_changes_en { cmd[3] |= 4; }
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
pub fn gnss_set_almanac_update_cmd(gps_en: bool, beidou_en: bool) -> [u8; 9] {
    let mut cmd = [0u8; 9];
    cmd[0] = 0x04;
    cmd[1] = 0x02;

    if gps_en { cmd[8] |= 2; }
    if beidou_en { cmd[8] |= 4; }
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
pub fn gnss_read_results_cmd() -> [u8; 2] {
    [0x04, 0x0D]
}

/// Pushes messages from GNSS solver to LR1120 (e.g., assistance position update)
pub fn gnss_push_solver_msg_cmd() -> [u8; 2] {
    [0x04, 0x14]
}

/// Pushes messages from LoRaWAN network to LR1120. FW 02.01+ only.
pub fn gnss_push_dm_msg_cmd() -> [u8; 2] {
    [0x04, 0x15]
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
pub fn gnss_get_sv_visible_req(time: u32, latitude: u16, longitude: u16, gps_en: bool, beidou_en: bool) -> [u8; 11] {
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
    if gps_en { cmd[10] |= 1; }
    if beidou_en { cmd[10] |= 2; }
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
pub fn gnss_fetch_time_cmd(best_effort: bool, fetch_time_mode: FetchTimeMode) -> [u8; 4] {
    let mut cmd = [0u8; 4];
    cmd[0] = 0x04;
    cmd[1] = 0x32;

    if best_effort { cmd[2] |= 1; }
    cmd[3] |= fetch_time_mode as u8;
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

/// Reads number of GPS time Week Number rollover (every 1024 weeks). In 2023, value is 2. Can be changed by GnssSetTime, GnssFetchTime, or GnssScan. FW 02.01+ only.
pub fn gnss_read_wn_rollover_req() -> [u8; 2] {
    [0x04, 0x67]
}

/// Reads number of visible satellites and time elapsed since last update of detected satellite list for this constellation. FW 02.01+ only.
pub fn gnss_read_warm_start_status_req(gps_en: bool, beidou_en: bool) -> [u8; 3] {
    let mut cmd = [0u8; 3];
    cmd[0] = 0x04;
    cmd[1] = 0x69;

    if gps_en { cmd[2] |= 1; }
    if beidou_en { cmd[2] |= 2; }
    cmd
}

/// Returns list of satellites for next keep sync scan. Must call GnssReadWarmStartStatus first to know how many satellites in list. FW 02.01+ only.
pub fn gnss_get_sv_warm_start_req(gps_en: bool, beidou_en: bool) -> [u8; 3] {
    let mut cmd = [0u8; 3];
    cmd[0] = 0x04;
    cmd[1] = 0x66;

    if gps_en { cmd[2] |= 1; }
    if beidou_en { cmd[2] |= 2; }
    cmd
}

/// Configures LR1120 to search for Almanacs for each satellite. For GPS: 32-bit mask for satellites 1-32. For BeiDou: two 32-bit masks for satellites 1-32 and 33-63. FW 02.01+ only.
pub fn gnss_write_bit_mask_sat_activated_cmd(gps_en: bool, beidou_en: bool, bit_mask_activated_0: u32) -> [u8; 7] {
    let mut cmd = [0u8; 7];
    cmd[0] = 0x04;
    cmd[1] = 0x72;

    if gps_en { cmd[2] |= 1; }
    if beidou_en { cmd[2] |= 2; }
    cmd[3] |= ((bit_mask_activated_0 >> 24) & 0xFF) as u8;
    cmd[4] |= ((bit_mask_activated_0 >> 16) & 0xFF) as u8;
    cmd[5] |= ((bit_mask_activated_0 >> 8) & 0xFF) as u8;
    cmd[6] |= (bit_mask_activated_0 & 0xFF) as u8;
    cmd
}

/// Configures LR1120 to search for Almanacs for each satellite. For GPS: 32-bit mask for satellites 1-32. For BeiDou: two 32-bit masks for satellites 1-32 and 33-63. FW 02.01+ only.
pub fn gnss_write_bit_mask_sat_activated_adv_cmd(gps_en: bool, beidou_en: bool, bit_mask_activated_0: u32, bit_mask_activated_1: u32) -> [u8; 11] {
    let mut cmd = [0u8; 11];
    cmd[0] = 0x04;
    cmd[1] = 0x72;

    if gps_en { cmd[2] |= 1; }
    if beidou_en { cmd[2] |= 2; }
    cmd[3] |= ((bit_mask_activated_0 >> 24) & 0xFF) as u8;
    cmd[4] |= ((bit_mask_activated_0 >> 16) & 0xFF) as u8;
    cmd[5] |= ((bit_mask_activated_0 >> 8) & 0xFF) as u8;
    cmd[6] |= (bit_mask_activated_0 & 0xFF) as u8;
    cmd[7] |= ((bit_mask_activated_1 >> 24) & 0xFF) as u8;
    cmd[8] |= ((bit_mask_activated_1 >> 16) & 0xFF) as u8;
    cmd[9] |= ((bit_mask_activated_1 >> 8) & 0xFF) as u8;
    cmd[10] |= (bit_mask_activated_1 & 0xFF) as u8;
    cmd
}

/// Launches GNSS scan to download Almanac parameters from satellite signal (subframe 4/5) for one constellation. Must be sent at precise time matching Almanac data availability - use GnssReadAlmanacStatus. Default: Almanac in RAM, written to flash when >6 satellites available or >half almanacs to update available. Can be aborted. FW 02.01+ only.
pub fn gnss_almanac_update_from_sat_cmd(best_effort: bool, gps_en: bool, beidou_en: bool) -> [u8; 4] {
    let mut cmd = [0u8; 4];
    cmd[0] = 0x04;
    cmd[1] = 0x55;

    if best_effort { cmd[2] |= 1; }
    if gps_en { cmd[3] |= 1; }
    if beidou_en { cmd[3] |= 2; }
    cmd
}

/// Returns detailed almanac update status for both GPS and BeiDou constellations including which satellites need update, next subframe timing, and activation status. Updated when SV almanac demodulated and stored in retention memory or flash. All 53 bytes must be read. FW 02.01+ only.
pub fn gnss_read_almanac_status_req() -> [u8; 2] {
    [0x04, 0x57]
}

/// Configures Almanac update period (days) after which application notified via GnssReadAlmanacStatus. Age compared with Period. Defaults: GPS 31 days, BeiDou MEO 60 days, BeiDou IGSO 30 days. FW 02.01+ only.
pub fn gnss_config_almanac_update_period_cmd(gps_en: bool, beidou_en: bool, beidou_type: BeidouType, period: u16) -> [u8; 6] {
    let mut cmd = [0u8; 6];
    cmd[0] = 0x04;
    cmd[1] = 0x63;

    if gps_en { cmd[2] |= 1; }
    if beidou_en { cmd[2] |= 2; }
    cmd[3] |= beidou_type as u8;
    cmd[4] |= ((period >> 8) & 0xFF) as u8;
    cmd[5] |= (period & 0xFF) as u8;
    cmd
}

/// Reads Almanac update period for constellation and SV type. FW 02.01+ only.
pub fn gnss_read_almanac_update_period_req(gps_en: bool, beidou_en: bool, beidou_type: BeidouType) -> [u8; 4] {
    let mut cmd = [0u8; 4];
    cmd[0] = 0x04;
    cmd[1] = 0x64;

    if gps_en { cmd[2] |= 1; }
    if beidou_en { cmd[2] |= 2; }
    cmd[3] |= beidou_type as u8;
    cmd
}

/// Updates all Almanac data for all satellites. Each constellation updated separately. Total 2580 bytes: 20-byte header + 128 satellites * 20 bytes. Max 512 bytes per SPI transaction - requires multiple transactions. Two approaches: (1) 129 transactions of 20 bytes each (min memory), (2) 5 transactions of 500 bytes + 1 of 80 bytes (min transactions). Almanac stored in flash, kept after power off/sleep without retention.
pub fn gnss_almanac_full_update_cmd() -> [u8; 2] {
    [0x04, 0x0E]
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

    /// GPS constellation Enabled
    pub fn gps_en(&self) -> bool {
        self.0[1] & 0x1 != 0
    }

    /// BeiDou constellation Enabled
    pub fn beidou_en(&self) -> bool {
        (self.0[1] >> 1) & 0x1 != 0
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

    /// GPS constellation supported
    pub fn gps_en(&self) -> bool {
        self.0[1] & 0x1 != 0
    }

    /// BeiDou constellation supported
    pub fn beidou_en(&self) -> bool {
        (self.0[1] >> 1) & 0x1 != 0
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

    /// Source of error, if any
    pub fn context_error(&self) -> ContextError {
        ((self.0[8] >> 4) & 0xF).into()
    }

    /// GPS almanac update
    pub fn gps_en(&self) -> bool {
        (self.0[8] >> 1) & 0x1 != 0
    }

    /// BeiDou almanac update
    pub fn beidou_en(&self) -> bool {
        (self.0[8] >> 2) & 0x1 != 0
    }

    /// Frequency search space in Heartz
    pub fn freq_search_space(&self) -> FreqSearchSpace {
        ((self.0[9] >> 7) |
        ((self.0[8] & 0x1) << 1)).into()
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

    /// GPS almanac update
    pub fn gps_en(&self) -> bool {
        (self.0[1] >> 1) & 0x1 != 0
    }

    /// BeiDou almanac update
    pub fn beidou_en(&self) -> bool {
        (self.0[1] >> 2) & 0x1 != 0
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
pub struct GnssGetSvDetectedRsp([u8; 4]);

impl GnssGetSvDetectedRsp {

    /// Create struct from existing response buffer
    pub fn from_slice(buffer: &[u8]) -> Self {
        let raw : [u8; 4] = buffer.try_into().expect("Buffer size should match response size !");
        Self(raw)
    }

    /// Identifier
    pub fn sv_id(&self) -> u8 {
        self.0[0]
    }

    /// SNR in dB
    pub fn snr(&self) -> u8 {
        self.0[1]
    }

    /// Doppler estimation
    pub fn doppler(&self) -> i16 {
        let raw = (self.0[3] as u16) |
            ((self.0[2] as u16) << 8);
        raw as i16
    }
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
    pub fn gnss_scan_type(&self) -> GnssScanType {
        self.0[1].into()
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
    pub fn accuracy(&self) -> u32 {
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

    /// Failure reason
    pub fn solver_error(&self) -> SolverError {
        self.0[1].into()
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

/// Response for GnssReadWNRollover command
#[derive(Default)]
pub struct GnssReadWNRolloverRsp([u8; 3]);

impl GnssReadWNRolloverRsp {
    /// Create a new response buffer
    pub fn new() -> Self {
        Self::default()
    }

    /// Return Status
    pub fn status(&mut self) -> Status {
        self.0[0].into()
    }

    /// Source of week number. Can be not set, set by GnssScan, or from GnssSetTime/GnssFetchTime.
    pub fn wn_source(&self) -> WnSource {
        self.0[1].into()
    }

    /// Current GPS Week Number rollover value
    pub fn wn_rollover(&self) -> u8 {
        self.0[2]
    }
}

impl AsMut<[u8]> for GnssReadWNRolloverRsp {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

/// Response for GnssReadWarmStartStatus command
#[derive(Default)]
pub struct GnssReadWarmStartStatusRsp([u8; 6]);

impl GnssReadWarmStartStatusRsp {
    /// Create a new response buffer
    pub fn new() -> Self {
        Self::default()
    }

    /// Return Status
    pub fn status(&mut self) -> Status {
        self.0[0].into()
    }

    /// Number of visible satellites in selected constellation. If 0: no satellites found or all unhealthy/deactivated
    pub fn nb_sv(&self) -> u8 {
        self.0[1]
    }

    /// Seconds since last update of satellite list. If 0xFFFFFFFF: time never set in LR1120
    pub fn time_elapsed(&self) -> u32 {
        (self.0[5] as u32) |
        ((self.0[4] as u32) << 8) |
        ((self.0[3] as u32) << 16) |
        ((self.0[2] as u32) << 24)
    }
}

impl AsMut<[u8]> for GnssReadWarmStartStatusRsp {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

/// Response for GnssGetSvWarmStart command
#[derive(Default)]
pub struct GnssGetSvWarmStartRsp([u8; 2]);

impl GnssGetSvWarmStartRsp {
    /// Create a new response buffer
    pub fn new() -> Self {
        Self::default()
    }

    /// Return Status
    pub fn status(&mut self) -> Status {
        self.0[0].into()
    }
    // TODO: Implement accessor for variable length field 'sv_list'
}

impl AsMut<[u8]> for GnssGetSvWarmStartRsp {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

/// Response for GnssReadAlmanacStatus command
pub struct GnssReadAlmanacStatusRsp([u8; 54]);

impl GnssReadAlmanacStatusRsp {
    /// Create a new response buffer
    pub fn new() -> Self {
        Self::default()
    }

    /// Return Status
    pub fn status(&mut self) -> Status {
        self.0[0].into()
    }

    /// GPS almanac status
    pub fn gps_status(&self) -> AlmanacStatus {
        self.0[1].into()
    }

    /// Milliseconds before next GPS satellite data subframe for new GPS Almanac
    pub fn gps_time_to_next_subframe(&self) -> u32 {
        (self.0[5] as u32) |
        ((self.0[4] as u32) << 8) |
        ((self.0[3] as u32) << 16) |
        ((self.0[2] as u32) << 24)
    }

    /// Next number of GPS subframes to demodulate
    pub fn gps_next_subframes_to_demod(&self) -> u8 {
        self.0[6]
    }

    /// GPS satellite number that can be demodulated in next subframe 4
    pub fn gps_sv_in_subframe_4(&self) -> u8 {
        self.0[7]
    }

    /// GPS satellite number that can be demodulated in next subframe 5
    pub fn gps_sv_in_subframe_5(&self) -> u8 {
        self.0[8]
    }

    /// Next GPS subframe ID start: 4 or 5 (if first almanac in that subframe), 0 (if no next almanac to demodulate)
    pub fn gps_next_subframe_id(&self) -> u8 {
        self.0[9]
    }

    /// Total number of GPS satellites requiring Almanac update. Default at device start: 32 (full update needed)
    pub fn gps_total_sv_to_update(&self) -> u8 {
        self.0[10]
    }

    /// Bit mask for GPS satellites 1-32 needing almanac update. Bit 0=SV1, Bit 31=SV32. 0: already updated, 1: needs update
    pub fn gps_sv_almanac_to_update_mask(&self) -> u32 {
        (self.0[14] as u32) |
        ((self.0[13] as u32) << 8) |
        ((self.0[12] as u32) << 16) |
        ((self.0[11] as u32) << 24)
    }

    /// Bit mask for GPS satellites 1-32 activation. Bit 0=SV1, Bit 31=SV32. 0: not activated, 1: activated
    pub fn gps_sv_activated_mask(&self) -> u32 {
        (self.0[18] as u32) |
        ((self.0[17] as u32) << 8) |
        ((self.0[16] as u32) << 16) |
        ((self.0[15] as u32) << 24)
    }

    /// BeiDou almanac status
    pub fn beidou_status(&self) -> AlmanacStatus {
        self.0[19].into()
    }

    /// Milliseconds before next BeiDou satellite data subframe for new BeiDou Almanac
    pub fn beidou_time_to_next_subframe(&self) -> u32 {
        (self.0[23] as u32) |
        ((self.0[22] as u32) << 8) |
        ((self.0[21] as u32) << 16) |
        ((self.0[20] as u32) << 24)
    }

    /// Next number of BeiDou subframes to demodulate
    pub fn beidou_next_subframes_to_demod(&self) -> u8 {
        self.0[24]
    }

    /// BeiDou satellite number that can be demodulated in next subframe 4
    pub fn beidou_sv_in_subframe_4(&self) -> u8 {
        self.0[25]
    }

    /// BeiDou satellite number that can be demodulated in next subframe 5
    pub fn beidou_sv_in_subframe_5(&self) -> u8 {
        self.0[26]
    }

    /// Next BeiDou subframe ID start: 4 or 5 (if first almanac in that subframe), 0 (if no next almanac to demodulate)
    pub fn beidou_next_subframe_id(&self) -> u8 {
        self.0[27]
    }

    /// Total number of BeiDou satellites requiring Almanac update. Default at device start: 34 (full update needed)
    pub fn beidou_total_sv_to_update(&self) -> u8 {
        self.0[28]
    }

    /// Bit mask for BeiDou satellites needing almanac update.
    pub fn beidou_sv_almanac_to_update_mask(&self) -> u64 {
        (self.0[36] as u64) |
        ((self.0[35] as u64) << 8) |
        ((self.0[34] as u64) << 16) |
        ((self.0[33] as u64) << 24) |
        ((self.0[32] as u64) << 32) |
        ((self.0[31] as u64) << 40) |
        ((self.0[30] as u64) << 48) |
        ((self.0[29] as u64) << 56)
    }

    /// Bit mask for BeiDou satellites activated.
    pub fn beidou_sv_activated_mask(&self) -> u64 {
        (self.0[44] as u64) |
        ((self.0[43] as u64) << 8) |
        ((self.0[42] as u64) << 16) |
        ((self.0[41] as u64) << 24) |
        ((self.0[40] as u64) << 32) |
        ((self.0[39] as u64) << 40) |
        ((self.0[38] as u64) << 48) |
        ((self.0[37] as u64) << 56)
    }

    /// Bit mask for BeiDou satellites blacklisted.
    pub fn beidou_sv_blacklist_mask(&self) -> u64 {
        (self.0[52] as u64) |
        ((self.0[51] as u64) << 8) |
        ((self.0[50] as u64) << 16) |
        ((self.0[49] as u64) << 24) |
        ((self.0[48] as u64) << 32) |
        ((self.0[47] as u64) << 40) |
        ((self.0[46] as u64) << 48) |
        ((self.0[45] as u64) << 56)
    }

    /// Pages 11-24 of subframe 5 broadcast almanacs of SVs 31-63 in 4 NAV messages (12 min each). 0: No almanac on pages 11-24, 1: SVs 31-43, 2: SVs 44-56, 3: SVs 57-63. LR1120 only downloads SVs 31-43.
    pub fn beidou_next_almanac_id(&self) -> u8 {
        self.0[53]
    }
}

impl AsMut<[u8]> for GnssReadAlmanacStatusRsp {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}
impl Default for GnssReadAlmanacStatusRsp {
    fn default() -> Self {
        let content : [u8; 54] = core::array::repeat(0);
        Self(content)
    }
}

/// Response for GnssReadAlmanacUpdatePeriod command
#[derive(Default)]
pub struct GnssReadAlmanacUpdatePeriodRsp([u8; 3]);

impl GnssReadAlmanacUpdatePeriodRsp {
    /// Create a new response buffer
    pub fn new() -> Self {
        Self::default()
    }

    /// Return Status
    pub fn status(&mut self) -> Status {
        self.0[0].into()
    }

    /// Period in days before almanac update needed
    pub fn period(&self) -> u16 {
        (self.0[2] as u16) |
        ((self.0[1] as u16) << 8)
    }
}

impl AsMut<[u8]> for GnssReadAlmanacUpdatePeriodRsp {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}
