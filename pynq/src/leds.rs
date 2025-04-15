use crate::mmio::Mmio;

#[derive(Copy, Clone, Debug)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Yellow = 6,
    White = 7,
}

pub struct RgbLeds {
    mem: Mmio,
}

impl RgbLeds {
    pub fn get() -> Self {
        let mut mem = Mmio::map(0x41210000, 8);
        // configure lowest 6 gpios as output
        mem[1] = !((7 << 3) | 7);
        RgbLeds { mem }
    }
    pub fn set(&mut self, ld4_color: Color, ld5_color: Color) {
        self.mem[0] = (ld4_color as u32 & 7) | ((ld5_color as u32 & 7) << 3);
    }
    pub fn set_ld4(&mut self, color: Color) {
        let old = self.mem[0];
        self.mem[0] = (old & !7) | ((color as u32) & 7);
    }
    pub fn set_ld5(&mut self, color: Color) {
        let old = self.mem[0];
        self.mem[0] = (old & !(7 << 3)) | (((color as u32) & 7) << 3);
    }
}
impl Drop for RgbLeds {
    fn drop(&mut self) {
        // reset to all inputs
        self.mem[1] = !0u32;
    }
}
