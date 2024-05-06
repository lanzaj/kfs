#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Entry {
    pub offset_0: u16,
    pub segment_selector: u16,
    pub reserved: u8,
    pub p_dpl_0_gatetype: u8,
    pub offset_48: u16,
}

impl Entry {
    pub fn new(offset:u32) -> Entry {
        Entry {
            offset_0: (offset & 0xffff) as u16,
            segment_selector: 0x08, //0b0000 0000 0000 1000
            reserved: 0,
            p_dpl_0_gatetype: 0x8e, //10001110
            offset_48: ((offset >> 16) &0xffff) as u16,
        }
    }
}

