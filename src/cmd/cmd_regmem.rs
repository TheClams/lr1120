// Regmem commands API

use crate::status::Status;
/// Writes blocks of 32-bit words in register/memory space starting at a specific address. Address must be 32-bit aligned and data length must be a multiple of 4. Maximum N is 64.
pub fn write_reg_mem32_cmd(addr: u32, data: u32) -> [u8; 9] {
    let mut cmd = [0u8; 9];
    cmd[0] = 0x01;
    cmd[1] = 0x05;

    cmd[2] |= ((addr >> 24) & 0xFF) as u8;
    cmd[3] |= ((addr >> 16) & 0xFF) as u8;
    cmd[4] |= ((addr >> 8) & 0xFF) as u8;
    cmd[5] |= (addr & 0xFF) as u8;
    cmd[5] |= ((data >> 24) & 0xFF) as u8;
    cmd[6] |= ((data >> 16) & 0xFF) as u8;
    cmd[7] |= ((data >> 8) & 0xFF) as u8;
    cmd[8] |= (data & 0xFF) as u8;
    cmd
}

/// Reads blocks of 32-bit words in register/memory space starting at a specific address. Address must be 32-bit aligned. Maximum len is 64 words.
pub fn read_reg_mem32_req(addr: u32, len: u8) -> [u8; 7] {
    let mut cmd = [0u8; 7];
    cmd[0] = 0x01;
    cmd[1] = 0x06;

    cmd[2] |= ((addr >> 24) & 0xFF) as u8;
    cmd[3] |= ((addr >> 16) & 0xFF) as u8;
    cmd[4] |= ((addr >> 8) & 0xFF) as u8;
    cmd[5] |= (addr & 0xFF) as u8;
    cmd[6] |= len;
    cmd
}

/// Reads/modifies/writes the masked bits (Mask bits = 1) of a single 32-bit word in register/memory space at the specified address. Address must be 32-bit aligned.
pub fn write_reg_mem_mask32_cmd(addr: u32, mask: u32, data: u32) -> [u8; 14] {
    let mut cmd = [0u8; 14];
    cmd[0] = 0x01;
    cmd[1] = 0x0C;

    cmd[2] |= ((addr >> 24) & 0xFF) as u8;
    cmd[3] |= ((addr >> 16) & 0xFF) as u8;
    cmd[4] |= ((addr >> 8) & 0xFF) as u8;
    cmd[5] |= (addr & 0xFF) as u8;
    cmd[6] |= ((mask >> 24) & 0xFF) as u8;
    cmd[7] |= ((mask >> 16) & 0xFF) as u8;
    cmd[8] |= ((mask >> 8) & 0xFF) as u8;
    cmd[9] |= (mask & 0xFF) as u8;
    cmd[10] |= ((data >> 24) & 0xFF) as u8;
    cmd[11] |= ((data >> 16) & 0xFF) as u8;
    cmd[12] |= ((data >> 8) & 0xFF) as u8;
    cmd[13] |= (data & 0xFF) as u8;
    cmd
}

// Response structs

/// Response for ReadRegMem32 command
#[derive(Default)]
pub struct ReadRegMem32Rsp([u8; 6]);

impl ReadRegMem32Rsp {
    /// Create a new response buffer
    pub fn new() -> Self {
        Self::default()
    }

    /// Return Status
    pub fn status(&mut self) -> Status {
        self.0[0].into()
    }

    /// Data blocks read from register/memory. Variable length array of 32-bit words.
    pub fn value(&self) -> u32 {
        (self.0[5] as u32) |
        ((self.0[4] as u32) << 8) |
        ((self.0[3] as u32) << 16) |
        ((self.0[2] as u32) << 24)
    }
}

impl AsMut<[u8]> for ReadRegMem32Rsp {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0
    }
}
