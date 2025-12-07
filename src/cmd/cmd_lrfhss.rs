// Lrfhss commands API


/// Coding rate
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum LrfhssCr {
    Cr5p6 = 0,
    Cr2p3 = 1,
    Cr1p2 = 2,
    Cr1p3 = 3,
}

/// Frequency grid selection (25.39kHz or 3.91kHz)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Grid {
    Grid25 = 0,
    Grid4 = 1,
}

/// Intra-packet hopping
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Hopping {
    NoHopping = 0,
    HoppingEnabled = 1,
}

/// Bandwidth occupied by hopping pattern
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum LrfhssBw {
    Bw39p06 = 0,
    Bw85p94 = 1,
    Bw136p72 = 2,
    Bw183p59 = 3,
    Bw335p94 = 4,
    Bw386p72 = 5,
    Bw722p66 = 6,
    Bw773p44 = 7,
    Bw1523p4 = 8,
    Bw1574p2 = 9,
}

#[allow(clippy::too_many_arguments)]
/// Encodes payload and configures internal hopping table. Returns CMD_OK if valid, CMD_PERR if invalid. Does NOT send packet - use SetTx to transmit. Max coded packet 255 bytes. See payload length table for user payload limits by CR and HeaderCount. FCC use case: BW 0x08/0x09, Hopping 0x01, Grid 0x00. If configured, LrFhssHop IRQ asserted at each hop after PA ramp-up.
pub fn lr_fhss_build_frame_cmd(sync_header_cnt: u8, lrfhss_cr: LrfhssCr, mod_type: u8, grid: Grid, hopping: Hopping, lrfhss_bw: LrfhssBw, hop_sequence: u16, device_offset: i8) -> [u8; 11] {
    let mut cmd = [0u8; 11];
    cmd[0] = 0x02;
    cmd[1] = 0x2C;

    cmd[2] |= sync_header_cnt;
    cmd[3] |= lrfhss_cr as u8;
    cmd[4] |= mod_type;
    cmd[5] |= grid as u8;
    cmd[6] |= hopping as u8;
    cmd[7] |= lrfhss_bw as u8;
    cmd[8] |= ((hop_sequence >> 8) & 0xFF) as u8;
    cmd[9] |= (hop_sequence & 0xFF) as u8;
    cmd[10] |= (device_offset) as u8;
    cmd
}

/// Sets LR-FHSS syncword (4 bytes). Default: {0x2C, 0x0F, 0x79, 0x95}
pub fn lr_fhss_set_sync_word_cmd(syncword: u32) -> [u8; 6] {
    let mut cmd = [0u8; 6];
    cmd[0] = 0x02;
    cmd[1] = 0x2D;

    cmd[2] |= ((syncword >> 24) & 0xFF) as u8;
    cmd[3] |= ((syncword >> 16) & 0xFF) as u8;
    cmd[4] |= ((syncword >> 8) & 0xFF) as u8;
    cmd[5] |= (syncword & 0xFF) as u8;
    cmd
}
