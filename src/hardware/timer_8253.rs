use std::fmt::{Debug, Display};

use super::{peripheral::Peripheral, pic_8259::{PIC8259, IRQs}};

#[derive(Clone, Copy)]
enum Mode {
    Mode0,
    Mode1,
    Mode2,
    Mode3,
    Mode4,
    Mode5
}

#[derive(Clone, Copy)]
pub struct Channel {
    pub current_count: DecimalFixed,
    reload_value: u16,          // TODO
    latch_val: u16,
    rl_mode: u8,
    mode: Mode,

    toggle: bool,

    active: bool,               // TODO
}

impl Channel {
    fn new() -> Self {
        Self {
            current_count: DecimalFixed(0),
            reload_value: 0,
            latch_val: 0,
            rl_mode: 0,
            mode: Mode::Mode0,

            toggle: true,

            active: true,
        }
    }
}

#[derive(Clone, Copy)]
pub struct TIM8253 {
    pub channels: [Channel; 3],
    mode_reg: u8,
}

impl TIM8253 {
    pub fn new() -> Self {
        Self {
            channels: [Channel::new(); 3],
            mode_reg: 0,
        }
    }

    // TODO TERMINAR FUNCIONAMIENTO
    pub fn update(&mut self, cycles: u32, pic: &mut PIC8259) {
        for channel in 0..3 {
            match self.channels[channel].mode {
                Mode::Mode0 => {
                    let before = self.get_current_count(channel);
                    self.channels[channel].current_count.dec(cycles);
                    let after = self.get_current_count(channel);

                    let dif = before.wrapping_sub(1).overflowing_sub(after.wrapping_sub(1));

                    let _a = self.get_current_count(1);
                    let _b = 0;

                    if dif.1 {
                        self.channels[channel].current_count = DecimalFixed(0);
                        if channel == 0 {
                            pic.irq(IRQs::Irq0);
                        }
                    }
                },
                Mode::Mode1 => {},
                Mode::Mode2 => {},
                Mode::Mode3 => {},
                Mode::Mode4 => {},
                Mode::Mode5 => {},
            }
        }
    }
}

impl Peripheral for TIM8253 {
    fn port_in(&mut self, port: u16) -> u16 {
        match port {
            0x40..=0x42 => {
                let channel = (port & 0b11) as usize;

                match self.channels[channel].rl_mode {
                    0b00 => self.channels[channel].latch_val,
                    0b01 => self.get_current_count(channel) as u8 as u16,
                    0b10 => self.get_current_count(channel) >> 8,
                    0b11 => {
                        if self.channels[channel].toggle {
                            self.channels[channel].toggle = false;
                            self.get_current_count(channel) as u8 as u16
                        } else {
                            self.channels[channel].toggle = true;
                            self.get_current_count(channel) >> 8
                        }
                    }
                    _ => unreachable!()
                }
            },
            _ => 0
        }
    }

    fn port_out(&mut self, val: u16, port: u16) {
        match port {
            0x40..=0x42 => {
                let channel = (port & 0b11) as usize;
                self.set_current_count(channel, val)
            },
            0x43 => {
                self.mode_reg = val as u8;
                let channel = ((self.mode_reg & 0b11000000) >> 6) as usize;
                let access_mode = (self.mode_reg & 0b00110000) >> 4;
                match access_mode {
                    0b00 => self.channels[channel].latch_val = self.get_current_count(channel),
                    _ => {self.channels[channel].rl_mode = access_mode},
                }

                let mode = (self.mode_reg & 0b00001110) >> 1;

                match mode {
                    0b000 => self.channels[channel].mode = Mode::Mode0,
                    0b001 => self.channels[channel].mode = Mode::Mode1,
                    0b010 | 0b110 => self.channels[channel].mode = Mode::Mode2,
                    0b011 | 0b111 => self.channels[channel].mode = Mode::Mode3,
                    0b100 => self.channels[channel].mode = Mode::Mode4,
                    0b101 => self.channels[channel].mode = Mode::Mode5,
                    _ => unreachable!(),
                }
            },
            _ => unreachable!(),
        }
    }
}

impl TIM8253 {
    // fn set_current_count(&mut self, channel: usize, val: u16) {
    //     let val = match self.channels[channel].rl_mode {
    //         0b01 => (((self.get_current_count(channel) & 0xFF00) | (val & 0x00FF)) as u32) << 2,
    //         0b10 => (((self.get_current_count(channel) & 0x00FF) | ((val & 0x00FF) << 8)) as u32) << 2,
    //         0b11 => if self.channels[channel].toggle {
    //             self.channels[channel].toggle = false;
    //             (((self.get_current_count(channel) & 0xFF00) | (val & 0x00FF)) as u32) << 2
    //         } else {
    //             self.channels[channel].toggle = true;
    //             (((self.get_current_count(channel) & 0x00FF) | ((val & 0x00FF) << 8)) as u32) << 2
    //         },
    //         _ => unreachable!()
    //     };

    //     self.channels[channel].current_count = val;
    // }

    fn set_current_count(&mut self, channel: usize, val: u16) {
        match self.channels[channel].rl_mode {
            0b01 => self.channels[channel].current_count.set_low(val as u8), 
            0b10 => self.channels[channel].current_count.set_high(val as u8),
            0b11 => if self.channels[channel].toggle {
                self.channels[channel].toggle = false;
                self.channels[channel].current_count.set_low(val as u8);
            } else {
                self.channels[channel].toggle = true;
                self.channels[channel].current_count.set_high(val as u8);
            },
            _ => unreachable!(),
        }
    }

    fn get_current_count(&self, channel: usize) -> u16 {
        self.channels[channel].current_count.get()
    }
}

#[derive(Clone, Copy)]
// 00000000_00000000.00 -> CLK / 4
pub struct DecimalFixed(u32);

impl DecimalFixed {
    fn set(&mut self, new_val: u16) {
        self.0 = (new_val as u32) << 2;
    }

    fn set_low(&mut self, val: u8) {
        let prev = (self.0 >> 2) as u16;
        let val = val as u16;
        let prev_high = prev & 0xFF00;
        let new_val = prev_high | val;
        self.set(new_val);
    }

    fn set_high(&mut self, val: u8) {
        let prev = (self.0 >> 2) as u16;
        let val = (val as u16) << 8;
        let prev_low = prev & 0x00FF;
        let new_val = val | prev_low;
        self.set(new_val);
    }

    fn get(&self) -> u16 {
        (self.0 >> 2) as u16
    }

    fn dec(&mut self, cycles: u32) {
        // (self.channels[channel].current_count & 0x3FFFF).wrapping_sub(cycles);self
        self.0 = self.0.wrapping_sub(cycles);
        self.0 &= 0x3FFFF;
    }
}

impl Debug for DecimalFixed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let val = match self.0 & 3 {
            0b00 => 0.0,
            0b01 => 0.25,
            0b10 => 0.5,
            0b11 => 0.75,
            _ => unreachable!(),
        } + ((self.0 << 2) as u16) as f64;

        write!(f, "{}", val)
    }
}