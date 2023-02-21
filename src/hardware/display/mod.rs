pub mod ibm_mda;
pub mod crtc6845;

pub trait DisplayAdapter {
    // fn create_frame(&mut self, ctx: &mut Context, vram: &[u8]) -> ImageGeneric<GlBackendSpec>;
    fn create_frame(&mut self, vram: &[u8], frame: &mut [u8]);
    fn render_font(&mut self, char: Char, width: usize, height: usize);
}

pub struct Char {
    pub index: usize,
    pub background_color: [u8; 4],
    pub foreground_color: [u8; 4],

    pub bright: bool,
    pub underline: bool,
}

impl Char {
    fn new(index: usize) -> Self {
        Char { 
            index,
            ..Default::default()
        }
    }

    fn decode_colors(mut self, attr: u8) -> Self {
        // self.bright = attr & 0x0F > 0x08;
        // self.underline = attr & 0x07 == 0x01;

        // if matches!(attr, 0x00 | 0x08 | 0x80 | 0x88) {
        //     self.background_color = [0x00, 0x00, 0x00, 0xFF];
        //     self.foreground_color = [0x00, 0x00, 0x00, 0xFF];
        // } else if matches!(attr, 0x70 | 0x78 | 0xF0 | 0xF8) {
        //     self.background_color = [0xFF; 4];
        //     self.foreground_color = [0x00, 0x00, 0x00, 0xFF];
        // } else {
        //     self.background_color = [0x00, 0x00, 0x00, 0xFF];
        //     self.foreground_color = [0xFF; 4];
        // }
        self.bright = attr & 0x08 > 0;
        self.underline = attr & 0x07 == 0x01;

        let back = attr >> 4 & 0x07;
        let front = attr & 0x07;

        match (back, front) {
            (0b000, 0b111) => {
                self.foreground_color = [0xFF; 4];
                self.background_color = [0x00, 0x00, 0x00, 0xFF];
            },
            (0b111, 0b000) => {
                self.foreground_color = [0x00, 0x00, 0x00, 0xFF];
                self.background_color = [0xFF; 4];
            },
            (0b000, 0b000) => {
                self.foreground_color = [0x00, 0x00, 0x00, 0xFF];
                self.background_color = [0x00, 0x00, 0x00, 0xFF];
            },
            (0b111, 0b111) => {
                self.foreground_color = [0xFF; 4];
                self.background_color = [0xFF; 4];
            },

            _ => {
                self.foreground_color = [0xFF; 4];
                self.background_color = [0x00, 0x00, 0x00, 0xFF];
            }
        }

        self
    }
}

impl Default for Char {
    fn default() -> Self {
        Self {
            index: 0x00,
            background_color: [0x00, 0x00, 0x00, 0xFF],
            foreground_color: [0xFF; 4],

            bright: false,
            underline: false,
        }
    }
}