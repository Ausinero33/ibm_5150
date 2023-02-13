use super::{Peripheral, pic_8259::{PIC8259, IRQs}};

#[derive(Clone, Copy, PartialEq)]
enum Mode {
    Mode0,
    Mode1,
    Mode2,
    Mode3,
    Mode4,
    Mode5
}

impl Default for Mode {
    fn default() -> Self {
        Mode::Mode0
    }
}

#[derive(Default, Clone)]
pub struct TIM8253 {
    pub cycles: u32,

    count: [u16; 3],
    reload: [u16; 3],
    latched: [bool; 3],
    latch_val: [u16; 3],
    rl_mode: [u8; 3],
    mode: [Mode; 3],
    out: [bool; 3],
    active: [bool; 3],

    toggle: [bool; 3],

    mode_reg: u8,
}

impl TIM8253 {
    pub fn new() -> Self {
        Self {
            active: [false; 3],
            ..Default::default()
        }
    }

    fn output(&mut self, channel: usize, state: bool, pic: &mut PIC8259) {
        if !self.out[channel] && state && channel == 0 {
            pic.irq(IRQs::Irq0);
        }
        self.out[channel] = state;
    }

    pub fn update(&mut self, pic: &mut PIC8259) {
        while self.cycles > 3 {
            for i in 0..3 {
                if self.active[i] {

                    match self.mode[i] {
                        Mode::Mode0 => {
                            self.count[i] = self.count[i].wrapping_sub(1);
                            if self.count[i] == 0 {
                                self.output(i, true, pic)
                            }
                        },

                        Mode::Mode2 => {
                            self.count[i] = self.count[i].wrapping_sub(1);
                            if self.count[i] == 1 {
                                self.output(i, false, pic);
                            } else {
                                self.output(i, true, pic);
                                if self.count[i] == 0 {
                                    self.count[i] = self.reload[i];
                                }
                            }
                        }

                        _ => {}, // TODO
                    }
                }
            }

            self.cycles -= 4;
        }
    }
}

impl Peripheral for TIM8253 {
    fn port_in(&mut self, port: u16) -> u16 {
        match port {
            0x40..=0x42 => {
                let channel = (port & 0b11) as usize;

                let val = if self.latched[channel] {
                    self.latch_val[channel]
                } else {
                    self.count[channel]
                };

                match self.rl_mode[channel] {
                    0b01 => {
                        // Invertir si latched esta activo
                        self.latched[channel] ^= self.latched[channel];
                        val as u8 as u16
                    },
                    0b10 => {
                        self.latched[channel] ^= self.latched[channel];
                        val >> 8
                    },
                    0b11 => {
                        if self.toggle[channel] {
                            self.toggle[channel] = false;
                            val as u8 as u16
                        } else {
                            self.toggle[channel] = true;
                            self.latched[channel] ^= self.latched[channel];
                            val >> 8
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
                
                match self.rl_mode[channel] {
                    0b01 => {
                        self.reload[channel] = self.reload[channel] & 0xFF00 | val & 0x00FF;
                    },
                    0b10 => {
                        self.reload[channel] = self.reload[channel] & 0x00FF | val & 0xFF00;
                    },
                    0b11 => {
                        if self.toggle[channel] {
                            self.toggle[channel] = false;
                            self.reload[channel] = self.reload[channel] & 0xFF00 | val & 0x00FF;
                        } else {
                            self.toggle[channel] = true;
                            self.reload[channel] = self.reload[channel] & 0x00FF | val & 0xFF00;
                        }
                    },
                    _ => unreachable!()
                }

                if self.rl_mode[channel] < 0b11 || self.toggle[channel] {
                    self.count[channel] = self.reload[channel];
                    self.active[channel] = true;
                    self.out[channel] = self.mode[channel] == Mode::Mode2 || self.mode[channel] == Mode::Mode3;
                }
            },
            0x43 => {
                self.mode_reg = val as u8;
                let channel = ((self.mode_reg & 0b11000000) >> 6) as usize;
                let access_mode = (self.mode_reg & 0b00110000) >> 4;
                match access_mode {
                    0b00 => self.latch_val[channel] = self.count[channel],
                    _ => {
                        self.rl_mode[channel] = access_mode;
                        let mode = (self.mode_reg & 0b00001110) >> 1;

                        match mode {
                            0b000 => self.mode[channel] = Mode::Mode0,
                            0b001 => self.mode[channel] = Mode::Mode1,
                            0b010 | 0b110 => self.mode[channel] = Mode::Mode2,
                            0b011 | 0b111 => self.mode[channel] = Mode::Mode3,
                            0b100 => self.mode[channel] = Mode::Mode4,
                            0b101 => self.mode[channel] = Mode::Mode5,
                            _ => unreachable!(),
                        }
                    },
                }

            },
            _ => unreachable!(),
        }
    }
}