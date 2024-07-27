const DATA_TYPE_FULL: u8 = 0x01;
const DATA_TYPE_HALF: u8 = 0x02;
const DATA_TYPE_WRITE_FLASH: u8 = 0x03;
const DATA_TYPE_PLAY_FLASH: u8 = 0x04;

#[repr(C)]
pub struct Full {
    data_type: u8,
    data: [u8; 16*16*3],
}

#[repr(C)]
pub struct Half {
    data_type: u8,
    data: [u8; 16*16*3/2],
}

#[repr(C)]
pub struct WriteFlash {
    data_type: u8,
    frame_idx: u16,
    data: [u8; 16*16*3],
}

#[repr(C)]
pub struct PlayFlash {
    data_type: u8,
    begin_frame_idx_inclusive: u16,
    end_frame_idx_inclusive: u16,
    time_interval_ms: u16,
}

impl Full {
    pub fn new(canvas: &[u8; 16*16*3]) -> Full {
        Full {
            data_type: DATA_TYPE_FULL,
            data: canvas.clone()
        }
    }
}

impl Half {
    // No visible difference for bright colors (top 4 bits), (bottom 4 bits are lost)
    pub fn new(canvas: &[u8; 16*16*3]) -> Half {
        let mut packet = Half {
            data_type: DATA_TYPE_HALF,
            data: [0u8; 16*16*3/2]
        };

        for y in 0..16 {
            for x in 0..16 {
                let idx = y as usize * 16 + x as usize;
            
                // Trim to 4 bits
                let r = canvas[idx * 3 + 0] >> 4; 
                let g = canvas[idx * 3 + 1] >> 4; 
                let b = canvas[idx * 3 + 2] >> 4;
            
                // Pack full canvas to the half_canvas, offset between individual pixels is 3/2 bytes (12 bits)
                if idx % 2 == 0 {
                    packet.data[idx * 3 / 2 + 0] = (packet.data[idx * 3 / 2 + 0] & 0x0f) | (r << 4);
                    packet.data[idx * 3 / 2 + 0] = (packet.data[idx * 3 / 2 + 0] & 0xf0) | (g);
                    packet.data[idx * 3 / 2 + 1] = (packet.data[idx * 3 / 2 + 1] & 0x0f) | (b << 4);
                } else {
                    packet.data[idx * 3 / 2 + 0] = (packet.data[idx * 3 / 2 + 0] & 0xf0) | (r);
                    packet.data[idx * 3 / 2 + 1] = (packet.data[idx * 3 / 2 + 1] & 0x0f) | (g << 4);
                    packet.data[idx * 3 / 2 + 1] = (packet.data[idx * 3 / 2 + 1] & 0xf0) | (b);
                } 
            }
        }

        packet
    }
}

impl WriteFlash {
    pub fn new(frame_idx: u16, canvas: &[u8; 16*16*3]) -> WriteFlash {
        WriteFlash {
            data_type: DATA_TYPE_WRITE_FLASH,
            frame_idx,
            data: canvas.clone()
        }
    }
}

impl PlayFlash {
    pub fn new(begin_frame_idx_inclusive: u16, end_frame_idx_inclusive: u16, time_interval_ms: u16) -> PlayFlash {
        PlayFlash {
            data_type: DATA_TYPE_PLAY_FLASH,
            begin_frame_idx_inclusive,
            end_frame_idx_inclusive,
            time_interval_ms,
        }
    }
}
