// pub mod hardware;
// mod util;
// use std::{sync::{Arc, mpsc::{Sender, Receiver, self}, RwLock}, thread::JoinHandle};

// use std::time::Duration;

// use eframe::{run_native, NativeOptions, App};
// use crate::hardware::sys::System;
// use util::debug::display;

// struct IbmPc {
//     system: Arc<RwLock<System>>,

//     running: bool,
//     run_handle: Option<JoinHandle<()>>,
//     tx: Sender<bool>,
// }

// impl IbmPc {
//     pub fn new(system: Arc<RwLock<System>>, tx: Sender<bool>) -> Self {
//         IbmPc { 
//             system,

//             running: false,
//             run_handle: None,
//             tx,
//         }
//     }
// }

// impl App for IbmPc {
//     fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
//         egui::CentralPanel::default().show(ctx, |ui| {
//             if let Some(v) = &self.run_handle {
//                 self.running = !v.is_finished();
//             }

//             if ui.button("Step").clicked() {
//                 if !self.running {
//                     self.system.write().unwrap().step();
//                 }
//             }

//             let ip = match self.system.try_read() {
//                 Ok(v) => v.cpu.ip,
//                 Err(_) => 0,
//             };

//             let halted = match self.system.try_read() {
//                 Ok(v) => v.cpu.halted,
//                 Err(_) => false,
//             };

//             ui.label(format!("{}", ip));
//             ui.label(format!("{}", halted));

//             if ui.button("Run").clicked() {
//                 let sys_thread = self.system.clone();

//                 self.tx.send(!self.running).unwrap();
//                 self.running = !self.running;

//                 if self.running {
//                     self.run_handle = Some(std::thread::spawn(move || {
//                         sys_thread.write().unwrap().run();
//                     }));
//                 }
//             }

//             ui.label(format!("{}", self.running));

//             if ui.button("Reset").clicked() {
//                 if !self.running {
//                     self.system.write().unwrap().rst();
//                     self.system.write().unwrap().load_bios();
//                 }
//             }
//         });

//         // display(&self.system.read().unwrap());
//     }
// }

// fn main() {
//     let (tx, rx): (Sender<bool>, Receiver<bool>) = mpsc::channel();

//     let sys = Arc::new(RwLock::new(System::new(rx)));

//     sys.write().unwrap().load_bios();

//     let app = IbmPc::new(sys.clone(), tx);

//     run_native("IBM PC", NativeOptions::default(), Box::new(|_cc| Box::new(app)));
// }
use ibm_5150::*; 

fn _main() -> GameResult {
    let mut app = IbmPc::new();
    let win_mode = WindowMode::default()
                            .dimensions(720., 350.)
                            .resize_on_scale_factor_change(true);

    let cb = ggez::ContextBuilder::new("IBM 5150", "Gonzalo").window_mode(win_mode);
 

    let (ctx, event_loop) = cb.build()?;

    //graphics::set_mode(&mut ctx, win_mode)?;

    app.sys.rst();
    app.sys.load_bios();

    event::run(ctx, event_loop, app);
}

fn main() {
    let mut app = IbmPc::new();

    app.sys.rst();
    app.sys.load_bios();

    loop {
        app.sys.update();
    }
}
