// Crypto commands API

use crate::status::Status;
/// Stores all Keys and Parameters from Crypto Engine RAM into flash memory for persistence
pub fn crypto_store_to_flash_req() -> [u8; 2] {
    [0x05, 0x0A]
}

/// Restores all Keys and Parameters from flash memory into Crypto Engine RAM
pub fn crypto_restore_from_flash_req() -> [u8; 2] {
    [0x05, 0x0B]
}

/// Sets a specific Parameter into Crypto Engine RAM
pub fn crypto_set_param_req(param_id: u8, data: u32) -> [u8; 7] {
    let mut cmd = [0u8; 7];
    cmd[0] = 0x05;
    cmd[1] = 0x0D;

    cmd[2] |= param_id;
    cmd[3] |= ((data >> 24) & 0xFF) as u8;
    cmd[4] |= ((data >> 16) & 0xFF) as u8;
    cmd[5] |= ((data >> 8) & 0xFF) as u8;
    cmd[6] |= (data & 0xFF) as u8;
    cmd
}

/// Gets a specific Parameter from Crypto Engine RAM
pub fn crypto_get_param_req(param_id: u8) -> [u8; 3] {
    let mut cmd = [0u8; 3];
    cmd[0] = 0x05;
    cmd[1] = 0x0E;

    cmd[2] |= param_id;
    cmd
}

/// Gets result of encrypted firmware image check after all chunks sent via CryptoCheckEncryptedFirmwareImage
pub fn crypto_check_encrypted_firmware_image_result_req() -> [u8; 2] {
    [0x05, 0x10]
}

// Response structs

/// Response for CryptoSetKey command
#[derive(Default)]
pub struct CryptoSetKeyRsp([u8; 2]);

impl CryptoSetKeyRsp {
    /// Create a new response buffer
    pub fn new() -> Self {
        Self::default()
    }

    /// Return Status
    pub fn status(&mut self) -> Status {
        self.0[0].into()
    }

    /// Crypto Engine status: 0: SUCCESS, 1: FAIL_CMAC, 3: INV_KEY_ID, 5: BUF_SIZE, 6: ERROR
    pub fn ce_status(&self) -> u8 {
        self.0[1]
    }
}

impl AsMut<[u8]> for CryptoSetKeyRsp {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

/// Response for CryptoDeriveKey command
#[derive(Default)]
pub struct CryptoDeriveKeyRsp([u8; 2]);

impl CryptoDeriveKeyRsp {
    /// Create a new response buffer
    pub fn new() -> Self {
        Self::default()
    }

    /// Return Status
    pub fn status(&mut self) -> Status {
        self.0[0].into()
    }

    /// Crypto Engine status: 0: SUCCESS, 1: FAIL_CMAC, 3: INV_KEY_ID, 5: BUF_SIZE, 6: ERROR
    pub fn ce_status(&self) -> u8 {
        self.0[1]
    }
}

impl AsMut<[u8]> for CryptoDeriveKeyRsp {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

/// Response for CryptoProcessJoinAccept command
#[derive(Default)]
pub struct CryptoProcessJoinAcceptRsp([u8; 2]);

impl CryptoProcessJoinAcceptRsp {
    /// Create a new response buffer
    pub fn new() -> Self {
        Self::default()
    }

    /// Return Status
    pub fn status(&mut self) -> Status {
        self.0[0].into()
    }

    /// Crypto Engine status: 0: SUCCESS, 1: FAIL_CMAC, 3: INV_KEY_ID, 5: BUF_SIZE, 6: ERROR
    pub fn ce_status(&self) -> u8 {
        self.0[1]
    }
    // TODO: Implement accessor for variable length field 'data'
}

impl AsMut<[u8]> for CryptoProcessJoinAcceptRsp {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

/// Response for CryptoComputeAesCmac command
#[derive(Default)]
pub struct CryptoComputeAesCmacRsp([u8; 6]);

impl CryptoComputeAesCmacRsp {
    /// Create a new response buffer
    pub fn new() -> Self {
        Self::default()
    }

    /// Return Status
    pub fn status(&mut self) -> Status {
        self.0[0].into()
    }

    /// Crypto Engine status: 0: SUCCESS, 1: FAIL_CMAC, 3: INV_KEY_ID, 5: BUF_SIZE, 6: ERROR
    pub fn ce_status(&self) -> u8 {
        self.0[1]
    }

    /// Message Integrity Check (first 4 bytes of CMAC)
    pub fn mic(&self) -> u32 {
        (self.0[5] as u32) |
        ((self.0[4] as u32) << 8) |
        ((self.0[3] as u32) << 16) |
        ((self.0[2] as u32) << 24)
    }
}

impl AsMut<[u8]> for CryptoComputeAesCmacRsp {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

/// Response for CryptoVerifyAesCmac command
#[derive(Default)]
pub struct CryptoVerifyAesCmacRsp([u8; 2]);

impl CryptoVerifyAesCmacRsp {
    /// Create a new response buffer
    pub fn new() -> Self {
        Self::default()
    }

    /// Return Status
    pub fn status(&mut self) -> Status {
        self.0[0].into()
    }

    /// Crypto Engine status: 0: SUCCESS (MICs match), 1: FAIL_CMAC (MICs differ), 3: INV_KEY_ID, 5: BUF_SIZE, 6: ERROR
    pub fn ce_status(&self) -> u8 {
        self.0[1]
    }
}

impl AsMut<[u8]> for CryptoVerifyAesCmacRsp {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

/// Response for CryptoAesEncrypt01 command
#[derive(Default)]
pub struct CryptoAesEncrypt01Rsp([u8; 2]);

impl CryptoAesEncrypt01Rsp {
    /// Create a new response buffer
    pub fn new() -> Self {
        Self::default()
    }

    /// Return Status
    pub fn status(&mut self) -> Status {
        self.0[0].into()
    }

    /// Crypto Engine status: 0: SUCCESS, 1: FAIL_CMAC, 3: INV_KEY_ID, 5: BUF_SIZE, 6: ERROR
    pub fn ce_status(&self) -> u8 {
        self.0[1]
    }
    // TODO: Implement accessor for variable length field 'encrypted_data'
}

impl AsMut<[u8]> for CryptoAesEncrypt01Rsp {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

/// Response for CryptoAesEncrypt command
#[derive(Default)]
pub struct CryptoAesEncryptRsp([u8; 2]);

impl CryptoAesEncryptRsp {
    /// Create a new response buffer
    pub fn new() -> Self {
        Self::default()
    }

    /// Return Status
    pub fn status(&mut self) -> Status {
        self.0[0].into()
    }

    /// Crypto Engine status: 0: SUCCESS, 1: FAIL_CMAC, 3: INV_KEY_ID, 5: BUF_SIZE, 6: ERROR
    pub fn ce_status(&self) -> u8 {
        self.0[1]
    }
    // TODO: Implement accessor for variable length field 'encrypted_data'
}

impl AsMut<[u8]> for CryptoAesEncryptRsp {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

/// Response for CryptoAesDecrypt command
#[derive(Default)]
pub struct CryptoAesDecryptRsp([u8; 2]);

impl CryptoAesDecryptRsp {
    /// Create a new response buffer
    pub fn new() -> Self {
        Self::default()
    }

    /// Return Status
    pub fn status(&mut self) -> Status {
        self.0[0].into()
    }

    /// Crypto Engine status: 0: SUCCESS, 1: FAIL_CMAC, 3: INV_KEY_ID, 5: BUF_SIZE, 6: ERROR
    pub fn ce_status(&self) -> u8 {
        self.0[1]
    }
    // TODO: Implement accessor for variable length field 'decrypted_data'
}

impl AsMut<[u8]> for CryptoAesDecryptRsp {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

/// Response for CryptoStoreToFlash command
#[derive(Default)]
pub struct CryptoStoreToFlashRsp([u8; 2]);

impl CryptoStoreToFlashRsp {
    /// Create a new response buffer
    pub fn new() -> Self {
        Self::default()
    }

    /// Return Status
    pub fn status(&mut self) -> Status {
        self.0[0].into()
    }

    /// Crypto Engine status: 0: SUCCESS, 1: FAIL_CMAC, 3: INV_KEY_ID, 5: BUF_SIZE, 6: ERROR
    pub fn ce_status(&self) -> u8 {
        self.0[1]
    }
}

impl AsMut<[u8]> for CryptoStoreToFlashRsp {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

/// Response for CryptoRestoreFromFlash command
#[derive(Default)]
pub struct CryptoRestoreFromFlashRsp([u8; 2]);

impl CryptoRestoreFromFlashRsp {
    /// Create a new response buffer
    pub fn new() -> Self {
        Self::default()
    }

    /// Return Status
    pub fn status(&mut self) -> Status {
        self.0[0].into()
    }

    /// Crypto Engine status: 0: SUCCESS, 1: FAIL_CMAC, 3: INV_KEY_ID, 5: BUF_SIZE, 6: ERROR
    pub fn ce_status(&self) -> u8 {
        self.0[1]
    }
}

impl AsMut<[u8]> for CryptoRestoreFromFlashRsp {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

/// Response for CryptoSetParam command
#[derive(Default)]
pub struct CryptoSetParamRsp([u8; 2]);

impl CryptoSetParamRsp {
    /// Create a new response buffer
    pub fn new() -> Self {
        Self::default()
    }

    /// Return Status
    pub fn status(&mut self) -> Status {
        self.0[0].into()
    }

    /// Crypto Engine status: 0: SUCCESS, 1: FAIL_CMAC, 3: INV_KEY_ID, 5: BUF_SIZE, 6: ERROR
    pub fn ce_status(&self) -> u8 {
        self.0[1]
    }
}

impl AsMut<[u8]> for CryptoSetParamRsp {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

/// Response for CryptoGetParam command
#[derive(Default)]
pub struct CryptoGetParamRsp([u8; 6]);

impl CryptoGetParamRsp {
    /// Create a new response buffer
    pub fn new() -> Self {
        Self::default()
    }

    /// Return Status
    pub fn status(&mut self) -> Status {
        self.0[0].into()
    }

    /// Crypto Engine status: 0: SUCCESS, 1: FAIL_CMAC, 3: INV_KEY_ID, 5: BUF_SIZE, 6: ERROR
    pub fn ce_status(&self) -> u8 {
        self.0[1]
    }

    /// Parameter data (32-bit)
    pub fn data(&self) -> u32 {
        (self.0[5] as u32) |
        ((self.0[4] as u32) << 8) |
        ((self.0[3] as u32) << 16) |
        ((self.0[2] as u32) << 24)
    }
}

impl AsMut<[u8]> for CryptoGetParamRsp {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

/// Response for CryptoCheckEncryptedFirmwareImageResult command
#[derive(Default)]
pub struct CryptoCheckEncryptedFirmwareImageResultRsp([u8; 2]);

impl CryptoCheckEncryptedFirmwareImageResultRsp {
    /// Create a new response buffer
    pub fn new() -> Self {
        Self::default()
    }

    /// Return Status
    pub fn status(&mut self) -> Status {
        self.0[0].into()
    }

    /// Firmware verification result: 0: Verification error, 1: Verification success
    pub fn success(&self) -> bool {
        self.0[1] & 0x1 != 0
    }
}

impl AsMut<[u8]> for CryptoCheckEncryptedFirmwareImageResultRsp {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

// Commands with variable length parameters (not implemented):
// - CryptoSetKey
// - CryptoDeriveKey
// - CryptoProcessJoinAccept
// - CryptoComputeAesCmac
// - CryptoVerifyAesCmac
// - CryptoAesEncrypt01
// - CryptoAesEncrypt
// - CryptoAesDecrypt
// - CryptoCheckEncryptedFirmwareImage
