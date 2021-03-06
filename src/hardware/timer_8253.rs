use super::peripheral::Peripheral;

#[derive(Clone, Copy)]
struct Channel {
    current_count: u32,
    reload_value: u32,
    latch_val: u32,
    mode: u8,

    toggle: bool,
}

impl Channel {
    fn new() -> Self {
        Self {
            current_count: 0,
            reload_value: 0,
            latch_val: 0,
            mode: 0,

            toggle: true,
        }
    }
}

#[derive(Clone, Copy)]
pub struct TIM8253 {
    channels: [Channel; 3],
    mode_reg: u8,
}

impl TIM8253 {
    pub fn new() -> Self {
        Self {
            channels: [Channel::new(); 3],
            mode_reg: 0,
        }
    }

    // TODO FUNCIONAMIENTO
    pub fn tick(&mut self) {
        // TODO Temporal (Esto hay que pensarlo mejor)
        self.channels[0].current_count = self.channels[0].current_count.wrapping_sub(1);
        if self.channels[0].current_count == 0 {
            // TODO INT
        }

        self.channels[1].current_count = self.channels[1].current_count.wrapping_sub(1);
    }
}

impl Peripheral for TIM8253 {
    fn port_in(&mut self, port: u16) -> u16 {
        match port {
            0x40..=0x42 => {
                let channel = (port & 0b11) as usize;
                let access_mode = (self.channels[channel].mode & 0b00110000) >> 4;

                match access_mode {
                    0b00 => (self.channels[channel].latch_val >> 2) as u16,
                    0b01 => (self.channels[channel].current_count >> 2) as u8 as u16,
                    0b10 => ((self.channels[channel].current_count >> 2) >> 8) as u16,
                    0b11 => {
                        if self.channels[channel].toggle {
                            self.channels[channel].toggle = false;
                            (self.channels[channel].current_count >> 2) as u8 as u16
                        } else {
                            self.channels[channel].toggle = true;
                            ((self.channels[channel].current_count >> 2) >> 8) as u16
                        }
                    }
                    _ => unreachable!()
                }
            }, // TODO
            _ => 0
        }
    }

    fn port_out(&mut self, val: u16, port: u16) {
        match port {
            0x40..=0x42 => {
                let channel = (port & 0b11) as usize;
                let access_mode = (self.channels[channel].mode & 0b00110000) >> 4;
                match access_mode {
                    0b01 => self.channels[channel].current_count = (((self.channels[channel].current_count as u16 & 0xFF00) | (val & 0x00FF)) as u32) << 2,
                    0b10 => self.channels[channel].current_count = (((self.channels[channel].current_count as u16 & 0x00FF) | ((val & 0x00FF) << 8)) as u32) << 2,
                    0b11 => self.channels[channel].current_count = if self.channels[channel].toggle {
                        self.channels[channel].toggle = false;
                        (((self.channels[channel].current_count as u16 & 0xFF00) | (val & 0x00FF)) as u32) << 2
                    } else {
                        self.channels[channel].toggle = true;
                        (((self.channels[channel].current_count as u16 & 0x00FF) | ((val & 0x00FF) << 8)) as u32) << 2
                    },
                    _ => unreachable!()
                }
            },
            0x43 => {
                self.mode_reg = val as u8;
                let channel = ((self.mode_reg & 0b11000000) >> 6) as usize;
                let access_mode = (self.mode_reg & 0b00110000) >> 4;
                self.channels[channel].mode = self.mode_reg;
                match access_mode {
                    0b00 => self.channels[channel].latch_val = self.channels[channel].current_count,
                    _ => {},
                }
            },
            _ => unreachable!(),
        }
    }
}