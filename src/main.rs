use std::time::Duration;

use ibm_5150::*;
use ibm_5150::hardware::display::DisplayAdapter;
use pixels::{Error, SurfaceTexture, Pixels};
use winit::{event_loop::EventLoop, dpi::LogicalSize, window::WindowBuilder};
use winit_input_helper::WinitInputHelper;

const FPS: u32 = 50;
const TIMESTEP: Duration = Duration::from_nanos(1_000_000_000 / FPS as u64);

fn main() -> Result<(), Error> {
    // env_logger::init();
    let event_loop = EventLoop::new();
    
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(720., 350.);
        WindowBuilder::new()
            .with_title("IBM 5150 Emulator")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .with_max_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(720, 350, surface_texture)?
    };
    let mut state = IbmPc::new(pixels);

    event_loop.run(move |event, _, control_flow| {

    });
}