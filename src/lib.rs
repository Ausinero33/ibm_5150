pub mod hardware;
pub mod util;

// A
use hardware::display::DisplayAdapter;
pub use hardware::sys::System;
use pixels::Pixels;

pub const DESIRED_FPS: f32 = 50.;

pub struct IbmPc {
    pub sys: System,
    pub pixels: Pixels,

    pub to_print: bool,
    pub msg: String,
}

impl IbmPc {
    pub fn new(pixels: Pixels) -> Self {
        IbmPc {
            sys: System::new(),
            pixels,

            to_print: false,
            msg: String::new(),
        }
    }

    pub fn update(&mut self) {
        self.sys.update();
    }

    pub fn debug(&mut self) {
        self.sys.debug();
    }

    pub fn draw(&mut self) {
        self.sys.bus.mda.create_frame(&self.sys.bus.memory[0xB0000..0xB0FA0], self.pixels.get_frame_mut());
    }
}