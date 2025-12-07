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
//! - [`clear_irqs`](Lr1120::clear_irqs) - Clear specific interrupt flags
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
//! ### I/O Management
//! - [`set_dio_irq`](Lr1120::set_dio_irq) - Configure a DIO pin for interrupt generation
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

    /// Set the RF channel (in Hz)
    pub async fn clear_irqs(&mut self, intr: Intr) -> Result<(), Lr1120Error> {
        let req = clear_irq_cmd(intr.value());
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
        let mode = if dcdc_en {RegMode::DcdcEnabled} else {RegMode::DcdcEnabled};
        let req = set_reg_mode_cmd(mode);
        self.cmd_wr(&req).await
    }

    /// Configure IRQ for DIO 9 and 11
    pub async fn set_dio_irq(&mut self, irq1: Intr, irq2: Intr) -> Result<(), Lr1120Error> {
        let req = set_dio_irq_params_cmd(irq1.value(), irq2.value());
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
        self.buffer.nop();
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
