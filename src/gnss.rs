//! # API related to GNSS
//!
//! This module provides an API to run GNSS operations.
//!
//! ## Available Methods
//!
//! ### Configuration
//! - [`gnss_set_constellation`](Lr1120::gnss_set_constellation) - Requires 32.768kHz clock for dual constellation. BUSY high until both scans complete.
//! - [`gnss_get_constellation`](Lr1120::gnss_get_constellation) - Reads selected constellation (GPS/BeiDou)
//! - [`gnss_supported_constellation`](Lr1120::gnss_supported_constellation) - Reads supported constellation (GPS/BeiDou)
//! - [`gnss_set_mode`](Lr1120::gnss_set_mode) - Configures GNSS scanning mode (single or multi)
//! - [`gnss_set_assist_pos`](Lr1120::gnss_set_assist_pos) - Configures approximate position for GNSS assisted mode.
//! - [`gnss_get_assist_pos`](Lr1120::gnss_get_assist_pos) - Reads approximate position used for GNSS assisted mode.
//! - [`gnss_set_delay_reset_assist`](Lr1120::gnss_set_delay_reset_assist) - Configures delay after which LR1120 resets Assistance Position and switches from assisted to cold start
//! - [`gnss_get_delay_reset_assist`](Lr1120::gnss_get_delay_reset_assist) - Return reset delay configuration for assistance position
//! - [`gnss_reset_assist`](Lr1120::gnss_reset_assist) - Reset Assist position
//!
//! ### Scan
//! - [`gnss_scan`](Lr1120::gnss_scan) - Captures GNSS signals independant of assistance data availability
//! - [`gnss_get_result_size`](Lr1120::gnss_get_result_size) - Return result size in byte
//! - [`gnss_get_nb_sv`](Lr1120::gnss_get_nb_sv) - Return number of satellite vehicles detected during last scan
//! - [`gnss_get_nb_sv_filt`](Lr1120::gnss_get_nb_sv_filt) - Return number of satellite vehicles detected for a given time position and constellation
//! - [`gnss_get_scan_type`](Lr1120::gnss_get_scan_type) - Returns type of scan launched during last scan.
//! - [`gnss_get_doppler](Lr1120::gnss_get_doppler) - Reads Assistance Position calculated by 2D Solver
//! - [`gnss_get_wn_rollover](Lr1120::gnss_get_wn_rollover) - Reads number of GPS time Week Number rollover (every 1024 weeks).
//! - [`gnss_get_warm_start_status](Lr1120::gnss_get_warm_start_status) - Reads number of visible satellites and time elapsed since last update of detected satellite list for this constellation.
//! - [`gnss_get_warm_start_sv](Lr1120::gnss_get_warm_start_sv) - Returns list of satellites ID for next keep sync scan.
//!
//! ### Time
//! - [`gnss_fetch_time`](Lr1120::gnss_fetch_time) - Determine time by demodulating satellite signals
//! - [`gnss_get_time`](Lr1120::gnss_get_time) - Return GPS Time
//! - [`gnss_reset_time`](Lr1120::gnss_reset_time) - Reset GPS Time
//! - [`gnss_set_time`](Lr1120::gnss_set_time) - Allows MCU host to set GPS Time
//!
//! ### Almanac
//! - [`gnss_set_almanac_update`](Lr1120::gnss_set_almanac_update) - Enable Almanac update for constellation GPS/Beidou
//! - [`gnss_get_almanac_update`](Lr1120::gnss_get_almanac_update) - Get aLmanac update configruation
//! - [`gnss_set_gps_sat_bitmask`](Lr1120::gnss_set_gps_sat_bitmask) - Configures LR1120 to search for Almanacs for each GPS satellite enabled by the mask
//! - [`gnss_set_beidou_sat_bitmask`](Lr1120::gnss_set_beidou_sat_bitmask) - Configures LR1120 to search for Almanacs for each Beidou satellite enabled by the masks
//! - [`gnss_updt_almanac_from_sat`](Lr1120::gnss_updt_almanac_from_sat) - Launches GNSS scan to download Almanac parameters from satellite signal (subframe 4/5) for one constellation.
//! - [`gnss_set_almanac_updt_period`](Lr1120::gnss_set_almanac_updt_period) - Configures Almanac update period (days) after which application notified via GnssReadAlmanacStatus.
//! - [`gnss_get_almanac_updt_period`](Lr1120::gnss_get_almanac_updt_period) - Read Almanac update period (days)
//! - [`gnss_get_almanac_status`](Lr1120::gnss_get_almanac_status) - Returns detailed almanac update status for both GPS and BeiDou constellations including which satellites need update, next subframe timing, and activation status.
//!
//! ### Message
//! - [`gnss_push_solver_msg`](Lr1120::gnss_push_solver_msg) Pushes messages from GNSS solver to LR1120 (e.g., assistance position update)
//! - [`gnss_push_dm_msg`](Lr1120::gnss_push_dm_msg) Pushes messages from LoRaWAN network to LR1120
//!
//! ### Misc
//! - [`gnss_get_version`](Lr1120::gnss_get_version) - Get the firmware and almanac version
//! - [`gnss_get_consumption`](Lr1120::gnss_get_consumption) - Return result size in byte
//!


use embedded_hal::digital::OutputPin;
use embedded_hal_async::spi::SpiBus;
use embassy_time::Duration;

use super::{BusyPin, Lr1120, Lr1120Error};

pub use crate::cmd::cmd_gnss::*;

#[derive(Debug, Clone)]
/// GNSS Scan configuration
pub struct GnssScanCfg {
    /// Continue scan even without strong satellites
    pub best_effort: bool,
    /// Include PseduoRange in NAV message
    pub pseudo_range: bool,
    /// Include Doppler information in NAV message
    pub doppler_info: bool,
    /// Include bit changes in NAV message
    pub bit_changes: bool,
    /// Max number of satellites reported
    pub max_sv: u8,
}

impl GnssScanCfg {
    /// Create GNSS scan configuration
    pub fn new(best_effort: bool) -> Self {
        GnssScanCfg {
            best_effort,
            pseudo_range: true,
            doppler_info: true,
            bit_changes: false,
            max_sv: if best_effort {0} else {4}
        }
    }
}

#[derive(Debug, Clone, Default)]
/// Almanac header used for update
pub struct AlmanacHeader {
    pub date: u16,
    pub crc: u32,
}

impl AlmanacHeader {
    /// Create New Almanac header
    pub fn new(date: u16, crc: u32) -> Self {
        AlmanacHeader { date, crc }
    }

    /// Copy struct in buffer of bytes
    /// Must be 20 byte long to get the whole structure
    pub fn to_bytes(&self, buffer: &mut [u8]) {
        for (i,b) in buffer.iter_mut().take(20).enumerate() {
            match i {
                0 => *b = 128,
                1 => *b = (self.date>>8) as u8,
                2 => *b = (self.date&0xFF) as u8,
                3 => *b = ( self.crc>>24      ) as u8,
                4 => *b = ((self.crc>>16)&0xFF) as u8,
                5 => *b = ((self.crc>> 8)&0xFF) as u8,
                6 => *b = ( self.crc     &0xFF) as u8,
                _ => *b = 0,
            }
        }
    }
}

#[derive(Debug, Clone, Default)]
/// Almanac for one satelitte
pub struct AlmanacSv {
    pub sv_id     : u8,
    pub content   : [u8;15],
    pub ca_code   : u16,
    pub modulation: u8,
    pub const_id  : u8,
}

impl AlmanacSv {

    /// Copy struct in buffer of bytes
    /// Must be 20 byte long to get the whole structure
    pub fn to_bytes(&self, buffer: &mut [u8]) {
        for (i,b) in buffer.iter_mut().take(20).enumerate() {
            match i {
                0 => *b = self.sv_id,
                ci if (1..=15).contains(&ci) => *b = self.content[ci],
                16 => *b = (self.ca_code>>8) as u8,
                17 => *b = (self.ca_code&0xFF) as u8,
                18 => *b = self.modulation,
                19 => *b = self.const_id,
                _ => *b = 0,
            }
        }
    }
}


impl<O,SPI, M> Lr1120<O,SPI, M> where
    O: OutputPin, SPI: SpiBus<u8>, M: BusyPin
{

    /// Configures GNSS scanning for selected constellation (GPS/BeiDou).
    /// If both selected, GPS scans first, then BeiDou after delay (4s fixed for FW ≤01.02, variable 1s steps for FW 02.01+).
    /// Requires 32.768kHz clock for dual constellation. BUSY high until both scans complete.
    pub async fn gnss_set_constellation(&mut self, gps: bool, beidou: bool) -> Result<(), Lr1120Error> {
        let req = gnss_set_constellation_to_use_cmd(gps, beidou);
        self.cmd_wr(&req).await
    }

    /// Reads selected constellation (GPS/BeiDou)
    pub async fn gnss_get_constellation(&mut self) -> Result<GnssReadConstellationToUseRsp, Lr1120Error> {
        let req = gnss_read_constellation_to_use_req();
        let mut rsp = GnssReadConstellationToUseRsp::new();
        self.cmd_rd(&req, rsp.as_mut()).await?;
        Ok(rsp)
    }

    /// Reads supported constellation (GPS/BeiDou)
    pub async fn gnss_supported_constellation(&mut self) -> Result<GnssReadSupportedConstellationsRsp, Lr1120Error> {
        let req = gnss_read_supported_constellations_req();
        let mut rsp = GnssReadSupportedConstellationsRsp::new();
        self.cmd_rd(&req, rsp.as_mut()).await?;
        Ok(rsp)
    }

    /// Configures GNSS scanning mode (single or multi)
    pub async fn gnss_set_mode(&mut self, mode: GnssScanMode) -> Result<(), Lr1120Error> {
        let req = gnss_set_mode_cmd(mode);
        self.cmd_wr(&req).await
    }

    /// Captures GNSS signals in autonomous mode (no assistance info available)
    /// Resets GNSS results and maintains busy high during scan.
    /// Time is the GPS time in seconds elapsed since 1980/01/06
    #[cfg(feature = "gnss_v1")]
    #[doc(cfg(feature = "gnss_v1"))]
    pub async fn gnss_autonomous(&mut self, time: u32, cfg: GnssScanCfg) -> Result<(), Lr1120Error> {
        let req = gnss_autonomous_cmd(time, cfg.best_effort, cfg.pseudo_range, cfg.doppler_info, cfg.bit_changes, cfg.max_sv);
        self.cmd_wr(&req).await
    }

    /// Captures GNSS signals in assisted mode
    /// Resets GNSS results and maintains busy high during scan.
    /// Time is the GPS time in seconds elapsed since 1980/01/06
    #[cfg(feature = "gnss_v1")]
    #[doc(cfg(feature = "gnss_v1"))]
    pub async fn gnss_assisted(&mut self, time: u32, cfg: GnssScanCfg) -> Result<(), Lr1120Error> {
        let req = gnss_assisted_cmd(time, cfg.best_effort, cfg.pseudo_range, cfg.doppler_info, cfg.bit_changes, cfg.max_sv);
        self.cmd_wr(&req).await
    }

    /// Captures GNSS signals independant of assistance data availability
    /// Resets GNSS results and maintains busy high during scan.
    /// Use sleep with retention to preserve assistance data
    #[cfg(not(feature = "gnss_v1"))]
    pub async fn gnss_scan(&mut self, cfg: GnssScanCfg) -> Result<(), Lr1120Error> {
        let req = gnss_scan_cmd(cfg.best_effort, cfg.pseudo_range, cfg.doppler_info, cfg.bit_changes, cfg.max_sv);
        self.cmd_wr(&req).await
    }

    /// Configures approximate position for GNSS assisted mode.
    pub async fn gnss_set_assist_pos(&mut self, latitude: u16, longitude: u16) -> Result<(), Lr1120Error> {
        let req = gnss_set_assistance_position_cmd(latitude, longitude);
        self.cmd_wr(&req).await
    }

    /// Reads approximate position used for GNSS assisted mode.
    pub async fn gnss_get_assist_pos(&mut self) -> Result<GnssReadAssistancePositionRsp, Lr1120Error> {
        let req = gnss_read_assistance_position_req();
        let mut rsp = GnssReadAssistancePositionRsp::new();
        self.cmd_rd(&req, rsp.as_mut()).await?;
        Ok(rsp)
    }

    /// Reads GNSS context status including firmware version, almanac CRC, error codes, and frequency search space
    pub async fn gnss_get_context_status(&mut self) -> Result<GnssGetContextStatusRsp, Lr1120Error> {
        let req = gnss_get_consumption_req();
        let mut rsp = GnssGetContextStatusRsp::new();
        self.cmd_rd(&req, rsp.as_mut()).await?;
        Ok(rsp)
    }

    /// Get the firmware and almanac version
    pub async fn gnss_get_version(&mut self) -> Result<GnssReadVersionRsp, Lr1120Error> {
        let req = gnss_read_version_req();
        let mut rsp = GnssReadVersionRsp::new();
        self.cmd_rd(&req, rsp.as_mut()).await?;
        Ok(rsp)
    }

    /// Enable Almanac update for constellation GPS/Beidou
    pub async fn gnss_set_almanac_update(&mut self, gps: bool, beidou: bool) -> Result<(), Lr1120Error> {
        let req = gnss_set_almanac_update_cmd(gps, beidou);
        self.cmd_wr(&req).await
    }

    /// Get aLmanac update configruation
    pub async fn gnss_get_almanac_update(&mut self) -> Result<GnssReadAlmanacUpdateRsp, Lr1120Error> {
        let req = gnss_read_almanac_update_req();
        let mut rsp = GnssReadAlmanacUpdateRsp::new();
        self.cmd_rd(&req, rsp.as_mut()).await?;
        Ok(rsp)
    }

    /// Return result size in byte
    pub async fn gnss_get_result_size(&mut self) -> Result<u16, Lr1120Error> {
        let req = gnss_get_result_size_req();
        let mut rsp = GnssGetResultSizeRsp::new();
        self.cmd_rd(&req, rsp.as_mut()).await?;
        Ok(rsp.result_size())
    }

    /// Return number of satellite vehicles detected during last scan
    pub async fn gnss_get_nb_sv(&mut self) -> Result<u8, Lr1120Error> {
        let req = gnss_get_nb_sv_detected_req();
        let mut rsp = GnssGetNbSvDetectedRsp::new();
        self.cmd_rd(&req, rsp.as_mut()).await?;
        Ok(rsp.nb_sv())
    }

    /// Return number of satellite vehicles detected for a given time position and constellation
    pub async fn gnss_get_nb_sv_filt(&mut self, time: u32, latitude: u16, longitude: u16, gps: bool, beidou: bool) -> Result<u8, Lr1120Error> {
        let req = gnss_get_sv_visible_req(time, latitude, longitude, gps, beidou);
        let mut rsp = GnssGetSvVisibleRsp::new();
        self.cmd_rd(&req, rsp.as_mut()).await?;
        Ok(rsp.nb_sv_visible())
    }


    /// Configures delay after which LR1120 resets Assistance Position and switches from assisted to cold start
    #[cfg(not(feature = "gnss_v1"))]
    pub async fn gnss_set_delay_reset_assist(&mut self, delay: u32) -> Result<(), Lr1120Error> {
        let req = gnss_config_delay_reset_ap_cmd(delay);
        self.cmd_wr(&req).await
    }

    /// Return reset delay configuration for assistance position
    #[cfg(not(feature = "gnss_v1"))]
    pub async fn gnss_get_delay_reset_assist(&mut self) -> Result<u32, Lr1120Error> {
        let req = gnss_read_delay_reset_ap_req();
        let mut rsp = GnssReadDelayResetAPRsp::new();
        self.cmd_rd(&req, rsp.as_mut()).await?;
        Ok(rsp.delay())
    }

    /// Reset Assist position
    #[cfg(not(feature = "gnss_v1"))]
    pub async fn gnss_reset_assist(&mut self) -> Result<(), Lr1120Error> {
        let req = gnss_reset_position_cmd();
        self.cmd_wr(&req).await
    }


    /// Pushes messages from GNSS solver to LR1120 (e.g., assistance position update)
    pub async fn gnss_push_solver_msg(&mut self, msg: &[u8]) -> Result<(), Lr1120Error> {
        let req = gnss_push_solver_msg_cmd();
        self.cmd_data_wr(&req, msg).await
    }

    /// Pushes messages from LoRaWAN network to LR1120
    #[cfg(not(feature = "gnss_v1"))]
    pub async fn gnss_push_dm_msg(&mut self, msg: &[u8]) -> Result<(), Lr1120Error> {
        let req = gnss_push_dm_msg_cmd();
        self.cmd_data_wr(&req, msg).await
    }

    /// Return result size in byte
    pub async fn gnss_get_consumption(&mut self) -> Result<GnssGetConsumptionRsp, Lr1120Error> {
        let req = gnss_get_consumption_req();
        let mut rsp = GnssGetConsumptionRsp::new();
        self.cmd_rd(&req, rsp.as_mut()).await?;
        Ok(rsp)
    }

    /// Returns type of scan launched during last scan.
    #[cfg(not(feature = "gnss_v1"))]
    pub async fn gnss_get_scan_type(&mut self) -> Result<GnssScanType, Lr1120Error> {
        let req = gnss_read_last_scan_mode_launched_req();
        let mut rsp = GnssReadLastScanModeLaunchedRsp::new();
        self.cmd_rd(&req, rsp.as_mut()).await?;
        Ok(rsp.gnss_scan_type())
    }

    /// Determine time by demodulating satellite signals
    #[cfg(not(feature = "gnss_v1"))]
    pub async fn gnss_fetch_time(&mut self, best_effort: bool, mode: FetchTimeMode) -> Result<(), Lr1120Error> {
        let req = gnss_fetch_time_cmd(best_effort, mode);
        self.cmd_wr(&req).await
    }

    /// Return GPS Time
    #[cfg(not(feature = "gnss_v1"))]
    pub async fn gnss_get_time(&mut self) -> Result<GnssReadTimeRsp, Lr1120Error> {
        let req = gnss_read_time_req();
        let mut rsp = GnssReadTimeRsp::new();
        self.cmd_rd(&req, rsp.as_mut()).await?;
        Ok(rsp)
    }

    /// Reset GPS Time
    #[cfg(not(feature = "gnss_v1"))]
    pub async fn gnss_reset_time(&mut self) -> Result<(), Lr1120Error> {
        let req = gnss_reset_time_cmd();
        self.cmd_wr(&req).await
    }

    /// Allows MCU host to set GPS Time
    #[cfg(not(feature = "gnss_v1"))]
    pub async fn gnss_set_time(&mut self, time: u32, accuracy: u16) -> Result<(), Lr1120Error> {
        let req = gnss_set_time_cmd(time, accuracy);
        self.cmd_wr(&req).await
    }

    /// Reads Assistance Position calculated by 2D Solver
    #[cfg(not(feature = "gnss_v1"))]
    pub async fn gnss_get_doppler(&mut self) -> Result<GnssReadDopplerSolverResRsp, Lr1120Error> {
        let req = gnss_read_doppler_solver_res_req();
        let mut rsp = GnssReadDopplerSolverResRsp::new();
        self.cmd_rd(&req, rsp.as_mut()).await?;
        Ok(rsp)
    }

    /// Reads number of GPS time Week Number rollover (every 1024 weeks).
    /// In 2025, value is 4. Can be changed by GnssSetTime, GnssFetchTime, or GnssScan.
    #[cfg(not(feature = "gnss_v1"))]
    pub async fn gnss_get_wn_rollover(&mut self) -> Result<GnssReadWNRolloverRsp, Lr1120Error> {
        let req = gnss_read_wn_rollover_req();
        let mut rsp = GnssReadWNRolloverRsp::new();
        self.cmd_rd(&req, rsp.as_mut()).await?;
        Ok(rsp)
    }

    /// Reads number of visible satellites and time elapsed since last update of detected satellite list for this constellation.
    #[cfg(not(feature = "gnss_v1"))]
    pub async fn gnss_get_warm_start_status(&mut self, gps: bool, beidou: bool) -> Result<GnssReadWarmStartStatusRsp, Lr1120Error> {
        let req = gnss_read_warm_start_status_req(gps, beidou);
        let mut rsp = GnssReadWarmStartStatusRsp::new();
        self.cmd_rd(&req, rsp.as_mut()).await?;
        Ok(rsp)
    }

    /// Returns list of satellites ID for next keep sync scan.
    /// Must call GnssReadWarmStartStatus first to know how many satellites in list (1 byte per satellites).
    #[cfg(not(feature = "gnss_v1"))]
    pub async fn gnss_get_warm_start_sv(&mut self, gps: bool, beidou: bool, nb_sv: u8) -> Result<&[u8], Lr1120Error> {
        let req = gnss_get_sv_warm_start_req(gps, beidou);
        self.cmd_wr(&req).await?;
        self.wait_ready(Duration::from_millis(1)).await?;
        let rsp_len = nb_sv as usize;
        self.rsp_rd(rsp_len).await?;
        Ok(&self.buffer()[..rsp_len])
    }

    /// Configures LR1120 to search for Almanacs for each GPS satellite enabled by the mask
    /// If mask is none, use default value (0xFFFFFFFF)
    #[cfg(not(feature = "gnss_v1"))]
    pub async fn gnss_set_gps_sat_bitmask(&mut self, mask: Option<u32>) -> Result<(), Lr1120Error> {
        let mask = mask.unwrap_or(0xFFFFFFFF);
        let req = gnss_write_bit_mask_sat_activated_cmd(true, false, mask);
        self.cmd_wr(&req).await
    }

    /// Configures LR1120 to search for Almanacs for each Beidou satellite enabled by the masks
    /// If mask is none, use default value (0xBFFCBFFF, 0xC0007FF)
    #[cfg(not(feature = "gnss_v1"))]
    pub async fn gnss_set_beidou_sat_bitmask(&mut self, mask: Option<(u32,u32)>) -> Result<(), Lr1120Error> {
        let (mask0,mask1) = mask.unwrap_or((0xBFFCBFFF, 0xC0007FF));
        let req = gnss_write_bit_mask_sat_activated_adv_cmd(true, false, mask0, mask1);
        self.cmd_wr(&req).await
    }

    /// Launches GNSS scan to download Almanac parameters from satellite signal (subframe 4/5) for one constellation.
    /// Must be sent at precise time matching Almanac data availability - use GnssReadAlmanacStatus.
    /// Default: Almanac in RAM, written to flash when >6 satellites available or >half almanacs to update available.
    #[cfg(not(feature = "gnss_v1"))]
    pub async fn gnss_updt_almanac_from_sat(&mut self, best_effort: bool, gps: bool) -> Result<(), Lr1120Error> {
        let req = gnss_almanac_update_from_sat_cmd(best_effort, gps, !gps);
        self.cmd_wr(&req).await
    }

    /// Manually update the almanac
    #[cfg(not(feature = "gnss_v1"))]
    pub async fn gnss_updt_almanac(&mut self, hdr: AlmanacHeader, sv_list: &[AlmanacSv]) -> Result<(), Lr1120Error> {
        let buffer = self.buffer.as_mut();
        buffer[0] = 0x04;
        buffer[1] = 0x0E;
        // Send Header
        hdr.to_bytes(&mut buffer[2..22]);
        self.cmd_buf_wr(22).await?;
        // Send SV params
        for sv_chunks in sv_list.chunks(25) {
            let buffer = self.buffer.as_mut();
            buffer[0] = 0x04;
            buffer[1] = 0x0E;
            let mut offset = 2;
            for sv in sv_chunks {
                let offset_next = offset + 20;
                sv.to_bytes(&mut buffer[offset..offset_next]);
                offset = offset_next;
            }
            self.cmd_buf_wr(offset).await?;
        }
        Ok(())
    }

    /// Configures Almanac update period (days) after which application notified via GnssReadAlmanacStatus.
    /// If beidou_type is none, constellation is GPS.
    #[cfg(not(feature = "gnss_v1"))]
    pub async fn gnss_set_almanac_updt_period(&mut self, beidou_type: Option<BeidouType>, period: u16) -> Result<(), Lr1120Error> {
        let beidou_en = beidou_type.is_some();
        let beidou_type = beidou_type.unwrap_or(BeidouType::Meo);
        let req = gnss_config_almanac_update_period_cmd(!beidou_en, beidou_en, beidou_type, period);
        self.cmd_wr(&req).await
    }

    /// Read Almanac update period (days)
    #[cfg(not(feature = "gnss_v1"))]
    pub async fn gnss_get_almanac_updt_period(&mut self, beidou_type: Option<BeidouType>) -> Result<u16, Lr1120Error> {
        let beidou_en = beidou_type.is_some();
        let beidou_type = beidou_type.unwrap_or(BeidouType::Meo);
        let req = gnss_read_almanac_update_period_req(!beidou_en, beidou_en, beidou_type);
        let mut rsp = GnssReadAlmanacUpdatePeriodRsp::new();
        self.cmd_rd(&req, rsp.as_mut()).await?;
        Ok(rsp.period())
    }

    /// Returns detailed almanac update status for both GPS and BeiDou constellations including which satellites need update, next subframe timing, and activation status.
    /// Updated when SV almanac demodulated and stored in retention memory or flash.
    #[cfg(not(feature = "gnss_v1"))]
    pub async fn gnss_get_almanac_status(&mut self) -> Result<GnssReadAlmanacStatusRsp, Lr1120Error> {
        let req = gnss_read_almanac_status_req();
        let mut rsp = GnssReadAlmanacStatusRsp::new();
        self.cmd_rd(&req, rsp.as_mut()).await?;
        Ok(rsp)
    }


}