// System commands API

use crate::status::{Status,Intr};

/// Device type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum HwType {
    Lr1110 = 1,
    Lr1120 = 2,
    Lr1121 = 3,
    Bootloader = 223,
}

impl From<u8> for HwType {
    fn from(value: u8) -> Self {
        match value {
            223 => HwType::Bootloader,
            3 => HwType::Lr1121,
            2 => HwType::Lr1120,
            _ => HwType::Lr1110,
        }
    }
}

/// LF (32.768kHz) clock source selection (RC, XTal, External clock on DIO11)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum LfClock {
    Rc = 0,
    Xtal = 1,
    Dio11 = 2,
}

/// TCXO supply voltage on VTCXO pin. Precision typically +/-50mV for 1.8V setting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum TcxoVoltage {
    Tcxo1v6 = 0,
    Tcxo1v7 = 1,
    Tcxo1v8 = 2,
    Tcxo2v2 = 3,
    Tcxo2v4 = 4,
    Tcxo2v7 = 5,
    Tcxo3v0 = 6,
    Tcxo3v3 = 7,
}

/// 0: Performs software restart, 3: Bootloader does not execute firmware but allows firmware upgrades, other values RFU
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum StayInBootloader {
    SoftwareRestart = 0,
    BootloaderMode = 3,
}

/// Select clock used in standaby (RC or XOsc)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum StandbyMode {
    Rc = 0,
    Xosc = 1,
}

/// 0: No pull-up or pull-down configured (default), 1: Pull-up or pull-down added on configured RF switch and IRQ DIOs in sleep mode based on DIO state in RC mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Enable {
    Disabled = 0,
    Enabled = 1,
}

/// 0: Use LDO in all modes (default), 1: Automatically switch on DC-DC converter in FS, RX and TX modes, other values RFU
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum RegMode {
    LdoOnly = 0,
    DcdcEnabled = 1,
}

/// Returns status of device including stat1, stat2, and interrupt flags. Clears the stat2 ResetStatus field.
pub fn get_status_req() -> [u8; 2] {
    [0x01, 0x00]
}

/// Returns the pending errors that occurred since the last ClearErrors() or circuit startup
pub fn get_errors_req() -> [u8; 2] {
    [0x01, 0x0D]
}

/// Clears all error flags pending in the device. Error flags cannot be cleared individually.
pub fn clear_errors_cmd() -> [u8; 2] {
    [0x01, 0x0E]
}

/// Reads a block of bytes from the radio RX buffer starting at a specific offset. RX buffer must be implemented as a ring buffer.
pub fn read_buffer8_req(offset: u8, len: u8) -> [u8; 4] {
    let mut cmd = [0u8; 4];
    cmd[0] = 0x01;
    cmd[1] = 0x0A;

    cmd[2] |= offset;
    cmd[3] |= len;
    cmd
}

/// Clears all data in the radio RX buffer by writing '0' over the whole buffer. Mainly used for debug purposes.
pub fn clear_rx_buffer_cmd() -> [u8; 2] {
    [0x01, 0x0B]
}

/// Gets a 32-bit random number. Not for security purposes.
pub fn get_random_number_req() -> [u8; 2] {
    [0x01, 0x20]
}

/// Enables/disables an 8-bit CRC on the SPI interface. CRC uses polynomial 0x65 (reversed reciprocal), initial value 0xFF. This command is always protected by CRC.
pub fn enable_spi_crc_cmd(enable: u8, crc: u8) -> [u8; 4] {
    let mut cmd = [0u8; 4];
    cmd[0] = 0x01;
    cmd[1] = 0x28;

    cmd[2] |= enable;
    cmd[3] |= crc;
    cmd
}

/// Returns the version of the LR1120 hardware and firmware
pub fn get_version_req() -> [u8; 2] {
    [0x01, 0x01]
}

/// Calibrates the requested blocks. Command operates in any mode and returns to Standby RC after completion. Note: PLL_TX calibration required before first Bluetooth Low Energy transmission.
pub fn calibrate_cmd(lf_rc: bool, hf_rc: bool, pll: bool, adc: bool, img: bool, pll_tx: bool) -> [u8; 3] {
    let mut cmd = [0u8; 3];
    cmd[0] = 0x01;
    cmd[1] = 0x0F;

    if lf_rc { cmd[2] |= 1; }
    if hf_rc { cmd[2] |= 2; }
    if pll { cmd[2] |= 4; }
    if adc { cmd[2] |= 8; }
    if img { cmd[2] |= 16; }
    if pll_tx { cmd[2] |= 32; }
    cmd
}

/// Launches image calibration for given frequency range on RFI_N/P_LF sub-GHz path. Frequencies in 4MHz steps. Operates in any mode, returns to Standby RC. Image calibration advised after large temperature variations (>10°C) or frequency changes (>10MHz).
pub fn calib_image_cmd(freq1: u8, freq2: u8) -> [u8; 4] {
    let mut cmd = [0u8; 4];
    cmd[0] = 0x01;
    cmd[1] = 0x11;

    cmd[2] |= freq1;
    cmd[3] |= freq2;
    cmd
}

/// Configures the 32kHz clock source
pub fn config_lf_clock_cmd(lf_clock: LfClock, busy_release: bool) -> [u8; 3] {
    let mut cmd = [0u8; 3];
    cmd[0] = 0x01;
    cmd[1] = 0x16;

    cmd[2] |= (lf_clock as u8) & 0x3;
    if busy_release { cmd[2] |= 4; }
    cmd
}

/// Configures the chip for a connected TCXO. Must be called before GetTemp() if TCXO is used. Only operates in Standby RC mode, otherwise returns CMD_FAIL. Complete chip reset required to return to normal XOSC operation.
pub fn set_tcxo_mode_cmd(tcxo_voltage: TcxoVoltage, delay: u32) -> [u8; 6] {
    let mut cmd = [0u8; 6];
    cmd[0] = 0x01;
    cmd[1] = 0x17;

    cmd[2] |= tcxo_voltage as u8;
    cmd[3] |= ((delay >> 16) & 0xFF) as u8;
    cmd[4] |= ((delay >> 8) & 0xFF) as u8;
    cmd[5] |= (delay & 0xFF) as u8;
    cmd
}

/// Triggers a restart of the LR1120 firmware. 32kHz clock configuration is retained.
pub fn reboot_cmd(stay_in_bootloader: StayInBootloader) -> [u8; 3] {
    let mut cmd = [0u8; 3];
    cmd[0] = 0x01;
    cmd[1] = 0x18;

    cmd[2] |= stay_in_bootloader as u8;
    cmd
}

/// Puts device in Power Down or Sleep mode with optional automatic wake-up. Device exits on NSS falling edge. BUSY=1, all MISO and DIOs in Hi-Z. After wake-up, performs firmware restart and goes to Standby RC when BUSY goes low.
pub fn set_sleep_cmd(wakeup_rtc: bool, ret_en: bool, sleep_time: u32) -> [u8; 7] {
    let mut cmd = [0u8; 7];
    cmd[0] = 0x01;
    cmd[1] = 0x1B;

    if wakeup_rtc { cmd[2] |= 2; }
    if ret_en { cmd[2] |= 1; }
    cmd[3] |= ((sleep_time >> 24) & 0xFF) as u8;
    cmd[4] |= ((sleep_time >> 16) & 0xFF) as u8;
    cmd[5] |= ((sleep_time >> 8) & 0xFF) as u8;
    cmd[6] |= (sleep_time & 0xFF) as u8;
    cmd
}

/// Sets the device in standby mode with chosen 32MHz oscillator. RC is default for all automatic mode transitions. Crystal/TCXO allows faster transitions at expense of higher power.
pub fn set_standby_cmd(standby_mode: StandbyMode) -> [u8; 3] {
    let mut cmd = [0u8; 3];
    cmd[0] = 0x01;
    cmd[1] = 0x1C;

    cmd[2] |= standby_mode as u8;
    cmd
}

/// Sets chip into Frequency Synthesis (FS) mode
pub fn set_fs_cmd() -> [u8; 2] {
    [0x01, 0x1D]
}

/// Configures which interrupt signals should be activated on the DIO9 (IRQ pin 1) and/or DIO11 (IRQ pin 2) interrupt pins
pub fn set_dio_irq_params_cmd(irq1_to_enable: u32, irq2_to_enable: u32) -> [u8; 10] {
    let mut cmd = [0u8; 10];
    cmd[0] = 0x01;
    cmd[1] = 0x13;

    cmd[2] |= ((irq1_to_enable >> 24) & 0xFF) as u8;
    cmd[3] |= ((irq1_to_enable >> 16) & 0xFF) as u8;
    cmd[4] |= ((irq1_to_enable >> 8) & 0xFF) as u8;
    cmd[5] |= (irq1_to_enable & 0xFF) as u8;
    cmd[6] |= ((irq2_to_enable >> 24) & 0xFF) as u8;
    cmd[7] |= ((irq2_to_enable >> 16) & 0xFF) as u8;
    cmd[8] |= ((irq2_to_enable >> 8) & 0xFF) as u8;
    cmd[9] |= (irq2_to_enable & 0xFF) as u8;
    cmd
}

/// Clears the selected interrupt signals by writing a 1 in the respective bit. IrqToClear mapping is identical to IrqToEnable.
pub fn clear_irq_cmd(irq_to_clear: u32) -> [u8; 6] {
    let mut cmd = [0u8; 6];
    cmd[0] = 0x01;
    cmd[1] = 0x14;

    cmd[2] |= ((irq_to_clear >> 24) & 0xFF) as u8;
    cmd[3] |= ((irq_to_clear >> 16) & 0xFF) as u8;
    cmd[4] |= ((irq_to_clear >> 8) & 0xFF) as u8;
    cmd[5] |= (irq_to_clear & 0xFF) as u8;
    cmd
}

#[allow(clippy::too_many_arguments)]
/// Configures DIO5, DIO6, DIO7, DIO8 and DIO10 to control external RF switches or LNAs on the Sub-GHz, GNSS, Wi-Fi and RFIO_HF RF paths. Only works in Standby RC mode, otherwise returns CMD_FAIL. Only lowest 5 bits of all configurations are used.
pub fn set_dio_as_rf_switch_cmd(rfsw_enable: u8, rfsw_stby_cfg: u8, rfsw_rx_cfg: u8, rfsw_tx_cfg: u8, rfsw_tx_hp_cfg: u8, rfsw_tx_hf_cfg: u8, rfsw_gnss_cfg: u8, rfsw_wifi_cfg: u8) -> [u8; 10] {
    let mut cmd = [0u8; 10];
    cmd[0] = 0x01;
    cmd[1] = 0x12;

    cmd[2] |= rfsw_enable;
    cmd[3] |= rfsw_stby_cfg;
    cmd[4] |= rfsw_rx_cfg;
    cmd[5] |= rfsw_tx_cfg;
    cmd[6] |= rfsw_tx_hp_cfg;
    cmd[7] |= rfsw_tx_hf_cfg;
    cmd[8] |= rfsw_gnss_cfg;
    cmd[9] |= rfsw_wifi_cfg;
    cmd
}

/// Enables or disables pull-up/down resistors on configured RF switch and IRQ line DIOs when in sleep mode. Saves power when RF switches are supplied by LR1120 DIOs.
pub fn drive_dios_in_sleep_mode_cmd(enable: Enable) -> [u8; 3] {
    let mut cmd = [0u8; 3];
    cmd[0] = 0x01;
    cmd[1] = 0x2A;

    cmd[2] |= enable as u8;
    cmd
}

/// Returns the temperature measurement from the built-in temperature sensor. Uses XOSC mode, so SetTcxoMode must be called first if TCXO is connected.
pub fn get_temp_req() -> [u8; 2] {
    [0x01, 0x1A]
}

/// Sets whether DC-DC converter is enabled for XOSC, FS, RX or TX modes. Only works in Standby RC mode, otherwise returns CMD_FAIL.
pub fn set_reg_mode_cmd(reg_mode: RegMode) -> [u8; 3] {
    let mut cmd = [0u8; 3];
    cmd[0] = 0x01;
    cmd[1] = 0x10;

    cmd[2] |= reg_mode as u8;
    cmd
}

/// Returns the battery supply voltage (VBAT) as a function of reference voltage. Formula: VBAT (V) = (5 * (Vbat(7:0) / 255)) / (1 - (1 / 1.35V))
pub fn get_vbat_req() -> [u8; 2] {
    [0x01, 0x19]
}

/// Reads LR1120's pre-provisioned ChipEui. Globally-unique number assigned by Semtech in production using Semtech's IEEE assigned EUIs. Stored in persistent memory. Can be used as LoRaWAN DeviceEui or user can provide their own.
pub fn get_chip_eui_req() -> [u8; 2] {
    [0x01, 0x25]
}

/// Reads LR1120's pre-programmed JoinEui installed in production by Semtech. LoRaWAN root keys (AppKey, NwkKey) derived from this JoinEui. In standard use, should be used as LoRaWAN JoinEui field in Join Request frame. Stored in persistent memory.
pub fn get_semtech_join_eui_req() -> [u8; 2] {
    [0x01, 0x26]
}

// Response structs

/// Response for GetStatus command
#[derive(Default)]
pub struct StatusRsp([u8; 6]);

impl StatusRsp {
    /// Create a new response buffer
    pub fn new() -> Self {
        Self::default()
    }

    /// Return Status
    pub fn status(&mut self) -> Status {
        Status::from_array([self.0[0], self.0[1]]) 
    }

    /// Status register with command status, interrupt status, reset source, and chip mode

    /// IRQ status register
    pub fn intr(&self) -> Intr {
        Intr::from_slice(&self.0[2..6])
    }
}

impl AsMut<[u8]> for StatusRsp {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

/// Response for GetErrors command
#[derive(Default)]
pub struct ErrorsRsp([u8; 3]);

impl ErrorsRsp {
    /// Create a new response buffer
    pub fn new() -> Self {
        Self::default()
    }

    /// Return Status
    pub fn status(&mut self) -> Status {
        self.0[0].into()
    }

    /// LF RC calibration error
    pub fn lf_rc_calib_err(&self) -> bool {
        self.0[1] & 0x1 != 0
    }

    /// HF RC calibration error
    pub fn hf_rc_calib_err(&self) -> bool {
        (self.0[1] >> 1) & 0x1 != 0
    }

    /// ADC calibration error
    pub fn adc_calib_err(&self) -> bool {
        (self.0[1] >> 2) & 0x1 != 0
    }

    /// PLL calibration error
    pub fn pll_calib_err(&self) -> bool {
        (self.0[1] >> 3) & 0x1 != 0
    }

    /// Image rejection calibration error
    pub fn img_calib_err(&self) -> bool {
        (self.0[1] >> 4) & 0x1 != 0
    }

    /// HF XOSC start error
    pub fn hf_xosc_start_err(&self) -> bool {
        (self.0[1] >> 5) & 0x1 != 0
    }

    /// LF XOSC start error
    pub fn lf_xosc_start_err(&self) -> bool {
        (self.0[1] >> 6) & 0x1 != 0
    }

    /// PLL lock error
    pub fn pll_lock_err(&self) -> bool {
        (self.0[1] >> 7) & 0x1 != 0
    }

    /// RX ADC offset error
    pub fn rx_adc_offset_err(&self) -> bool {
        self.0[2] & 0x1 != 0
    }
    /// 16 bits value
    pub fn value(&self) -> u16 {
        u16::from_be_bytes([self.0[1], self.0[2]])
    }

    /// Flag when no error are present
    pub fn none(&self) -> bool {
        self.0[1] == 0 && self.0[2] == 0
    }
}

impl AsMut<[u8]> for ErrorsRsp {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}
#[cfg(feature = "defmt")]
impl defmt::Format for ErrorsRsp {
    fn format(&self, f: defmt::Formatter) {
        defmt::write!(f, "Errors: ");
        if self.none() {
            defmt::write!(f, "None");
            return;
        }
        if self.lf_rc_calib()   {defmt::write!(f, "LfRcCalib ")};
        if self.hf_rc_calib()   {defmt::write!(f, "HfRcCalib ")};
        if self.adc_calib()     {defmt::write!(f, "AdcCalib ")};
        if self.pll_calib()     {defmt::write!(f, "PllCalib ")};
        if self.img_calib()     {defmt::write!(f, "ImgCalib ")};
        if self.hf_xosc_start() {defmt::write!(f, "HfXoscStart ")};
        if self.lf_xosc_start() {defmt::write!(f, "LfXoscStart ")};
        if self.pll_lock()      {defmt::write!(f, "PllLock ")};
        if self.rx_adc_offset() {defmt::write!(f, "RxAdcOffset ")};
    }
}

/// Response for ReadBuffer8 command
#[derive(Default)]
pub struct ReadBuffer8Rsp([u8; 2]);

impl ReadBuffer8Rsp {
    /// Create a new response buffer
    pub fn new() -> Self {
        Self::default()
    }

    /// Return Status
    pub fn status(&mut self) -> Status {
        self.0[0].into()
    }
    // TODO: Implement accessor for variable length field 'data'
}

impl AsMut<[u8]> for ReadBuffer8Rsp {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

/// Response for GetRandomNumber command
#[derive(Default)]
pub struct RandomNumberRsp([u8; 5]);

impl RandomNumberRsp {
    /// Create a new response buffer
    pub fn new() -> Self {
        Self::default()
    }

    /// Return Status
    pub fn status(&mut self) -> Status {
        self.0[0].into()
    }

    /// 32-bit random number
    pub fn random_number(&self) -> u32 {
        (self.0[4] as u32) |
        ((self.0[3] as u32) << 8) |
        ((self.0[2] as u32) << 16) |
        ((self.0[1] as u32) << 24)
    }
}

impl AsMut<[u8]> for RandomNumberRsp {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

/// Response for GetVersion command
#[derive(Default)]
pub struct VersionRsp([u8; 5]);

impl VersionRsp {
    /// Create a new response buffer
    pub fn new() -> Self {
        Self::default()
    }

    /// Return Status
    pub fn status(&mut self) -> Status {
        self.0[0].into()
    }

    /// Hardware version of the LR1120
    pub fn hw_version(&self) -> u8 {
        self.0[1]
    }

    /// Device type
    pub fn hw_type(&self) -> HwType {
        self.0[2].into()
    }

    /// Firmware major version number
    pub fn fw_major(&self) -> u8 {
        self.0[3]
    }

    /// Firmware minor version number
    pub fn fw_minor(&self) -> u8 {
        self.0[4]
    }
}

impl AsMut<[u8]> for VersionRsp {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}
#[cfg(feature = "defmt")]
impl defmt::Format for VersionRsp {
    fn format(&self, fmt: defmt::Formatter) {
        defmt::write!(fmt, "{:02x}.{:02x}", self.major(), self.minor());
    }
}

/// Response for GetTemp command
#[derive(Default)]
pub struct TempRsp([u8; 3]);

impl TempRsp {
    /// Create a new response buffer
    pub fn new() -> Self {
        Self::default()
    }

    /// Return Status
    pub fn status(&mut self) -> Status {
        self.0[0].into()
    }

    /// Temperature value. Formula: Temperature (°C) = 25 - (1000 / (-1.7mV/°C)) * ((Temp(10:0) / 2047) * 1.35V - 0.7295V)
    pub fn temp(&self) -> u16 {
        (self.0[2] as u16) |
        ((self.0[1] as u16) << 8)
    }
}

impl AsMut<[u8]> for TempRsp {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

/// Response for GetVbat command
#[derive(Default)]
pub struct VbatRsp([u8; 2]);

impl VbatRsp {
    /// Create a new response buffer
    pub fn new() -> Self {
        Self::default()
    }

    /// Return Status
    pub fn status(&mut self) -> Status {
        self.0[0].into()
    }

    /// Battery voltage measurement value
    pub fn vbat(&self) -> u8 {
        self.0[1]
    }
}

impl AsMut<[u8]> for VbatRsp {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

/// Response for GetChipEui command
#[derive(Default)]
pub struct ChipEuiRsp([u8; 9]);

impl ChipEuiRsp {
    /// Create a new response buffer
    pub fn new() -> Self {
        Self::default()
    }

    /// Return Status
    pub fn status(&mut self) -> Status {
        self.0[0].into()
    }

    /// 8-byte globally unique ChipEui in big endian format
    pub fn chip_eui(&self) -> u64 {
        (self.0[8] as u64) |
        ((self.0[7] as u64) << 8) |
        ((self.0[6] as u64) << 16) |
        ((self.0[5] as u64) << 24) |
        ((self.0[4] as u64) << 32) |
        ((self.0[3] as u64) << 40) |
        ((self.0[2] as u64) << 48) |
        ((self.0[1] as u64) << 56)
    }
}

impl AsMut<[u8]> for ChipEuiRsp {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

/// Response for GetSemtechJoinEui command
#[derive(Default)]
pub struct SemtechJoinEuiRsp([u8; 9]);

impl SemtechJoinEuiRsp {
    /// Create a new response buffer
    pub fn new() -> Self {
        Self::default()
    }

    /// Return Status
    pub fn status(&mut self) -> Status {
        self.0[0].into()
    }

    /// 8-byte factory JoinEui in big endian format. Re-used across set of Semtech devices.
    pub fn semtech_join_eui(&self) -> u64 {
        (self.0[8] as u64) |
        ((self.0[7] as u64) << 8) |
        ((self.0[6] as u64) << 16) |
        ((self.0[5] as u64) << 24) |
        ((self.0[4] as u64) << 32) |
        ((self.0[3] as u64) << 40) |
        ((self.0[2] as u64) << 48) |
        ((self.0[1] as u64) << 56)
    }
}

impl AsMut<[u8]> for SemtechJoinEuiRsp {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

// Commands with variable length parameters (not implemented):
// - WriteBuffer8
