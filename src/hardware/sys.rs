// use std::fs::File;
use std::fs::File;
#[cfg(not(debug_assertions))]
use std::time::{Instant, Duration};
// use std::thread::sleep;
// use std::time::{Instant, Duration};

// use crossterm::execute;
// use crossterm::terminal::SetSize;

use super::cpu_8088::cpu_utils;
use super::cpu_8088::CPU;
use super::bus::Bus;
// use crate::util::debug::*;

use std::fs::OpenOptions;

pub struct System {
    pub cpu: CPU,
    pub bus: Bus,

    pub running: bool,

    pub file: File,
    // pub rx: Mutex<Receiver<bool>>,
}

impl System {
    pub fn new() -> Self {
        // execute!(stdout(), SetSize(120, 30)).unwrap();
        let sys = System { 
            cpu: CPU::new(),
            bus: Bus::new(),

            running: false,

            file: OpenOptions::new().create(true).write(true).open("logs/logs.txt").unwrap(),
            // rx: Mutex::new(rx),
        };
      
        // sys.rst();
        sys
    }
}

use crate::{DESIRED_FPS, util::debug_bios::debug_82};

impl System {
    pub fn rst(&mut self) {
        self.cpu = CPU::new();
        self.bus = Bus::new();

        self.bus.write_8(0x40, 0x12, 1);
        self.running = false;
    }

    // Llamar cada frame
    pub fn update(&mut self) {
        let max_cycles = (4_772_726.7 / DESIRED_FPS) as u32;
        let mut cycles_ran = 0;

        while cycles_ran <= max_cycles {
            self.step(&mut cycles_ran);
        }

        // self.file.flush().unwrap();
        // display(self);
    }

    #[inline]
    pub fn step(&mut self, cycles_ran: &mut u32) {
        if self.cpu.ip == 0xE6BD {
            // println!("llego")
            let _a = 0;
        }

        // ACTUALIZAR TIMER
        self.bus.update_timer();
    
        debug_82(&mut self.cpu);
        let (cycles, _ip) = self.cpu.fetch_decode_execute(&mut self.bus);
        // println!("{:04X}", _ip);

        self.cpu.handle_interrupts(&mut self.bus);

        *cycles_ran += cycles;
        // writeln!(&mut self.file, "{:05X} - {}", ((self.cpu.cs as usize) << 4) + _ip as usize, self.cpu.instr.opcode).unwrap();
        //self.file.flush().unwrap();

        if self.cpu.halted { 
            // let _a = 0;
            // self.file.flush().unwrap();
            // println!("{:04X}", self.cpu.ip);
            todo!("Halted") 
        }

        // if (((self.cpu.cs as usize) << 4) + self.cpu.ip as usize) >= 0xF6000 && (((self.cpu.cs as usize) << 4) + self.cpu.ip as usize) < 0xFE000 {
        //     // self.file.flush().unwrap();
        //     println!("Tecnicamente esta booteando?")
        // }
        if cpu_utils::get_address(&mut self.cpu) == 0xF6000 {
            println!("Tecnicamente esta booteando?")
        }
    }

    pub fn load_bios(&mut self) {
        unsafe {
            for (idx, element) in std::fs::read("roms/basic.bin").unwrap().into_iter().enumerate() {
                std::ptr::write(&mut self.bus.memory[0xF6000 + idx], element);
                // println!("{:04X} : {:02X}", 0xF6000 + idx, element);
            }

            for (idx, element) in std::fs::read("roms/BIOS_IBM5150_27OCT82_1501476_U33.BIN").unwrap().into_iter().enumerate() {
                std::ptr::write(&mut self.bus.memory[0xFE000 + idx], element);
            }
            // for (idx, element) in std::fs::read("roms/bios.BIN").unwrap().into_iter().enumerate() {
            //     std::ptr::write(&mut self.bus.memory[0xFE000 + idx], element);
            // }
        }
    }
}
