pub struct Timer {
    divider: u8, // DIV
    counter: u8, // TIMA
    modulo: u8,  // TMA
    start: bool,
    step: u32,
    cnt: u32,
    div: u32,
    pub interrupt: bool,
}

impl Timer {
    pub fn new() -> Timer {
        Timer {
            divider: 0,
            counter: 0,
            modulo: 0,
            start: false,
            step: 0,
            cnt: 0,
            div: 0,
            interrupt: false,
        }
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0xff04 => self.divider,
            0xff05 => self.counter,
            0xff06 => self.modulo,
            0xff07 => {
                (if self.start { 0x04 } else { 0 })
                    | (match self.step {
                        16 => 1,
                        64 => 2,
                        256 => 3,
                        _ => 0,
                    })
            }
            _ => unimplemented!("read: Timer I/O {:04x}", addr),
        }
    }

    pub fn write_byte(&mut self, addr: u16, v: u8) {
        match addr {
            0xff04 => self.divider = 0,
            0xff05 => self.counter = v,
            0xff06 => self.modulo = v,
            0xff07 => {
                self.start = v & 0x04 != 0;
                self.step = match v & 0x03 {
                    1 => 16,
                    2 => 64,
                    3 => 256,
                    _ => 1024,
                };
            }
            _ => unimplemented!("write: Timer I/O {:04x} {:02x}", addr, v),
        };
    }

    pub fn run(&mut self, ticks: u32) {
        self.div += ticks;
        while self.div >= 256 {
            self.divider = self.divider.wrapping_add(1);
            self.div -= 256;
        }

        if self.start {
            self.cnt += ticks;
            while self.cnt >= self.step {
                self.counter = self.counter.wrapping_add(1);
                if self.counter == 0 {
                    self.counter = self.modulo;
                    self.interrupt = true;
                }
                self.cnt -= self.step;
            }
        }
    }
}
