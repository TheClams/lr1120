//! # API related to WiFi Scanning
//!
//! This module provides an API for configuring and operating the LR1120 chip for WiFi scanning.
//!
//! ## Available Methods
//!
//! ### Scanning commands
//! - [`wifi_scan`](Lr1120::wifi_scan) - Capture WiFi packet
//! - [`wifi_scan_time_limit`](Lr1120::wifi_scan_time_limit) - Capture WiFi packet with a time limit per channel in ms
//! - [`wifi_scan_country_code`](Lr1120::wifi_scan_country_code) - Capture WiFi-B beacon and look for `max_res` country code
//! - [`wifi_scan_country_code_time_limit`](Lr1120::wifi_scan_country_code_time_limit) - Capture WiFi-B beacon and look for `max_res` country code
//!
//! ### Results
//! - [`wifi_get_nb_res`](Lr1120::wifi_get_nb_res) - Return number of result capture by previous scanning. Must be called before `wifi_get_result_*` methods
//! - [`wifi_get_nb_country_code`](Lr1120::wifi_get_nb_country_code) - Return number of result capture by previous country code scanning. Must be called before `wifi_get_result_country`
//! - [`wifi_get_result_short`](Lr1120::wifi_get_result_short) - Return short result (9B) of previous Wifi Scanning
//! - [`wifi_get_result_long`](Lr1120::wifi_get_result_long) - Return long result 229B) of previous Wifi Scanning
//! - [`wifi_get_result_ext`](Lr1120::wifi_get_result_ext) - Return extended result (79B) of previous Wifi Scanning
//! - [`wifi_get_result_country`](Lr1120::wifi_get_result_country) - Return country code result (10B) of previous Wifi Scanning Country Code
//!
//! ### Misc
//! - [`wifi_reset_timings`](Lr1120::wifi_reset_timings) - Reset cumulative timings
//! - [`wifi_get_timings`](Lr1120::wifi_get_timings) - Get scanning cumulative timings
//! - [`wifi_set_timestamp_thr`](Lr1120::wifi_set_timestamp_thr) - Configure timestamp threshold (in seconds) to discrimante mobile access point from gateways
//! - [`wifi_get_fw_version`](Lr1120::wifi_get_fw_version) - Return firmware version of wifi-scanning
//!

use core::marker::PhantomData;

use embassy_time::Duration;
use embedded_hal::digital::OutputPin;
use embedded_hal_async::spi::SpiBus;

pub use crate::cmd::cmd_wifi::*;

use super::{BusyPin, Lr1120, Lr1120Error};

#[derive(Debug, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// Wifi Scan parameters
pub struct WifiScanParams {
    /// Channel mask (bit 0 to 13)
    pub chan_mask: u16,
    /// Standard Selection (B, G, N, or all)
    pub standard: WifiStandard,
    /// Acquisition mode
    pub acq_mode: AcqMode,
    /// Maximum number of result (up to 32)
    pub max_res: u8,
    /// Maximum number of scan per channel
    pub max_scan: u8,
    /// Timeout in ms
    pub timeout: u16,
    /// Abort current channel when timeout is reached
    pub abort_on_timeout: bool,
}

impl WifiScanParams {
    /// Create Wifi Scanning Configuration structure with all channel enabled,
    /// 16 results, 8 scans, 105ms timeout
    pub fn new(standard: WifiStandard, acq_mode: AcqMode) -> Self {
        Self {
            chan_mask: 0x3FFF,
            standard,
            acq_mode,
            max_res: 16,
            max_scan: 8,
            timeout: 105,
            abort_on_timeout: true,
        }
    }
}

trait ResultFromSlice<T> {
    fn from_slice(buffer: &[u8]) -> T;
    const SIZE : u8;
}

const WIFI_RES_SHORT_SIZE : u8 = 9;
const WIFI_RES_LONG_SIZE : u8 = 22;
const WIFI_RES_EXT_SIZE : u8 = 79;
const WIFI_RES_COUNTRY_SIZE : u8 = 10;

impl ResultFromSlice<WifiReadResultsRsp> for WifiReadResultsRsp {
    const SIZE : u8 = WIFI_RES_SHORT_SIZE;
    fn from_slice(buffer: &[u8]) -> WifiReadResultsRsp {
        WifiReadResultsRsp::from_slice(buffer)
    }

}

impl ResultFromSlice<WifiReadLongResultsRsp> for WifiReadLongResultsRsp {
    const SIZE : u8 = WIFI_RES_LONG_SIZE;
    fn from_slice(buffer: &[u8]) -> WifiReadLongResultsRsp {
        WifiReadLongResultsRsp::from_slice(buffer)
    }

}

impl ResultFromSlice<WifiReadExtendedResultsRsp> for WifiReadExtendedResultsRsp {
    const SIZE : u8 = WIFI_RES_EXT_SIZE;
    fn from_slice(buffer: &[u8]) -> WifiReadExtendedResultsRsp {
        WifiReadExtendedResultsRsp::from_slice(buffer)
    }
}

impl ResultFromSlice<WifiReadCountryCodeResultsRsp> for WifiReadCountryCodeResultsRsp {
    const SIZE : u8 = WIFI_RES_COUNTRY_SIZE;
    fn from_slice(buffer: &[u8]) -> WifiReadCountryCodeResultsRsp {
        WifiReadCountryCodeResultsRsp::from_slice(buffer)
    }
}

/// Struct to iter over Wifi results, yielding own copy
struct WifiResultsIter<'a, T> {
    marker: PhantomData<T>,
    buffer: &'a[u8],
    index: usize,
    max: usize,
}

impl<'a, T: ResultFromSlice<T> > WifiResultsIter<'a, T> {
    fn new(buffer: &'a [u8], nb: u8) -> Self {
        WifiResultsIter {
            marker: PhantomData,
            buffer,
            index: 0,
            max: nb as usize * T::SIZE as usize
        }
    }
}

impl<'a,T: ResultFromSlice<T>> Iterator for WifiResultsIter<'a,T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index>=self.max {
            return None;
        }
        let next = self.index + T::SIZE as usize;
        let v = T::from_slice(&self.buffer[self.index..next]);
        self.index = next;
        Some(v)
    }
}


impl<O,SPI, M> Lr1120<O,SPI, M> where
    O: OutputPin, SPI: SpiBus<u8>, M: BusyPin
{

    /// Capture WiFi packet
    /// Busy stays high during scan and interrupt WifiScanDone is raised when finished.
    pub async fn wifi_scan(&mut self, params: &WifiScanParams) -> Result<(), Lr1120Error> {
        let req = wifi_scan_cmd(params.standard, params.chan_mask, params.acq_mode, params.max_res, params.max_scan, params.timeout, params.abort_on_timeout);
        self.cmd_wr(&req).await
    }

    /// Capture WiFi packet with a time limit per channel in ms
    pub async fn wifi_scan_time_limit(&mut self, params: &WifiScanParams, time_limit: u16) -> Result<(), Lr1120Error> {
        let req = wifi_scan_time_limit_cmd(params.standard, params.chan_mask, params.acq_mode, params.max_res, time_limit, params.timeout);
        self.cmd_wr(&req).await
    }

    /// Capture WiFi-B beacon and look for `max_res` country code
    pub async fn wifi_scan_country_code(&mut self, params: &WifiScanParams) -> Result<(), Lr1120Error> {
        let req = wifi_country_code_cmd(params.chan_mask, params.max_res, params.max_scan, params.timeout, params.abort_on_timeout);
        self.cmd_wr(&req).await
    }

    /// Capture WiFi-B beacon and look for `max_res` country code
    pub async fn wifi_scan_country_code_time_limit(&mut self, params: &WifiScanParams, time_limit: u16) -> Result<(), Lr1120Error> {
        let req = wifi_country_code_time_limit_cmd(params.chan_mask, params.max_res, time_limit, params.timeout);
        self.cmd_wr(&req).await
    }

    /// Get number of results from last scan
    pub async fn wifi_get_nb_res(&mut self) -> Result<u8, Lr1120Error> {
        let req = wifi_get_nb_results_req();
        let mut rsp = WifiGetNbResultsRsp::new();
        self.cmd_rd(&req, rsp.as_mut()).await?;
        Ok(rsp.nb_results())
    }

    /// Get number of country code from last scan
    pub async fn wifi_get_nb_country_code(&mut self) -> Result<u8, Lr1120Error> {
        let req = wifi_get_nb_country_code_results_req();
        let mut rsp = WifiGetNbCountryCodeResultsRsp::new();
        self.cmd_rd(&req, rsp.as_mut()).await?;
        Ok(rsp.nb_results())
    }

    /// Reset cumulative timings
    pub async fn wifi_reset_timings(&mut self) -> Result<(), Lr1120Error> {
        let req = wifi_reset_cumul_timings_cmd();
        self.cmd_wr(&req).await
    }

    /// Get scanning cumulative timings
    pub async fn wifi_get_timings(&mut self) -> Result<WifiReadCumulTimingsRsp, Lr1120Error> {
        let req = wifi_read_cumul_timings_req();
        let mut rsp = WifiReadCumulTimingsRsp::new();
        self.cmd_rd(&req, rsp.as_mut()).await?;
        Ok(rsp)
    }

    /// Configure timestamp threshold (in seconds) to discrimante mobile access point from gateways
    pub async fn wifi_set_timestamp_thr(&mut self, threshold: u32) -> Result<(), Lr1120Error> {
        let req = wifi_cfg_timestamp_a_pphone_cmd(threshold);
        self.cmd_wr(&req).await
    }

    /// Return firmware version of wifi-scanning
    pub async fn wifi_get_fw_version(&mut self) -> Result<(u8,u8), Lr1120Error> {
        let req = wifi_read_version_req();
        let mut rsp = WifiReadVersionRsp::new();
        self.cmd_rd(&req, rsp.as_mut()).await?;
        Ok((rsp.version_major(), rsp.version_minor()))
    }

    /// Return short result (9B) of previous Wifi Scanning
    /// with acquisition mode set to BeaconSearch or BeaconPacket
    pub async fn wifi_get_result_short(&mut self, index: u8, nb: u8) -> Result<impl Iterator<Item=WifiReadResultsRsp>, Lr1120Error> {
        let req = wifi_read_results_req(index, nb, WifiResultFormat::Short);
        let nb_byte = nb.min(32) as usize * WIFI_RES_SHORT_SIZE as usize;
        self.cmd_wr(&req).await?;
        self.wait_ready(Duration::from_millis(100)).await?;
        self.rsp_rd(nb_byte).await?;
        let iter : WifiResultsIter<'_, WifiReadResultsRsp> = WifiResultsIter::new(&self.buffer()[..nb_byte],nb);
        Ok(iter)
    }

    /// Return long result 229B) of previous Wifi Scanning
    /// with acquisition mode set to BeaconSearch or BeaconPacket
    pub async fn wifi_get_result_long(&mut self, index: u8, nb: u8) -> Result<impl Iterator<Item=WifiReadLongResultsRsp>, Lr1120Error> {
        let req = wifi_read_results_req(index, nb, WifiResultFormat::Long);
        let nb_byte = nb.min(32) as usize * WIFI_RES_LONG_SIZE as usize;
        self.cmd_wr(&req).await?;
        self.wait_ready(Duration::from_millis(100)).await?;
        self.rsp_rd(nb_byte).await?;
        let iter : WifiResultsIter<'_, WifiReadLongResultsRsp> = WifiResultsIter::new(&self.buffer()[..nb_byte],nb);
        Ok(iter)
    }

    /// Return extended result (79B) of previous Wifi Scanning
    /// with acquisition mode set to BeaconSearch or BeaconPacket
    pub async fn wifi_get_result_ext(&mut self, index: u8, nb: u8) -> Result<impl Iterator<Item=WifiReadExtendedResultsRsp>, Lr1120Error> {
        let req = wifi_read_results_req(index, nb, WifiResultFormat::Long);
        let nb_byte = nb.min(12) as usize * WIFI_RES_EXT_SIZE as usize;
        self.cmd_wr(&req).await?;
        self.wait_ready(Duration::from_millis(100)).await?;
        self.rsp_rd(nb_byte).await?;
        let iter : WifiResultsIter<'_, WifiReadExtendedResultsRsp> = WifiResultsIter::new(&self.buffer()[..nb_byte],nb);
        Ok(iter)
    }

    /// Return country code result (10B) of previous Wifi Scanning Country Code
    pub async fn wifi_get_result_country(&mut self, index: u8, nb: u8) -> Result<impl Iterator<Item=WifiReadCountryCodeResultsRsp>, Lr1120Error> {
        let req = wifi_read_country_code_results_req(index, nb);
        let nb_byte = nb.min(32) as usize * WIFI_RES_COUNTRY_SIZE as usize;
        self.cmd_wr(&req).await?;
        self.wait_ready(Duration::from_millis(100)).await?;
        self.rsp_rd(nb_byte).await?;
        let iter : WifiResultsIter<'_, WifiReadCountryCodeResultsRsp> = WifiResultsIter::new(&self.buffer()[..nb_byte],nb);
        Ok(iter)
    }

}