//! # API related to Crypto-Engine
//!
//! This module provides an API for key handling, encryption, decryption and message authentication
//!
//! ## Available Methods
//!
//! ### Key Handling
//! - [`ce_set_key`](Lr1120::ce_set_key) - Set an encryption key
//! - [`ce_derive_key`](Lr1120::ce_derive_key) - Derives input value from the source key into the destination key
//!
//! ### Computation
//! - [`ce_process_join_accept`]::(Lr1120::ce_process_join_accept) - Return decryption status and decrypted payload
//! - [`ce_compute_cmac`]::(Lr1120::ce_compute_cmac) - Compute AES CMAC of the provided data
//! - [`ce_verify_cmac`]::(Lr1120::ce_verify_cmac) - Verify AES CMAC of the provided data
//! - [`ce_encrypt_lorawan`]::(Lr1120::ce_encrypt_lorawan) - Encrypt data for LoRaWAN operation (key limited to unicast/multicast)
//! - [`ce_encrypt`]::(Lr1120::ce_encrypt) - Encrypt data for non-LoRaWAN operation
//! - [`ce_decrypt`]::(Lr1120::ce_decrypt) - Encrypt data for non-LoRaWAN operation
//!
//! ### Utils
//! - [`ce_store_to_flash`](Lr1120::ce_store_to_flash) - Store all keys and parameters from Crypto Engine into falsh memory
//! - [`ce_restore_from_flash`](Lr1120::ce_restore_from_flash) - Read all keys and parameters from falsh memory to Crypto Engine
//! - [`ce_set_param`](Lr1120::ce_set_param) - Set a parameter by ID
//! - [`ce_get_param`](Lr1120::ce_get_param) - Get a parameter by ID
//! - [`ce_check_fw_image`](Lr1120::ce_check_fw_image) - Check if the firmware image is valid
//! - [`ce_fw_image_ok`](Lr1120::ce_fw_image_ok) - Return true if the all previous calls to all chunks of the fimrware image were correct
//!

use embassy_time::Duration;
use embedded_hal::digital::OutputPin;
use embedded_hal_async::spi::SpiBus;

use super::{BusyPin, Lr1120, Lr1120Error};

pub use crate::cmd::cmd_crypto::*;

/// Struct holding result from an encryption/decryption
pub struct CeDataRes<'a> {
    /// Status of the crupto operation (success or fail)
    pub status: CeStatus,
    /// Data after crypto operation
    pub data: &'a [u8],
}

impl<O,SPI, M> Lr1120<O,SPI, M> where
    O: OutputPin, SPI: SpiBus<u8>, M: BusyPin
{

    /// Set an encryption key
    pub async fn ce_set_key(&mut self, id: KeyId, key: u128) -> Result<CeStatus, Lr1120Error> {
        let req = crypto_set_key_req(id, key);
        let mut rsp = CryptoSetKeyRsp::new();
        self.cmd_rd(&req, rsp.as_mut()).await?;
        Ok(rsp.ce_status())
    }

    /// Derives input value from the source key into the destination key
    /// The source key can be either the network, application or any other General Purpose transport key
    /// The destination key can be any key other than the network or application key.
    pub async fn ce_derive_key(&mut self, src: KeyId, dst: KeyId, input: u128) -> Result<CeStatus, Lr1120Error> {
        if !src.is_core() && !src.is_gp_transport() {
            return Err(Lr1120Error::InvalidParam);
        }
        if dst.is_core() {
            return Err(Lr1120Error::InvalidParam);
        }
        let req = crypto_derive_key_req(src, dst, input);
        let mut rsp = CryptoDeriveKeyRsp::new();
        self.cmd_rd(&req, rsp.as_mut()).await?;
        Ok(rsp.ce_status())
    }

    /// Decrypt a join/accept LoRaWAN message using two keys (one for decryption, one for integrity check)
    /// Input data buffer contains header (1 or 12 bytes) followed by 16 or 32 bytes of encrypted payload
    /// Return decryption status and decrypted payload
    pub async fn ce_process_join_accept(&'_ mut self, dec: KeyId, mic: KeyId, lorawan: LorawanVersion, data: &[u8]) -> Result<CeDataRes<'_>, Lr1120Error> {
        // Check input data has a valid length
        let hdr_len = match lorawan {
            LorawanVersion::V1p0 => 1,
            LorawanVersion::V1p1 => 12,
        };
        let rsp_len = data.len().saturating_sub(hdr_len);
        if rsp_len != 16 && rsp_len!=32 {
            return Err(Lr1120Error::InvalidParam);
        }
        let req = crypto_process_join_accept_req(dec, mic, lorawan);
        self.cmd_data_wr(&req, data).await?;
        self.wait_ready(Duration::from_millis(100)).await?;
        self.rsp_rd(rsp_len).await?;
        let status : CeStatus = self.buffer()[0].into();
        let payload = &self.buffer()[1..rsp_len+1];
        Ok(CeDataRes{status, data:payload})
    }

    /// Compute AES CMAC of the provided data
    pub async fn ce_compute_cmac(&mut self, key: KeyId, data: &[u8]) -> Result<CryptoComputeAesCmacRsp, Lr1120Error> {
        if key!=KeyId::Nwk && key!=KeyId::JsInt && !key.is_unicast() {
            return Err(Lr1120Error::InvalidParam);
        }
        let req = crypto_compute_aes_cmac_req(key);
        self.cmd_data_wr(&req, data).await?;
        let mut rsp = CryptoComputeAesCmacRsp::new();
        self.rsp_rd_to(rsp.as_mut()).await?;
        Ok(rsp)
    }

    /// Verify AES CMAC of the provided data
    pub async fn ce_verify_cmac(&mut self, key: KeyId, mic: u32, data: &[u8]) -> Result<CeStatus, Lr1120Error> {
        if key!=KeyId::Nwk && key!=KeyId::JsInt && !key.is_unicast() && !key.is_multicast() {
            return Err(Lr1120Error::InvalidParam);
        }
        let req = crypto_verify_aes_cmac_req(key, mic);
        self.cmd_data_wr(&req, data).await?;
        let mut rsp = CryptoVerifyAesCmacRsp::new();
        self.rsp_rd_to(rsp.as_mut()).await?;
        Ok(rsp.ce_status())
    }

    /// Encrypt data for LoRaWAN operation (key limited to unicast/multicast)
    pub async fn ce_encrypt_lorawan(&'_ mut self, key: KeyId, din: &[u8]) -> Result<CeDataRes<'_>, Lr1120Error> {
        if !key.is_unicast() && !key.is_multicast() {
            return Err(Lr1120Error::InvalidParam);
        }
        let req = crypto_aes_encrypt01_req(key);
        self.cmd_data_wr(&req, din).await?;
        self.rsp_rd(din.len()).await?;
        let status : CeStatus = self.buffer()[0].into();
        let data = &self.buffer()[1..din.len()+1];
        Ok(CeDataRes{status, data})
    }

    /// Encrypt data for non-LoRaWAN operation
    pub async fn ce_encrypt(&'_ mut self, key: KeyId, din: &[u8]) -> Result<CeDataRes<'_>, Lr1120Error> {
        let req = crypto_aes_encrypt_req(key);
        self.cmd_data_wr(&req, din).await?;
        self.rsp_rd(din.len()).await?;
        let status : CeStatus = self.buffer()[0].into();
        let data = &self.buffer()[1..din.len()+1];
        Ok(CeDataRes{status, data})
    }

    /// Encrypt data for non-LoRaWAN operation
    pub async fn ce_decrypt(&'_ mut self, key: KeyId, din: &[u8]) -> Result<CeDataRes<'_>, Lr1120Error> {
        let req = crypto_aes_decrypt_req(key);
        self.cmd_data_wr(&req, din).await?;
        self.rsp_rd(din.len()).await?;
        let status : CeStatus = self.buffer()[0].into();
        let data = &self.buffer()[1..din.len()+1];
        Ok(CeDataRes{status, data})
    }

    /// Store all keys and parameters from Crypto Engine into falsh memory
    pub async fn ce_store_to_flash(&mut self) -> Result<CeStatus, Lr1120Error> {
        let req = crypto_store_to_flash_req();
        let mut rsp = CryptoStoreToFlashRsp::new();
        self.cmd_rd(&req, rsp.as_mut()).await?;
        Ok(rsp.ce_status())
    }

    /// Read all keys and parameters from falsh memory to Crypto Engine
    pub async fn ce_restore_from_flash(&mut self) -> Result<CeStatus, Lr1120Error> {
        let req = crypto_restore_from_flash_req();
        let mut rsp = CryptoRestoreFromFlashRsp::new();
        self.cmd_rd(&req, rsp.as_mut()).await?;
        Ok(rsp.ce_status())
    }

    /// Set a parameter by ID
    pub async fn ce_set_param(&mut self, id:  u8, value: u32) -> Result<CeStatus, Lr1120Error> {
        let req = crypto_set_param_req(id, value);
        let mut rsp = CryptoSetParamRsp::new();
        self.cmd_rd(&req, rsp.as_mut()).await?;
        Ok(rsp.ce_status())
    }

    /// Get a parameter by ID
    pub async fn ce_get_param(&mut self, id:  u8) -> Result<CryptoGetParamRsp, Lr1120Error> {
        let req = crypto_get_param_req(id);
        let mut rsp = CryptoGetParamRsp::new();
        self.cmd_rd(&req, rsp.as_mut()).await?;
        Ok(rsp)
    }

    /// Check if the firmware image is valid
    pub async fn ce_check_fw_image(&mut self, offset: u32, data: &[u8]) -> Result<(), Lr1120Error> {
        let req = crypto_check_encrypted_firmware_image_cmd(offset);
        self.cmd_data_wr(&req, data).await
    }

    /// Return true if the all previous calls to all chunks of the fimrware image were correct
    pub async fn ce_fw_image_ok(&mut self) -> Result<bool, Lr1120Error> {
        let req = crypto_check_encrypted_firmware_image_result_req();
        let mut rsp = CryptoCheckEncryptedFirmwareImageResultRsp::new();
        self.cmd_rd(&req, rsp.as_mut()).await?;
        Ok(rsp.success())
    }

}