//! # System control and chip management API
//!
//! This module provides general APIs to control the LR1120 chip, including calibration, interrupts,
//! mode/status management, and low-level register access. These are the core system
//! functions required for chip initialization, operation, and debugging across all communication protocols.
//!
//! ## Available Methods
//!
//! ### Status and Information
//! - [`get_status`](Lr1120::get_status) - Read current chip status and interrupt flags
//! - [`get_errors`](Lr1120::get_errors) - Get detailed error information from the chip
//! - [`get_version`](Lr1120::get_version) - Get chip firmware version information
//! - [`get_chip_eui`](Lr1120::get_chip_eui) - Read Chip EUI
//! - [`get_join_eui`](Lr1120::get_join_eui) - Read Semtech Join EUI
//! - [`clear_irqs`](Lr1120::clear_irqs) - Clear irqs with an optional mask
//!
//! ### Chip Mode and Power Management
//! - [`set_chip_mode`](Lr1120::set_chip_mode) - Set chip operational mode (sleep, standby, FS, TX, RX)
//! - [`set_regulator_mode`](Lr1120::set_regulator_mode) - Choose regulator (LDO or DCDC)
//!
//! ### Calibration
//! - [`calibrate`](Lr1120::calibrate) - Run calibration of different blocks
//! - [`calib_image`](Lr1120::calib_image) - Run front-end image calibration on a frequency band
//!
//! ### Clock Management
//! - [`set_lf_clk`](Lr1120::set_lf_clk) - Configure the LF clock
//! - [`set_tcxo`](Lr1120::set_tcxo) - Configure the chip to use a TCXO
//!
//! ### TX/RX Buffer
//! - [`fn wr_tx_buffer_from`](Lr1120::fn wr_tx_buffer_from) - Write TX data
//! - [`fn wr_tx_buffer`](Lr1120::fn wr_tx_buffer) - Send TX data using internal buffer
//! - [`fn clear_rx_buffer`](Lr1120::fn clear_rx_buffer) - Clear RX Buffer
//! - [`fn rd_rx_buffer_to`](Lr1120::fn rd_rx_buffer_to) - Read data from the RX buffer
//! - [`fn rd_rx_buffer`](Lr1120::fn rd_rx_buffer) - Read data from the LR1120 buffer to the local buffer
//!
//! ### I/O Management
//! - [`set_dio_irq`](Lr1120::set_dio_irq) - Configure a DIO pin for interrupt generation
//! - [`set_dio_rf_switch`](Lr1120::set_dio_rf_switch) - Configure the DIO to control RF switches
//!
//! ### Register and Memory Access
//! - [`rd_reg`](Lr1120::rd_reg) - Read a 32-bit register value
//! - [`wr_reg`](Lr1120::wr_reg) - Write a 32-bit register value
//! - [`wr_reg_mask`](Lr1120::wr_reg_mask) - Write a 32-bit register value with a mask
//! - [`wr_field`](Lr1120::wr_field) - Write to specific bit field in a register
//! - [`rd_mem`](Lr1120::rd_mem) - Read multiple 32-bit words from memory to internal buffer
//!
//! ### Measurements
//! - [`get_temperature`](Lr1120::get_temperature) - Return temperature as voltage measurement (11-bit precision)
//! - [`get_vbat`](Lr1120::get_vbat) - Return the battery voltage
//! - [`get_random_number`](Lr1120::get_random_number) - Return a random number using entropy from PLL and ADC

use embassy_time::Duration;
use embedded_hal::digital::OutputPin;
use embedded_hal_async::spi::SpiBus;

use crate::cmd::cmd_regmem::{read_reg_mem32_req, write_reg_mem32_cmd, write_reg_mem_mask32_cmd, ReadRegMem32Rsp};

use super::{BusyPin, Lr1120, Lr1120Error};
use super::status::{Intr, Status};

pub use super::cmd::cmd_system::*;
use super::radio::{set_rx_cmd, set_tx_cmd};

/// Chip Mode: Sleep/Standby/Fs/...
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum ChipMode {
    /// Set chip in sleep mode without retention: will wakeup on NSS
    DeepSleep,
    /// Set chip in sleep mode with retention: will wakeup on NSS.
    DeepRetention,
    /// Set chip in sleep mode without retention with timeout on 32k clock
    Sleep(u32),
    /// Set chip in sleep mode with retention and timeout on 32k clock
    Retention(u32),
    /// Set Chip in Standby using RC clock
    StandbyRc,
    /// Set Chip in Standby using crystal oscillator.
    StandbyXosc,
    /// Set Chip in Frequency Synthesis: allows immediate TX/RX
    Fs,
    /// Set Chip in Transmit mode
    Tx,
    /// Set Chip in Receive mode
    Rx,
}

/// DIO number (allowed values are 5,6,7,8 or 10)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum DioNum {
    Dio5 = 5,
    Dio6 = 6,
    Dio7 = 7,
    Dio8 = 8,
    Dio10 = 10,
    None = 0
}

impl DioNum {
    /// Create new Dio: 5,6,7,8 or 10
    /// Any other value will return DioNum::None
    pub fn new(num: u8) -> Self {
        num.into()
    }

    /// Return a mask corresponding to the DIO (used by SetDioAsRfSwitch)
    pub fn as_mask(&self) -> u8 {
        match self {
            DioNum::Dio5  => 1,
            DioNum::Dio6  => 2,
            DioNum::Dio7  => 4,
            DioNum::Dio8  => 8,
            DioNum::Dio10 => 16,
            DioNum::None  => 0,
        }
    }
}

impl From<u8> for DioNum {
    fn from(value: u8) -> Self {
        match value {
            5 => DioNum::Dio5,
            6 => DioNum::Dio6,
            7 => DioNum::Dio7,
            8 => DioNum::Dio8,
            10 => DioNum::Dio10,
            _ => DioNum::None,
        }
    }
}



/// Configuration of which RF switch is connected to which DIO
#[derive(Clone, Debug)]
pub struct DioRfSwitchCfg {
    pub tx_lf: DioNum,
    pub tx_hp: DioNum,
    pub tx_hf: DioNum,
    pub rx_lf: DioNum,
    pub rx_mf: DioNum,
    pub rx_hf: DioNum,
}

impl DioRfSwitchCfg {
    /// Create a configuration with only TX/RX RF switch in the low frequency band (Sub-GHz)
    pub fn new_lf(tx: DioNum, rx: DioNum) -> Self {
        Self {
            tx_lf: tx,
            tx_hp: tx,
            tx_hf: DioNum::None,
            rx_lf: rx,
            rx_mf: DioNum::None,
            rx_hf: DioNum::None,
        }
    }

    /// Create a configuration with TX/RX RF switch for both low (Sub-GHz) and high (2.4GHz) frequency band
    pub fn new_lf_hf(tx_lf: DioNum, rx_lf: DioNum, tx_hf: DioNum, rx_hf: DioNum) -> Self {
        Self {
            tx_lf, tx_hp: tx_lf, tx_hf,
            rx_lf, rx_mf: DioNum::None, rx_hf,
        }
    }

    /// Update configuration TX high power switch
    pub fn with_tx_hp(self, tx_hp: DioNum) -> Self {
        Self {
            tx_lf: self.tx_lf, tx_hp, tx_hf: self.tx_hf,
            rx_lf: self.rx_lf, rx_mf: self.rx_mf, rx_hf: self.rx_hf,
        }
    }

    /// Update configuration with RF switch for GNSS band
    pub fn with_gnss(self, rx_mf: DioNum) -> Self {
        Self {
            tx_lf: self.tx_lf, tx_hp: self.tx_hp, tx_hf: self.tx_hf,
            rx_lf: self.rx_lf, rx_mf, rx_hf: self.rx_hf,
        }
    }
}

/// Define a frequency range [min..max] used for image calibration
/// Frequency unit is 4MHz
#[derive(Clone, Copy, Debug)]
pub struct FreqBand {
    min: u8,
    max: u8,
}

impl FreqBand {

    /// Create a manual frequency range
    pub fn new(min: u8, max: u8) -> Self {
        Self{min, max}
    }

    /// Create frequency range for ISM band around 430MHz
    pub fn ism_430() -> Self {
        Self {min: 0x6B, max: 0x6E}
    }

    /// Create frequency range for ISM band around 480MHz
    pub fn ism_480() -> Self {
        Self {min: 0x75, max: 0x81}
    }

    /// Create frequency range for ISM band around 780MHz
    pub fn ism_780() -> Self {
        Self {min: 0xC1, max: 0xC5}
    }

    /// Create frequency range for ISM band around 868MHz
    pub fn ism_868() -> Self {
        Self {min: 0xD7, max: 0xDB}
    }

    /// Create frequency range for ISM band around 430MHz
    pub fn ism_920() -> Self {
        Self {min: 0xE1, max: 0xE9}
    }

}

pub fn pllstep_to_hz(val_step: u32) -> u32 {
    let val_scaled : u64 = (val_step as u64) * 15625;
    (val_scaled >> 14) as u32
}

impl<O,SPI, M> Lr1120<O,SPI, M> where
    O: OutputPin, SPI: SpiBus<u8>, M: BusyPin
{
    /// Read status and interrupt from the chip
    pub async fn get_status(&mut self) -> Result<(Status,Intr), Lr1120Error> {
        let req = get_status_req();
        let mut rsp = StatusRsp::new();
        self.cmd_rd(&req, rsp.as_mut()).await?;
        Ok((rsp.status(), rsp.intr()))
    }

    /// Read status and interrupt from the chip
    pub async fn get_errors(&mut self) -> Result<ErrorsRsp, Lr1120Error> {
        let req = get_errors_req();
        let mut rsp = ErrorsRsp::new();
        self.cmd_rd(&req, rsp.as_mut()).await?;
        Ok(rsp)
    }

    /// Read status and interrupt from the chip
    pub async fn get_version(&mut self) -> Result<VersionRsp, Lr1120Error> {
        let req = get_version_req();
        let mut rsp = VersionRsp::new();
        self.cmd_rd(&req, rsp.as_mut()).await?;
        Ok(rsp)
    }

    /// Read Chip EUI
    pub async fn get_chip_eui(&mut self) -> Result<u64, Lr1120Error> {
        let req = get_chip_eui_req();
        let mut rsp = ChipEuiRsp::new();
        self.cmd_rd(&req, rsp.as_mut()).await?;
        Ok(rsp.chip_eui())
    }

    /// Read Semtech Join EUI
    pub async fn get_join_eui(&mut self) -> Result<u64, Lr1120Error> {
        let req = get_semtech_join_eui_req();
        let mut rsp = SemtechJoinEuiRsp::new();
        self.cmd_rd(&req, rsp.as_mut()).await?;
        Ok(rsp.semtech_join_eui())
    }

    /// Clear irqs with an optional mask.
    /// If no mask is provided, all IRQs are cleared
    pub async fn clear_irqs(&mut self, intr: Option<Intr>) -> Result<(), Lr1120Error> {
        let msk = intr.map(|i| i.value()).unwrap_or(0xFFFFFFFF);
        let req = clear_irq_cmd(msk);
        self.cmd_wr(&req).await
    }

    /// Run calibration of different blocks
    /// Work in any chip mode and on exit the chip goes into Standby RC
    /// Eventual calibration error can be read with get_errors
    pub async fn calibrate(&mut self, lf_rc: bool, hf_rc: bool, pll: bool, adc: bool, img: bool, pll_tx: bool) -> Result<(), Lr1120Error> {
        let req = calibrate_cmd(lf_rc, hf_rc, pll, adc, img, pll_tx);
        self.cmd_wr(&req).await
    }

    /// Run image calibration on a frequency band
    /// Frequency are given as multiple of 4 MHz
    pub async fn calib_image(&mut self, range: FreqBand) -> Result<(), Lr1120Error> {
        let req = calib_image_cmd(range.min, range.max);
        self.cmd_wr(&req).await
    }

    /// Set Tx power and ramp time
    pub async fn set_chip_mode(&mut self, chip_mode: ChipMode) -> Result<(), Lr1120Error> {
        match chip_mode {
            ChipMode::DeepSleep      => self.cmd_wr(&set_sleep_cmd(false, false, 0)).await,
            ChipMode::DeepRetention  => self.cmd_wr(&set_sleep_cmd(false, true, 0)).await,
            ChipMode::Sleep(t)       => self.cmd_wr(&set_sleep_cmd(true, false, t)).await,
            ChipMode::Retention(t)   => self.cmd_wr(&set_sleep_cmd(true, true, t)).await,
            ChipMode::StandbyRc      => self.cmd_wr(&set_standby_cmd(StandbyMode::Rc)).await,
            ChipMode::StandbyXosc    => self.cmd_wr(&set_standby_cmd(StandbyMode::Xosc)).await,
            ChipMode::Fs => self.cmd_wr(&set_fs_cmd()).await,
            ChipMode::Tx => self.cmd_wr(&set_tx_cmd(0)).await,
            ChipMode::Rx => self.cmd_wr(&set_rx_cmd(0xFFFFFF)).await,
        }
    }

    /// Configure regulator (LDO or DCDC)
    /// Shall only be called while in Standby RC
    pub async fn set_regulator_mode(&mut self, dcdc_en: bool) -> Result<(), Lr1120Error> {
        let mode = if dcdc_en {RegMode::DcdcEnabled} else {RegMode::LdoOnly};
        let req = set_reg_mode_cmd(mode);
        self.cmd_wr(&req).await
    }

    /// Configure IRQ for DIO 9 and 11
    pub async fn set_dio_irq(&mut self, irq1: Intr, irq2: Intr) -> Result<(), Lr1120Error> {
        let req = set_dio_irq_params_cmd(irq1.value(), irq2.value());
        self.cmd_wr(&req).await
    }

    /// Configure the DIO to control RF switches
    /// Drive_sleep allow to set up pull-up or pull-down on all enabled RF switches when chip goes into sleep
    pub async  fn set_dio_rf_switch(&mut self, cfg: DioRfSwitchCfg, drive_sleep: bool) -> Result<(), Lr1120Error> {
        let rfsw_tx_cfg    = cfg.tx_lf.as_mask();
        let rfsw_tx_hp_cfg = cfg.tx_hp.as_mask();
        let rfsw_tx_hf_cfg = cfg.tx_hf.as_mask();
        let rfsw_rx_cfg    = cfg.rx_lf.as_mask();
        let rfsw_gnss_cfg  = cfg.rx_mf.as_mask();
        let rfsw_wifi_cfg  = cfg.rx_hf.as_mask();
        let rfsw_enable = rfsw_tx_cfg | rfsw_tx_hp_cfg | rfsw_tx_hf_cfg | rfsw_rx_cfg | rfsw_gnss_cfg | rfsw_wifi_cfg;
        let req = set_dio_as_rf_switch_cmd(rfsw_enable, 0, rfsw_rx_cfg, rfsw_tx_cfg, rfsw_tx_hp_cfg, rfsw_tx_hf_cfg, rfsw_gnss_cfg, rfsw_wifi_cfg);
        self.cmd_wr(&req).await?;
        // Configure pull-up/down for all enabled switch
        let drive_en = if drive_sleep {rfsw_enable} else {0};
        let req = drive_dios_in_sleep_mode_cmd(drive_en);
        self.cmd_wr(&req).await
    }


    /// Configure the LF clock
    pub async fn set_lf_clk(&mut self, sel: LfClock, busy_release: bool) -> Result<(), Lr1120Error> {
        let req = config_lf_clock_cmd(sel, busy_release);
        self.cmd_wr(&req).await
    }

    /// Configure the chip to use a TCXO
    pub async fn set_tcxo(&mut self, volt: TcxoVoltage, start_time: u32) -> Result<(), Lr1120Error> {
        let req = set_tcxo_mode_cmd(volt, start_time);
        self.cmd_wr(&req).await
    }

    /// Return temperature as a voltage measurement (11b precision)
    /// Conversion in degree Celcius is given by 25+1000/1.7*(v/2048*1.35 - 0.7295)
    pub async fn get_temperature(&mut self) -> Result<u16, Lr1120Error> {
        let req = get_temp_req();
        let mut rsp = TempRsp::new();
        self.cmd_rd(&req, rsp.as_mut()).await?;
        Ok(rsp.temp())
    }

    /// Return the battery voltage
    // Conversion in volt is given by 1.35 * (5*v/256 - 1)
    pub async fn get_vbat(&mut self) -> Result<u8, Lr1120Error> {
        let req = get_vbat_req();
        let mut rsp = VbatRsp::new();
        self.cmd_rd(&req, rsp.as_mut()).await?;
        Ok(rsp.vbat())
    }

    /// Return a random number using entropy from PLL and ADC
    pub async fn get_random_number(&mut self) -> Result<u32, Lr1120Error> {
        let req = get_random_number_req();
        let mut rsp = RandomNumberRsp::new();
        self.cmd_rd(&req, rsp.as_mut()).await?;
        Ok(rsp.random_number())
    }

    /// Write TX data
    pub async fn wr_tx_buffer_from(&mut self, buffer: &[u8]) -> Result<(), Lr1120Error> {
        let req = write_buffer8_cmd();
        self.cmd_data_wr(&req, buffer).await
    }

    /// Send TX data using internal buffer
    pub async fn wr_tx_buffer(&mut self, len: usize) -> Result<(), Lr1120Error> {
        let req = write_buffer8_cmd();
        self.cmd_wr_begin(&req).await?;
        self.spi
            .transfer_in_place(&mut self.buffer.data_mut()[..len]).await
            .map_err(|_| Lr1120Error::Spi)?;
        self.nss.set_high().map_err(|_| Lr1120Error::Pin)
    }

    /// Clear RX Buffer
    pub async fn clear_rx_buffer(&mut self) -> Result<(), Lr1120Error> {
        self.cmd_wr(&clear_rx_buffer_cmd()).await
    }

    /// Read data from the RX buffer into a provided buffer
    /// The provided buffer must zeroed contains one extra byte at the beginning for the statud
    pub async fn rd_rx_buffer_to(&mut self, offset: u8, buffer: &mut[u8]) -> Result<(), Lr1120Error> {
        let nb_byte = buffer.len() as u8;
        let req = read_buffer8_cmd(offset, nb_byte);
        self.cmd_rd(&req, buffer).await
    }

    /// Read data from the LR1120 buffer to the local buffer
    pub async fn rd_rx_buffer(&mut self, offset: u8, len: u8) -> Result<(), Lr1120Error> {
        let req = read_buffer8_cmd(offset, len);
        self.cmd_wr(&req).await?;
        self.wait_ready(Duration::from_millis(1)).await?;
        self.rsp_rd(len.into()).await
    }

    /// Read a register value
    pub async fn rd_reg(&mut self, addr: u32) -> Result<u32, Lr1120Error> {
        let req = read_reg_mem32_req(addr, 1);
        let mut rsp = ReadRegMem32Rsp::new();
        self.cmd_rd(&req, rsp.as_mut()).await?;
        Ok(rsp.value())
    }

    /// Read nb32 qword (max 40) from memory and save them inside local buffer
    pub async fn rd_mem(&mut self, addr: u32, nb32: u8) -> Result<(), Lr1120Error> {
        if nb32 > 40 {
            return Err(Lr1120Error::CmdErr);
        }
        let req = read_reg_mem32_req(addr, nb32);
        self.cmd_wr(&req).await?;
        self.wait_ready(Duration::from_millis(1)).await?;
        self.nss.set_low().map_err(|_| Lr1120Error::Pin)?;
        self.buffer.clear(4*nb32 as usize);
        let rsp_buf = &mut self.buffer.0[..4*nb32 as usize];
        self.spi
            .transfer_in_place(rsp_buf).await
            .map_err(|_| Lr1120Error::Spi)?;
        self.nss.set_high().map_err(|_| Lr1120Error::Pin)?;
        self.buffer.cmd_status().check()
    }

    /// Write a register value
    pub async fn wr_reg(&mut self, addr: u32, value: u32) -> Result<(), Lr1120Error> {
        let req = write_reg_mem32_cmd(addr, value);
        self.cmd_wr(&req).await
    }

    /// Write a register value with a mask (only bit where mask is high are changed)
    pub async fn wr_reg_mask(&mut self, addr: u32, mask: u32, value: u32) -> Result<(), Lr1120Error> {
        let req = write_reg_mem_mask32_cmd(addr, mask, value);
        self.cmd_wr(&req).await
    }

    /// Write a field value
    pub async fn wr_field(&mut self, addr: u32, value: u32, pos: u8, width: u8) -> Result<(), Lr1120Error> {
        let mask =
            if width >= 32 {0xFFFFFFFF}
            else { ((1 << width) - 1) << pos };
        let req = write_reg_mem_mask32_cmd(addr, mask, value << pos);
        self.cmd_wr(&req).await
    }

}
