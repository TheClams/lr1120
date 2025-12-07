//! # Radio control and RF management API
//!
//! This module provides APIs for controlling the LR1120 chip's radio functionality, including RF channel 
//! configuration, power amplifier setup, antenna signal measurement, and transmission/reception control.
//! These are the core radio functions needed to configure the RF characteristics and manage wireless
//! communication across all supported protocols.
//!
//! ## Available Methods
//!
//! ### RF Configuration
//! - [`set_rf`](Lr1120::set_rf) - Set RF frequency channel in Hz
//! - [`set_packet_type`](Lr1120::set_packet_type) - Set packet type (LoRa, FSK)
//!
//! ### Power Amplifier Configuration
//! - [`set_tx_params`](Lr1120::set_tx_params) - Set TX power level and ramp time
//! - [`set_pa`](Lr1120::set_pa) - Configure Power Amplifier (LF/HF) with duty cycle
//!
//! ### Operation Mode Control
//! - [`set_fallback`](Lr1120::set_fallback) - Set fallback mode after TX/RX completion
//! - [`set_tx`](Lr1120::set_tx) - Enter transmission mode with timeout
//! - [`set_tx_cw`](Lr1120::set_tx_cw) - Start TX in continuous wave test mode
//! - [`set_rx`](Lr1120::set_rx) - Enter reception mode with timeout and ready wait option
//! - [`set_rx_continous`](Lr1120::set_rx_continous) - Start RX in continuous mode
//! - [`set_rx_duty_cycle`](Lr1120::set_rx_duty_cycle) - Start periodic RX
//!
//! ### Gain and Signal Control
//! - [`get_rssi_inst`](Lr1120::get_rssi_inst) - Get instantaneous RSSI measurement
//! - [`get_rssi_avg`](Lr1120::get_rssi_avg) - Get average RSSI measurement over specified duration
//!
//! ### Reception Management
//! - [`clear_rx_stats`](Lr1120::clear_rx_stats) - Clear reception statistics
//! - [`get_rx_buffer_status`](Lr1120::get_rx_buffer_status) - Get RX buffer status (packet length and pointer)
//!
//! ### Timing
//! - [`set_stop_timeout`](Lr1120::set_stop_timeout) - Set whether the RX timeout stops when preamble is detected or when the synchronization is confirmed
//!


use embassy_time::Duration;
use embedded_hal::digital::OutputPin;
use embedded_hal_async::spi::SpiBus;

pub use super::cmd::cmd_radio::*;
use super::{BusyPin, Lr1120, Lr1120Error};

impl<O,SPI, M> Lr1120<O,SPI, M> where
    O: OutputPin, SPI: SpiBus<u8>, M: BusyPin
{

    /// Set the RF channel (in Hz)
    pub async fn set_rf(&mut self, freq: u32) -> Result<(), Lr1120Error> {
        let req = set_rf_frequency_cmd(freq);
        self.cmd_wr(&req).await
    }

    /// Set the packet type
    pub async fn set_packet_type(&mut self, packet_type: PacketType) -> Result<(), Lr1120Error> {
        let req = set_packet_type_cmd(packet_type);
        self.cmd_wr(&req).await
    }

    /// Set Tx power and ramp time
    /// TX Power in given in half-dB unit. Range is -19..44 for LF Path and -39..24 for HF path
    /// Ramp-time is important to reduce Out-of-band emission. A safe rule of thumb is to set it to around 4/Bandwidth.
    pub async fn set_tx_params(&mut self, tx_power: i8, ramp_time: RampTime) -> Result<(), Lr1120Error> {
        let req = set_tx_params_cmd(tx_power, ramp_time);
        self.cmd_wr(&req).await
    }

    /// Configure Power PA
    pub async fn set_pa(&mut self, pa_sel: PaSel, duty_cycle: u8) -> Result<(), Lr1120Error> {
        let pa_supply = if pa_sel==PaSel::HpPa {RegPaSupply::Vbat} else {RegPaSupply::Vreg};
        let req = set_pa_config_cmd(pa_sel, pa_supply, duty_cycle, 7);
        self.cmd_wr(&req).await
    }

    /// Set the Fallback mode after TX/RX
    pub async fn set_fallback(&mut self, fallback_mode: FallbackMode) -> Result<(), Lr1120Error> {
        let req = set_rx_tx_fallback_mode_cmd(fallback_mode);
        self.cmd_wr(&req).await
    }

    /// Set chip in TX mode. Set timeout to 0 or to a value longer than the packet duration.
    /// Timeout is given in LF clock step (1/32.768kHz ~ 30.5us)
    pub async fn set_tx(&mut self, tx_timeout: u32) -> Result<(), Lr1120Error> {
        let req = set_tx_cmd(tx_timeout);
        self.cmd_wr(&req).await
    }

    /// Start TX in test continuous wave
    pub async fn set_tx_cw(&mut self) -> Result<(), Lr1120Error> {
        let req = set_tx_cw_cmd();
        self.cmd_wr(&req).await
    }

    /// Set chip in RX mode. A timeout equal to 0 means a single reception, the value 0xFFFFFF is for continuous RX (i.e. always restart reception)
    /// and any other value, the chip will go back to its fallback mode if a reception does not occur before the timeout is elapsed
    /// Timeout is given in LF clock step (1/32.768kHz ~ 30.5us)
    pub async fn set_rx(&mut self, rx_timeout: u32, wait_ready: bool) -> Result<(), Lr1120Error> {
        let req = set_rx_cmd(rx_timeout);
        self.cmd_wr(&req).await?;
        if wait_ready {
            self.wait_ready(Duration::from_millis(100)).await?;
        }
        Ok(())
    }

    /// Set RX in continuous mode
    pub async fn set_rx_continous(&mut self) -> Result<(), Lr1120Error> {
        self.set_rx(0xFFFFFF,true).await
    }

    /// Start periodic RX
    /// Radio listens for `rx_max_time`: go to sleep once packet is received or no packet was detect
    /// Repeat operation every `cycle_time` (which must be bigger than rx_max_time)
    /// The `use_lora_cad` is only valid if packet type was set to LoRa and performs a CAD instead of a standard reception.
    /// In this case the exit mode of the CAD is performed, i.e. it can start a TX if configured as Listen-Before-Talk
    pub async fn set_rx_duty_cycle(&mut self, listen_time: u32, cycle_time: u32, use_lora_cad: bool) -> Result<(), Lr1120Error> {
        let req = set_rx_duty_cycle_cmd(listen_time, cycle_time, use_lora_cad);
        self.cmd_wr(&req).await
    }

    /// Clear RX stats
    pub async fn clear_rx_stats(&mut self) -> Result<(), Lr1120Error> {
        self.cmd_wr(&reset_stats_cmd()).await
    }

    /// Return length of last packet received
    pub async fn get_rx_buffer_status(&mut self) -> Result<RxBufferStatusRsp, Lr1120Error> {
        let req = get_rx_buffer_status_req();
        let mut rsp = RxBufferStatusRsp::new();
        self.cmd_rd(&req, rsp.as_mut()).await?;
        Ok(rsp)
    }

    /// Measure RSSI instantaneous
    pub async fn get_rssi_inst(&mut self) -> Result<u8, Lr1120Error> {
        let req = get_rssi_inst_req();
        let mut rsp = RssiInstRsp::new();
        self.cmd_rd(&req, rsp.as_mut()).await?;
        Ok(rsp.rssi())
    }

    /// Measure an average RSSI (in -0.5dBm)
    /// Average is the result of n instantaneous RSSI measurement
    pub async fn get_rssi_avg(&mut self, nb_meas: u16) -> Result<u8, Lr1120Error> {
        let mut rssi = 0;
        for _ in 0..nb_meas {
            rssi += self.get_rssi_inst().await? as u16;
        }
        let avg = (rssi + (nb_meas>>1)) / nb_meas;
        Ok(avg as u8)
    }

    /// Set whether the RX timeout stops when preamble is detected or when the synchronization is confirmed (Default)
    pub async fn set_stop_timeout(&mut self, on_preamble: bool) -> Result<(), Lr1120Error> {
        let req = stop_timeout_on_preamble_cmd(on_preamble);
        self.cmd_wr(&req).await
    }

}
