#[allow(deprecated)]
use std::io::{self, Write};

pub struct RVM {
    memory: [u8; 65536],   // 64 Kb
    ip: usize,
    running: bool,
    sp: usize,
    registers: [u64; 16],  // 64-bit registres
    zero_flag: bool,
}

#[allow(dead_code)]
impl RVM {
    pub fn new(regs: [u64; 16]) -> Self {
        RVM {
            memory: [0; 65536],
            ip: 0x200, // 512
            sp: 65536,
            running: true,
            registers: regs,
            zero_flag: false,
        }
    }

    pub fn load_program(&mut self, program_bytes: &[u8], offset: usize) {
        for (i, &byte) in program_bytes.iter().enumerate() {
            self.memory[offset + i] = byte;
        }
    }

    // Print a single 16-byte row in hex + ASCII format
    fn print_hex_row(addr: usize, slice: &[u8]) {
        print!("{:04X}: ", addr);
        for i in 0..16 {
            if i < slice.len() {
                print!("{:02X} ", slice[i]);
            } else {
                print!("   ");
            }
            if i == 7 {
                print!(" ");
            }
        }
        print!(" |");
        for &b in slice {
            let ch = if b.is_ascii_graphic() || b == b' ' {
                b as char
            } else {
                '.'
            };
            print!("{}", ch);
        }
        println!("|");
    }

    // Print hex dump of a specific memory range
    pub fn dump_memory(&self, start: usize, len: usize) {
        let end = start.saturating_add(len).min(self.memory.len());
        println!("\n--- Memory Dump ({:#06X}..{:#06X}) ---", start, end);
        for addr in (start..end).step_by(16) {
            let line_end = (addr + 16).min(end);
            Self::print_hex_row(addr, &self.memory[addr..line_end]);
        }
    }

    // Print hex dump of all non-zero 16-byte aligned memory rows
    pub fn dump_memory_nonzero(&self) {
        println!("\n--- Memory Dump (Hex) ---");
        let mut any = false;
        for addr in (0..self.memory.len()).step_by(16) {
            let slice = &self.memory[addr..addr + 16];
            if slice.iter().any(|&b| b != 0) {
                any = true;
                Self::print_hex_row(addr, slice);
            }
        }
        if !any {
            println!("(memory is empty)");
        }
    }

    pub fn run(&mut self) {
        println!("--- Start ---");

        while self.running {
            if self.ip + 8 > self.memory.len() {
                println!("\n[Err] IP out of bounds: {}", self.ip);
                break;
            }

            // Fetch (32 bit)
            // Fetch 64-bit instruction
            let opcode = u64::from_be_bytes(
                self.memory[self.ip..self.ip + 8].try_into().unwrap(),
            );
            self.ip += 8;

            let group = ((opcode >> 60) & 0x0F) as u8;
            let subop = ((opcode >> 56) & 0x0F) as u8;
            let r1    = ((opcode >> 52) & 0x0F) as usize;
            let r2    = ((opcode >> 48) & 0x0F) as usize;
            let imm   = opcode & 0xFFFF_FFFF_FFFF;
            let addr  = (opcode & 0xFFFF_FFFF_FFFF) as usize;

            match group {
                // 0x0: system и I/O
                0x0 => match subop {
                    0x0 => self.running = false, // halt
                    0x1 => println!(),           // nl
                    0x2 => {                     // out char
                        print!("{}", (imm as u8) as char);
                        io::stdout().flush().unwrap();
                    }
                    0x3 => {                     // out reg (char)
                        print!("{}", (self.registers[r1] as u8) as char);
                        io::stdout().flush().unwrap();
                    }
                    0x4 => {                     // outn reg (u64)
                        print!("{}", self.registers[r1]);
                        io::stdout().flush().unwrap();
                    }
                    0x5 => {                     // out imm
                        print!("{}", imm);
                        io::stdout().flush().unwrap();
                    }
                    0x6 => {                     // in reg
                        let mut input = String::new();
                        io::stdout().flush().unwrap();
                        if io::stdin().read_line(&mut input).is_ok() {
                            match input.trim().parse::<u64>() {
                                Ok(num) => self.registers[r1] = num,
                                Err(_) => {
                                    println!("\n[Err] Expected 64-bit number.");
                                    self.running = false;
                                }
                            }
                        } else {
                            self.running = false;
                        }
                    }
                    _ => self.running = false,
                },

                // 0x1: ALU Register-Register
                0x1 => match subop {
                    0x0 => self.registers[r1] = self.registers[r2],
                    0x1 => {
                        self.registers[r1] = self.registers[r1].wrapping_add(self.registers[r2]);
                        self.zero_flag = self.registers[r1] == 0;
                    }
                    0x2 => {
                        self.registers[r1] = self.registers[r1].wrapping_sub(self.registers[r2]);
                        self.zero_flag = self.registers[r1] == 0;
                    }
                    0x3 => {
                        self.registers[r1] = self.registers[r1].wrapping_mul(self.registers[r2]);
                        self.zero_flag = self.registers[r1] == 0;
                    }
                    0x4 => {
                        if self.registers[r2] == 0 {
                            println!("\n[Err] Division by zero.");
                            self.running = false;
                        } else {
                            self.registers[r1] = self.registers[r1].wrapping_div(self.registers[r2]);
                            self.zero_flag = self.registers[r1] == 0;
                        }
                    }
                    0x5 => self.zero_flag = self.registers[r1] == self.registers[r2],
                    _ => self.running = false,
                },

                // 0x2: ALU Register-Immediate
                0x2 => match subop {
                    0x0 => self.registers[r1] = imm,
                    0x1 => {
                        self.registers[r1] = self.registers[r1].wrapping_add(imm);
                        self.zero_flag = self.registers[r1] == 0;
                    }
                    0x2 => {
                        self.registers[r1] = self.registers[r1].wrapping_sub(imm);
                        self.zero_flag = self.registers[r1] == 0;
                    }
                    0x3 => {
                        self.registers[r1] = self.registers[r1].wrapping_mul(imm);
                        self.zero_flag = self.registers[r1] == 0;
                    }
                    0x4 => {
                        if imm == 0 {
                            println!("\n[Err] Division by zero.");
                            self.running = false;
                        } else {
                            self.registers[r1] = self.registers[r1].wrapping_div(imm);
                            self.zero_flag = self.registers[r1] == 0;
                        }
                    }
                    0x5 => self.zero_flag = self.registers[r1] == imm,
                    _ => self.running = false,
                },

                // 0x3: Control Flow / Jumps
                0x3 => match subop {
                    0x0 => self.ip = addr,
                    0x1 => if self.zero_flag { self.ip = addr; },
                    0x2 => if !self.zero_flag { self.ip = addr; },
                    _ => self.running = false,
                },

                // 0x4: Memory Operations
                0x4 => match subop {
                    0x0 => { // loadb Vx, [Vy]
                        let a = self.registers[r2] as usize;
                        if a < self.memory.len() {
                            self.registers[r1] = self.memory[a] as u64;
                        } else {
                            println!("\n[Err] Out of bounds loadb at {:#X}", a);
                            self.running = false;
                        }
                    }
                    0x1 => { // storeb [Vy], Vx
                        let a = self.registers[r1] as usize;
                        if a < self.memory.len() {
                            self.memory[a] = (self.registers[r2] & 0xFF) as u8;
                        } else {
                            println!("\n[Err] Out of bounds storeb at {:#X}", a);
                            self.running = false;
                        }
                    }
                    0x2 => { // loadb Vx, [addr]
                        if addr < self.memory.len() {
                            self.registers[r1] = self.memory[addr] as u64;
                        } else {
                            println!("\n[Err] Out of bounds loadb at {:#X}", addr);
                            self.running = false;
                        }
                    }
                    0x3 => { // storeb [addr], Vx
                        if addr < self.memory.len() {
                            self.memory[addr] = (self.registers[r2] & 0xFF) as u8;
                        } else {
                            println!("\n[Err] Out of bounds storeb at {:#X}", addr);
                            self.running = false;
                        }
                    }
                    0x4 => { // load Vx, [Vy] (64-bit)
                        let a = self.registers[r2] as usize;
                        if a.checked_add(8).map_or(false, |end| end <= self.memory.len()) {
                            self.registers[r1] = u64::from_be_bytes(self.memory[a..a + 8].try_into().unwrap());
                        } else {
                            println!("\n[Err] Out of bounds load at {:#X}", a);
                            self.running = false;
                        }
                    }
                    0x5 => { // store [Vy], Vx (64-bit)
                        let a = self.registers[r1] as usize;
                        if a.checked_add(8).map_or(false, |end| end <= self.memory.len()) {
                            self.memory[a..a + 8].copy_from_slice(&self.registers[r2].to_be_bytes());
                        } else {
                            println!("\n[Err] Out of bounds store at {:#X}", a);
                            self.running = false;
                        }
                    }
                    0x6 => { // load Vx, [addr] (64-bit)
                        if addr.checked_add(8).map_or(false, |end| end <= self.memory.len()) {
                            self.registers[r1] = u64::from_be_bytes(self.memory[addr..addr + 8].try_into().unwrap());
                        } else {
                            println!("\n[Err] Out of bounds load at {:#X}", addr);
                            self.running = false;
                        }
                    }
                    0x7 => { // store [addr], Vx (64-bit)
                        if addr.checked_add(8).map_or(false, |end| end <= self.memory.len()) {
                            self.memory[addr..addr + 8].copy_from_slice(&self.registers[r2].to_be_bytes());
                        } else {
                            println!("\n[Err] Out of bounds store at {:#X}", addr);
                            self.running = false;
                        }
                    }
                    _ => self.running = false,
                },
                0x5 => match subop {
                    0x0 => { // push Vx
                        if self.sp >= 8 {
                            self.sp -= 8;
                            self.memory[self.sp..self.sp + 8]
                                .copy_from_slice(&self.registers[r1].to_be_bytes());
                        } else {
                            println!("\n[Err] Stack overflow on push.");
                            self.running = false;
                        }
                    }
                    0x1 => { // pop Vx
                        if self.sp + 8 <= self.memory.len() {
                            self.registers[r1] = u64::from_be_bytes(
                                self.memory[self.sp..self.sp + 8].try_into().unwrap(),
                            );
                            self.sp += 8;
                        } else {
                            println!("\n[Err] Stack underflow on pop.");
                            self.running = false;
                        }
                    }
                    0x2 => { // call addr
                        if self.sp >= 8 {
                            self.sp -= 8;
                            self.memory[self.sp..self.sp + 8]
                                .copy_from_slice(&(self.ip as u64).to_be_bytes());
                            self.ip = addr;
                        } else {
                            println!("\n[Err] Stack overflow on call.");
                            self.running = false;
                        }
                    }
                    0x3 => { // ret
                        if self.sp + 8 <= self.memory.len() {
                            let ret_addr = u64::from_be_bytes(
                                self.memory[self.sp..self.sp + 8].try_into().unwrap(),
                            );
                            self.sp += 8;
                            self.ip = ret_addr as usize;
                        } else {
                            println!("\n[Err] Stack underflow on ret.");
                            self.running = false;
                        }
                    }
                    _ => {
                        self.running = false;
                    }
                }
                _ => {
                        println!("\n[Err] Unknown opcode: {:#018X} at IP: {}", opcode, self.ip - 8);
                        self.running = false;
                }
            }
        }
        println!("\n---  END  ---");
    }
}