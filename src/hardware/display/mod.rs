pub mod ibm_mda;
pub mod crtc6845;

pub trait DisplayAdapter {
    // fn create_frame(&mut self, ctx: &mut Context, vram: &[u8]) -> ImageGeneric<GlBackendSpec>;
    fn create_frame(&mut self, vram: &[u8], frame: &mut [u32]);
    fn render_font(&mut self, char: Char, width: usize, height: usize);
}

pub struct Char {
    pub index: usize,
    pub background_color: u32,
    pub foreground_color: u32,

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
        //     self.background_color = 0x000000FF;
        //     self.foreground_color = 0x000000FF;
        // } else if matches!(attr, 0x70 | 0x78 | 0xF0 | 0xF8) {
        //     self.background_color = 0xFFFFFFFF;
        //     self.foreground_color = 0x000000FF;
        // } else {
        //     self.background_color = 0x000000FF;
        //     self.foreground_color = 0xFFFFFFFF;
        // }
        self.bright = attr & 0x08 > 0;
        self.underline = attr & 0x07 == 0x01;

        let back = attr >> 4 & 0x07;
        let front = attr & 0x07;

        match (back, front) {
            (0b000, 0b111) => {
                self.foreground_color = 0xFFFFFFFF;
                self.background_color = 0x000000FF;
            },
            (0b111, 0b000) => {
                self.foreground_color = 0x000000FF;
                self.background_color = 0xFFFFFFFF;
            },
            (0b000, 0b000) => {
                self.foreground_color = 0x000000FF;
                self.background_color = 0x000000FF;
            },
            (0b111, 0b111) => {
                self.foreground_color = 0xFFFFFFFF;
                self.background_color = 0xFFFFFFFF;
            },

            _ => {
                self.foreground_color = 0xFFFFFFFF;
                self.background_color = 0x000000FF;
            }
        }

        self
    }
}

impl Default for Char {
    fn default() -> Self {
        Self {
            index: 0x00,
            background_color: 0x000000FF,
            foreground_color: 0xFFFFFFFF,

            bright: false,
            underline: false,
        }
    }
}