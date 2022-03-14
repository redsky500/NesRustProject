pub struct CPU {
    pub register_a: u8,
    pub register_x: u8,
    pub register_y: u8,
    pub processor_status: u8,
    pub program_counter: u16,
    memory: [u8; 0xFFFF]
}

trait Memory {
    fn memory_read(&self, addr: u16) -> u8; 

    fn memory_write(&mut self, addr: u16, data: u8);
    
    fn memory_read_u16(&self, pos: u16) -> u16 {
        let lo = self.memory_read(pos) as u16;
        let hi = self.memory_read(pos + 1) as u16;
        (hi << 8) | (lo as u16)
    }

    fn memory_write_u16(&mut self, pos: u16, data: u16) {
        let hi = (data >> 8) as u8;
        let lo = (data & 0xff) as u8;
        self.memory_write(pos, lo);
        self.memory_write(pos + 1, hi);
    }
}


impl Memory for CPU {
    
    fn memory_read(&self, addr: u16) -> u8 { 
        self.memory[addr as usize]
    }

    fn memory_write(&mut self, addr: u16, data: u8) { 
        self.memory[addr as usize] = data;
    }
}

impl CPU {
    pub fn new() -> Self {
        CPU {
            register_a: 0,
            register_x: 0,
            register_y: 0,
            processor_status: 0,
            program_counter: 0,
            memory: [0; 0xFFFF]
        }
    }

    pub fn reset(&mut self) {
        self.register_a = 0;
        self.register_x = 0;
        self.processor_status = 0;
 
        self.program_counter = self.memory_read_u16(0xFFFC);
    }
 
    pub fn load(&mut self, program: Vec<u8>) {
        self.memory[0x8000 .. (0x8000 + program.len())].copy_from_slice(&program[..]);
        self.memory_write_u16(0xFFFC, 0x8000);
    }
 
    pub fn load_and_run(&mut self, program: Vec<u8>) {
        self.load(program);
        self.reset();
        self.execute()
    }

    fn lda(&mut self, value: u8) {
        self.register_a = value;
        self.update_zero_and_negative_flags(self.register_a);
    }

    fn tax(&mut self) {
        self.register_x = self.register_a;
        self.update_zero_and_negative_flags(self.register_x);
    }

    fn inx(&mut self) {
        self.register_x = self.register_x.wrapping_add(1);
        self.update_zero_and_negative_flags(self.register_x);
    }


    fn update_zero_and_negative_flags(&mut self, result: u8) {
        if result == 0 {
            self.processor_status = self.processor_status | 0b0000_0010;
        } else {
            self.processor_status = self.processor_status & 0b1111_1101;
        }

        if result & 0b1000_0000 != 0 {
            self.processor_status = self.processor_status | 0b1000_0000;
        } else {
            self.processor_status = self.processor_status & 0b0111_1111;
        }
    }

    pub fn execute(&mut self) {
    
        loop {
            let opscode = self.memory_read(self.program_counter);
            self.program_counter += 1;
    
            match opscode {
                0xA9 => {
                    let param = self.memory[self.program_counter as usize];
                    self.program_counter += 1;

                    self.lda(param);
                }

                0xAA => self.tax(),

                0xE8 => self.inx(),

                0x00 => return,

                _ => todo!(),
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_0xa9_lda_is_loading_accumulator() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x05, 0x00]);
        assert_eq!(cpu.register_a, 5);
        assert!(cpu.processor_status & 0b0000_0010 == 0);
        assert!(cpu.processor_status & 0b1000_0000 == 0);
    }

    #[test]
    fn test_0xa9_lda_zero_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x00, 0x00]);
        assert_eq!(cpu.register_a, 0);
        assert!(cpu.processor_status & 0b0000_0010 == 0b10);
    }

    #[test]
    fn test_0xa9_lda_negative_flag() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0xff, 0x00]);
        assert!(cpu.processor_status & 0b1000_0000 == 0b1000_0000);
    }

    #[test]
    fn test_0xaa_tax_is_moving_from_a_to_x() {
        let mut cpu = CPU::new();
        cpu.load_and_run(vec![0xa9, 0x05, 0xaa, 0x00]);
        assert_eq!(cpu.register_x, 5);
    }

    #[test]
    fn test_inx_overflow() {
        let mut cpu = CPU::new();
        cpu.register_x = 0xff;
        cpu.load_and_run(vec![0xa9, 0xff, 0xaa, 0xe8, 0x00]);

        assert_eq!(cpu.register_x, 0)
    }
}
