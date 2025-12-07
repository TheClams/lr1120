// Fsk commands API


/// Bit rate precision: HIGH indicates 8 fractional bits precision, while BASIC indicates no fractional bits
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Precision {
    Basic = 0,
    High = 1,
}

/// Pulse shape filter
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum PulseShape {
    None = 0,
    Bt0p3 = 8,
    Bt0p5 = 9,
    Bt0p7 = 10,
    Bt1p0 = 11,
    Rc0p7 = 22,
}

/// RX Bandwidth (double side-band)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum RxBw {
    Bw4800 = 31,
    Bw5800 = 23,
    Bw7300 = 15,
    Bw9700 = 30,
    Bw11700 = 22,
    Bw14600 = 14,
    Bw19500 = 29,
    Bw23400 = 21,
    Bw29300 = 13,
    Bw39000 = 28,
    Bw46900 = 20,
    Bw58600 = 12,
    Bw78200 = 27,
    Bw93800 = 19,
    Bw117300 = 11,
    Bw156200 = 26,
    Bw187200 = 18,
    Bw234300 = 10,
    Bw312000 = 25,
    Bw373600 = 17,
    Bw467000 = 9,
}

/// Preamble detector length: 0x00: Off (lock on syncword directly), 0x04: 8 bits, 0x05: 16 bits (recommended), 0x06: 24 bits, 0x07: 32 bits. Must be < SyncWordLen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum PblLenDetect {
    None = 0,
    Len8Bits = 4,
    Len16Bits = 5,
    Len24Bits = 6,
    Len32Bits = 7,
}

/// Address filtering: 0x00: Disabled, 0x01: Enabled on Node address (RX & TX), 0x02: Enabled on Node & Broadcast (RX), Node only (TX). Set addresses with SetGfskAddress. Aborts RX and sets adrsErr if match fails.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum AddrComp {
    Off = 0,
    Node = 1,
    NodeBcast = 2,
}

/// Packet Format: Fixed length, Variable length with 8-bit header (SX126x) or 9-bit header (SX128x)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum FskPktFormat {
    FixedLength = 0,
    Variable8bit = 1,
    Variable9bit = 2,
}

/// 0x01: CRC_OFF, 0x00: CRC_1_BYTE, 0x02: CRC_2_BYTE, 0x04: CRC_1_BYTE_INV, 0x06: CRC_2_BYTE_INV. Configure polynomial/init with SetGfskCrcParams.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Crc {
    CrcOff = 1,
    Crc1Byte = 0,
    Crc2Byte = 2,
    Crc1ByteInv = 4,
    Crc2ByteInv = 6,
}

/// Whitening: 0x00: No encoding, 0x01: SX127x/SX126x/LR11xx compatible, 0x03: SX128x compatible. Configure seed with SetGfskWhitParams.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum DcFree {
    DcFreeOff = 0,
    DcFreeWhitening = 1,
    DcFreeSx128x = 3,
}

/// Configures (G)FSK modulation parameters. Returns CMD_FAIL if packet type is not (G)FSK. Bandwidth must satisfy: (2*Fdev + BR) < Bandwidth. Special register settings required for 0.6kbps and 1.2kbps bit rates (see section 8.4.5).
pub fn set_fsk_modulation_params_cmd(precision: Precision, bitrate: u32, pulse_shape: PulseShape, rx_bw: RxBw, fdev: u32) -> [u8; 12] {
    let mut cmd = [0u8; 12];
    cmd[0] = 0x02;
    cmd[1] = 0x0F;

    cmd[2] |= (precision as u8) & 0x1;
    cmd[2] |= ((bitrate >> 24) & 0xFF) as u8;
    cmd[3] |= ((bitrate >> 16) & 0xFF) as u8;
    cmd[4] |= ((bitrate >> 8) & 0xFF) as u8;
    cmd[5] |= (bitrate & 0xFF) as u8;
    cmd[6] |= pulse_shape as u8;
    cmd[7] |= rx_bw as u8;
    cmd[8] |= ((fdev >> 24) & 0xFF) as u8;
    cmd[9] |= ((fdev >> 16) & 0xFF) as u8;
    cmd[10] |= ((fdev >> 8) & 0xFF) as u8;
    cmd[11] |= (fdev & 0xFF) as u8;
    cmd
}

#[allow(clippy::too_many_arguments)]
/// Configures (G)FSK RF packet parameters. Preamble recommended minimum 16 bits. PblDetect must be < SyncWordLen. For SX128x compatibility: PacketType 0x02, CRC 0/1/2 bytes, SyncWordLen 8/16/24/32/40 bits, AddrComp disabled, DcFree 0x00 or 0x03 only.
pub fn set_fsk_packet_params_cmd(pbl_len_tx: u16, pbl_len_detect: PblLenDetect, sync_word_len: u8, addr_comp: AddrComp, fsk_pkt_format: FskPktFormat, pld_len: u8, crc: Crc, dc_free: DcFree) -> [u8; 11] {
    let mut cmd = [0u8; 11];
    cmd[0] = 0x02;
    cmd[1] = 0x10;

    cmd[2] |= ((pbl_len_tx >> 8) & 0xFF) as u8;
    cmd[3] |= (pbl_len_tx & 0xFF) as u8;
    cmd[4] |= pbl_len_detect as u8;
    cmd[5] |= sync_word_len;
    cmd[6] |= (addr_comp as u8) & 0x3;
    cmd[7] |= (fsk_pkt_format as u8) & 0x3;
    cmd[8] |= pld_len;
    cmd[9] |= crc as u8;
    cmd[10] |= dc_free as u8;
    cmd
}

/// Configures (G)FSK syncword. Default 0x9723522556536564. For RX only: syncword must be multiple of 8 bits. If not, configure as next multiple of 8 and add filler bits at beginning (e.g., 30 bits -> configure as 32 bits with '01b' or '10b' prefix).
pub fn set_fsk_sync_word_cmd(syncword: u64) -> [u8; 10] {
    let mut cmd = [0u8; 10];
    cmd[0] = 0x02;
    cmd[1] = 0x06;

    cmd[2] |= ((syncword >> 56) & 0xFF) as u8;
    cmd[3] |= ((syncword >> 48) & 0xFF) as u8;
    cmd[4] |= ((syncword >> 40) & 0xFF) as u8;
    cmd[5] |= ((syncword >> 32) & 0xFF) as u8;
    cmd[6] |= ((syncword >> 24) & 0xFF) as u8;
    cmd[7] |= ((syncword >> 16) & 0xFF) as u8;
    cmd[8] |= ((syncword >> 8) & 0xFF) as u8;
    cmd[9] |= (syncword & 0xFF) as u8;
    cmd
}

/// Sets Node and Broadcast addresses for (G)FSK packet filtering when addr_comp enabled (0x01 or 0x02 in SetGfskPacketParams). Reception aborted with adrsErr flag if address comparison fails.
pub fn set_fsk_address_cmd(addr_node: u8, addr_bcast: u8) -> [u8; 4] {
    let mut cmd = [0u8; 4];
    cmd[0] = 0x02;
    cmd[1] = 0x12;

    cmd[2] |= addr_node;
    cmd[3] |= addr_bcast;
    cmd
}

/// Configures CRC polynomial and initial value for flexible CRC configuration. Examples: IBM CRC (InitValue=0xFFFF, Poly=0x8005, CrcType=CRC_2_BYTE), CCITT CRC (InitValue=0x1D0F, Poly=0x1021, CrcType=CRC_2_BYTE_INV)
pub fn set_fsk_crc_params_cmd(init: u32, polynom: u32) -> [u8; 10] {
    let mut cmd = [0u8; 10];
    cmd[0] = 0x02;
    cmd[1] = 0x24;

    cmd[2] |= ((init >> 24) & 0xFF) as u8;
    cmd[3] |= ((init >> 16) & 0xFF) as u8;
    cmd[4] |= ((init >> 8) & 0xFF) as u8;
    cmd[5] |= (init & 0xFF) as u8;
    cmd[6] |= ((polynom >> 24) & 0xFF) as u8;
    cmd[7] |= ((polynom >> 16) & 0xFF) as u8;
    cmd[8] |= ((polynom >> 8) & 0xFF) as u8;
    cmd[9] |= (polynom & 0xFF) as u8;
    cmd
}

/// Sets whitening seed. Polynomial: x^9+x^5+1 (sub-GHz), x^7+x^4+1 (HF). Limits consecutive 1's/0's to 9. Seed must match on all peer devices. Only needed if data has high correlation with long 0/1 strings.
pub fn set_fsk_whit_params_cmd(seed: u16) -> [u8; 4] {
    let mut cmd = [0u8; 4];
    cmd[0] = 0x02;
    cmd[1] = 0x25;

    cmd[2] |= ((seed >> 8) & 0xFF) as u8;
    cmd[3] |= (seed & 0xFF) as u8;
    cmd
}
