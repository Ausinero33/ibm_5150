pub mod hardware;
pub mod util;

use std::time::Duration;

// A
use hardware::display::DisplayAdapter;
pub use hardware::sys::System;
use pixels::Pixels;

pub const DESIRED_FPS: u32 = 50;
pub const TIMESTEP: Duration = Duration::from_nanos(1_000_000_000 / DESIRED_FPS as u64);
const ONE_FRAME: Duration = Duration::from_nanos(1_000_000_000 / 50);

pub struct IbmPc {
    pub sys: System,
    pub pixels: Pixels,

    dt: Duration,

    pub to_print: bool,
    pub msg: String,
}

impl IbmPc {
    pub fn new(pixels: Pixels) -> Self {
        IbmPc {
            sys: System::new(),
            pixels,

            dt: Duration::default(),

            to_print: false,
            msg: String::new(),
        }
    }

    pub fn update(&mut self) {
        self.sys.update();
        // self.dt += TIMESTEP;

        // while self.dt >= ONE_FRAME {
        //     self.dt -= ONE_FRAME;
        //     self.sys.update();
        // }
    }

    pub fn debug(&mut self) {
        self.sys.debug();
    }

    pub fn draw(&mut self) {
        self.sys.bus.mda.create_frame(&self.sys.bus.memory[0xB0000..0xB0FA0], self.pixels.get_frame_mut());
    }
}