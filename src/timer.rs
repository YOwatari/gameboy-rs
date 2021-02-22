pub struct Timer {
    div: u8,  // divider
    tima: u8, // counter
    tma: u8,  // modulo
    tac: u8,  // control

    idiv: u32,
    ticks: u32,
    pub interrupt: bool,
}

impl Timer {
    pub fn new() -> Timer {
        Timer {
            div: 0,
            tima: 0,
            tma: 0,
            tac: 0,
            idiv: 0,
            ticks: 0,
            interrupt: false,
        }
    }

    pub fn read_byte(&self, addr: u16) -> u8 {
        match addr {
            0xff04 => self.div,
            0xff05 => self.tima,
            0xff06 => self.tma,
            0xff07 => self.tac,
            _ => unimplemented!("read: Timer I/O {:04x}", addr),
        }
    }

    pub fn write_byte(&mut self, addr: u16, v: u8) {
        match addr {
            0xff04 => self.div = 0,
            0xff05 => self.tima = v,
            0xff06 => self.tma = v,
            0xff07 => self.tac = v,
            _ => unimplemented!("write: Timer I/O {:04x} {:02x}", addr, v),
        };
    }

    pub fn run(&mut self, ticks: u32) {
        self.idiv += ticks;
        while self.idiv >= 256 {
            self.div = self.div.wrapping_add(1);
            self.idiv -= 256;
        }

        if self.tac & 0x04 != 0 {
            let steps: u32 = match self.tac & 0x03 {
                0 => 1024, //   4096 Hz
                1 => 16,   // 262144 Hz
                2 => 64,   //  65536 Hz
                3 => 128,  //  16384 Hz
                _ => 1024,
            };

            self.ticks += ticks;
            while self.ticks >= steps {
                self.tima = self.tima.wrapping_add(1);
                if self.tima == 0 {
                    self.tima = self.tma;
                    self.interrupt = true;
                }
                self.ticks -= steps;
            }
        }
    }
}
