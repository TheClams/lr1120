// Crypto commands API

use crate::status::Status;

/// Key identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum KeyId {
    Nwk = 2,
    App = 3,
    JsEnc = 4,
    JsInt = 5,
    GpKe0 = 6,
    GpKe1 = 7,
    GpKe2 = 8,
    GpKe3 = 9,
    GpKe4 = 10,
    GpKe5 = 11,
    AppS = 12,
    FNwkSInt = 13,
    SNwkSInt = 14,
    NwkSEnc = 15,
    Rfu0 = 16,
    Rfu1 = 17,
    McAppS0 = 18,
    McAppS1 = 19,
    McAppS2 = 20,
    McAppS3 = 21,
    McNwkS0 = 22,
    McNwkS1 = 23,
    McNwkS2 = 24,
    McNwkS3 = 25,
    Gp0 = 26,
    Gp1 = 27,
}

impl KeyId {
    /// True when key is Network or application
    pub fn is_core(&self) -> bool {
        matches!(self, KeyId::Nwk|KeyId::App)
    }

    /// True when key is Lifetime
    pub fn is_lifetime(&self) -> bool {
        matches!(self, KeyId::JsEnc|KeyId::JsInt)
    }

    /// True when key is multicast
    pub fn is_multicast(&self) -> bool {
        matches!(self, KeyId::McAppS0|KeyId::McAppS1|KeyId::McAppS2|KeyId::McAppS3|KeyId::McNwkS0|KeyId::McNwkS1|KeyId::McNwkS2|KeyId::McNwkS3)
    }

    /// True when key is general purpose transport
    pub fn is_gp_transport(&self) -> bool {
        matches!(self, KeyId::GpKe0|KeyId::GpKe1|KeyId::GpKe2|KeyId::GpKe3|KeyId::GpKe4|KeyId::GpKe5)
    }

    /// True when key is unicast
    pub fn is_unicast(&self) -> bool {
        matches!(self, KeyId::AppS|KeyId::FNwkSInt|KeyId::SNwkSInt|KeyId::NwkSEnc|KeyId::Rfu0|KeyId::Rfu1)
    }

    /// True when key is general purpose
    pub fn is_gp(&self) -> bool {
        matches!(self, KeyId::Gp0|KeyId::Gp1)
    }
}

/// Crypto Engine status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum CeStatus {
    Success = 0,
    FailCmac = 1,
    InvKeyId = 3,
    BufSize = 5,
    Error = 6,
}

impl From<u8> for CeStatus {
    fn from(value: u8) -> Self {
        match value {
            6 => CeStatus::Error,
            5 => CeStatus::BufSize,
            3 => CeStatus::InvKeyId,
            1 => CeStatus::FailCmac,
            _ => CeStatus::Success,
        }
    }
}

/// Determines Header size: - LoRaWAN 1.0: 1 byte MHDR - LoRaWAN 1.1: 12 bytes with SIntKey, JoinReqType, JoinEUI, DevNonce, MHDR
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum LorawanVersion {
    V1p0 = 0,
    V1p1 = 1,
}

/// Sets a specific Key identified by KeyID into Crypto Engine. Key is 16-byte AES-128 key as defined in FIPS-197.
pub fn crypto_set_key_req(key_id: KeyId, key: u128) -> [u8; 19] {
    let mut cmd = [0u8; 19];
    cmd[0] = 0x05;
    cmd[1] = 0x02;

    cmd[2] |= key_id as u8;
    cmd[3] |= ((key >> 120) & 0xFF) as u8;
    cmd[4] |= ((key >> 112) & 0xFF) as u8;
    cmd[5] |= ((key >> 104) & 0xFF) as u8;
    cmd[6] |= ((key >> 96) & 0xFF) as u8;
    cmd[7] |= ((key >> 88) & 0xFF) as u8;
    cmd[8] |= ((key >> 80) & 0xFF) as u8;
    cmd[9] |= ((key >> 72) & 0xFF) as u8;
    cmd[10] |= ((key >> 64) & 0xFF) as u8;
    cmd[11] |= ((key >> 56) & 0xFF) as u8;
    cmd[12] |= ((key >> 48) & 0xFF) as u8;
    cmd[13] |= ((key >> 40) & 0xFF) as u8;
    cmd[14] |= ((key >> 32) & 0xFF) as u8;
    cmd[15] |= ((key >> 24) & 0xFF) as u8;
    cmd[16] |= ((key >> 16) & 0xFF) as u8;
    cmd[17] |= ((key >> 8) & 0xFF) as u8;
    cmd[18] |= (key & 0xFF) as u8;
    cmd
}

/// Derives (encrypts) input value into destination Key using source Key. Generated key stored in Crypto Engine RAM - use CryptoStoreToFlash to persist. See LoRaWAN key derivation schemes in Ch 16.3-16.4.
pub fn crypto_derive_key_req(src_key_id: KeyId, dst_key_id: KeyId, input: u128) -> [u8; 20] {
    let mut cmd = [0u8; 20];
    cmd[0] = 0x05;
    cmd[1] = 0x03;

    cmd[2] |= src_key_id as u8;
    cmd[3] |= dst_key_id as u8;
    cmd[4] |= ((input >> 120) & 0xFF) as u8;
    cmd[5] |= ((input >> 112) & 0xFF) as u8;
    cmd[6] |= ((input >> 104) & 0xFF) as u8;
    cmd[7] |= ((input >> 96) & 0xFF) as u8;
    cmd[8] |= ((input >> 88) & 0xFF) as u8;
    cmd[9] |= ((input >> 80) & 0xFF) as u8;
    cmd[10] |= ((input >> 72) & 0xFF) as u8;
    cmd[11] |= ((input >> 64) & 0xFF) as u8;
    cmd[12] |= ((input >> 56) & 0xFF) as u8;
    cmd[13] |= ((input >> 48) & 0xFF) as u8;
    cmd[14] |= ((input >> 40) & 0xFF) as u8;
    cmd[15] |= ((input >> 32) & 0xFF) as u8;
    cmd[16] |= ((input >> 24) & 0xFF) as u8;
    cmd[17] |= ((input >> 16) & 0xFF) as u8;
    cmd[18] |= ((input >> 8) & 0xFF) as u8;
    cmd[19] |= (input & 0xFF) as u8;
    cmd
}

/// Decrypts join accept message (using AES-ECB encrypt per LoRaWAN spec) on Data and Header, then verifies MIC. Returns decrypted data if MIC verification successful.
pub fn crypto_process_join_accept_req(dec_key_id: KeyId, ver_key_id: KeyId, lorawan_version: LorawanVersion) -> [u8; 5] {
    let mut cmd = [0u8; 5];
    cmd[0] = 0x05;
    cmd[1] = 0x04;

    cmd[2] |= dec_key_id as u8;
    cmd[3] |= ver_key_id as u8;
    cmd[4] |= lorawan_version as u8;
    cmd
}

/// Computes AES CMAC of provided data using specified Key and returns MIC (first 4 bytes of CMAC). Maximum data size 256 bytes.
pub fn crypto_compute_aes_cmac_req(key_id: KeyId) -> [u8; 3] {
    let mut cmd = [0u8; 3];
    cmd[0] = 0x05;
    cmd[1] = 0x05;

    cmd[2] |= key_id as u8;
    cmd
}

/// Computes AES CMAC of provided data using specified Key and compares with provided MIC. Returns SUCCESS if MICs match, FAIL_CMAC otherwise. Maximum data size 256 bytes.
pub fn crypto_verify_aes_cmac_req(key_id: KeyId, expected_mic: u32) -> [u8; 7] {
    let mut cmd = [0u8; 7];
    cmd[0] = 0x05;
    cmd[1] = 0x06;

    cmd[2] |= key_id as u8;
    cmd[3] |= ((expected_mic >> 24) & 0xFF) as u8;
    cmd[4] |= ((expected_mic >> 16) & 0xFF) as u8;
    cmd[5] |= ((expected_mic >> 8) & 0xFF) as u8;
    cmd[6] |= (expected_mic & 0xFF) as u8;
    cmd
}

/// Encrypts provided data using specified Key and returns encrypted data. Cannot be used on key indexes 2-11 (prevents re-calculating session keys). For LoRaWAN encryption operations.
pub fn crypto_aes_encrypt01_req(key_id: KeyId) -> [u8; 3] {
    let mut cmd = [0u8; 3];
    cmd[0] = 0x05;
    cmd[1] = 0x07;

    cmd[2] |= key_id as u8;
    cmd
}

/// Encrypts provided data using specified Key and returns encrypted data. For generic non-LoRaWAN operations using Crypto Engine as hardware accelerator. Only for General Purpose keys (26-27).
pub fn crypto_aes_encrypt_req(key_id: KeyId) -> [u8; 3] {
    let mut cmd = [0u8; 3];
    cmd[0] = 0x05;
    cmd[1] = 0x08;

    cmd[2] |= key_id as u8;
    cmd
}

/// Decrypts provided data using specified Key and returns decrypted data. For non-LoRaWAN security tasks using Crypto Engine as standalone hardware accelerator.
pub fn crypto_aes_decrypt_req(key_id: KeyId) -> [u8; 3] {
    let mut cmd = [0u8; 3];
    cmd[0] = 0x05;
    cmd[1] = 0x09;

    cmd[2] |= key_id as u8;
    cmd
}

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

/// Adds a chunk to encrypted firmware image to be checked. Call multiple times until complete image sent. Takes max 64x32-bit words (256 bytes) per call, except last segment may be shorter. BUSY released when ready for next chunk. Call CryptoCheckEncryptedFirmwareImageResult to get final result.
pub fn crypto_check_encrypted_firmware_image_cmd(offset: u32) -> [u8; 6] {
    let mut cmd = [0u8; 6];
    cmd[0] = 0x05;
    cmd[1] = 0x0F;

    cmd[2] |= ((offset >> 24) & 0xFF) as u8;
    cmd[3] |= ((offset >> 16) & 0xFF) as u8;
    cmd[4] |= ((offset >> 8) & 0xFF) as u8;
    cmd[5] |= (offset & 0xFF) as u8;
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

    /// Crypto Engine status
    pub fn ce_status(&self) -> CeStatus {
        self.0[1].into()
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

    /// Crypto Engine status
    pub fn ce_status(&self) -> CeStatus {
        self.0[1].into()
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

    /// Crypto Engine status
    pub fn ce_status(&self) -> CeStatus {
        self.0[1].into()
    }
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

    /// Crypto Engine status
    pub fn ce_status(&self) -> CeStatus {
        self.0[1].into()
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

    /// Crypto Engine status
    pub fn ce_status(&self) -> CeStatus {
        self.0[1].into()
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

    /// Crypto Engine status
    pub fn ce_status(&self) -> CeStatus {
        self.0[1].into()
    }
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

    /// Crypto Engine status
    pub fn ce_status(&self) -> CeStatus {
        self.0[1].into()
    }
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

    /// Crypto Engine status
    pub fn ce_status(&self) -> CeStatus {
        self.0[1].into()
    }
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

    /// Crypto Engine status
    pub fn ce_status(&self) -> CeStatus {
        self.0[1].into()
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

    /// Crypto Engine status
    pub fn ce_status(&self) -> CeStatus {
        self.0[1].into()
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

    /// Crypto Engine status
    pub fn ce_status(&self) -> CeStatus {
        self.0[1].into()
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

    /// Crypto Engine status
    pub fn ce_status(&self) -> CeStatus {
        self.0[1].into()
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
