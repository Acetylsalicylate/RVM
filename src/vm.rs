#[allow(deprecated)]
use std::io::{self, Write};

pub struct RVM {
    memory: [u8; 65536],   // 64 KB memory
    ip: usize,             // Instruction pointer
    sp: usize,             // Stack pointer
    running: bool,
    registers: [u64; 16],  // 64-bit general purpose registers
    zero_flag: bool,
}

impl RVM {
    pub fn new(regs: [u64; 16]) -> Self {
        RVM {
            memory: [0; 65536],
            ip: 0x200,      // Code segment begins at 512 (0x200)
            sp: 65536,      // Stack grows downwards from the top of memory
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

    pub fn run(&mut self) {
        println!("--- Start ---");

        while self.running {
            if self.ip + 4 > self.memory.len() {
                println!("\n[Err] IP out of bounds: {}", self.ip);
                break;
            }

            // Fetch (32-bit big-endian instruction)
            let opcode = u32::from_be_bytes([
                self.memory[self.ip],
                self.memory[self.ip + 1],
                self.memory[self.ip + 2],
                self.memory[self.ip + 3],
            ]);
            self.ip += 4;

            let group = ((opcode >> 28) & 0x0F) as u8;
            let subop = ((opcode >> 24) & 0x0F) as u8;
            let r1    = ((opcode >> 20) & 0x0F) as usize;
            let r2    = ((opcode >> 16) & 0x0F) as usize;
            let imm   = (opcode & 0xFFFF) as u64;
            let addr  = (opcode & 0xFFFF) as usize;

            match group {
                // 0x0: System & I/O
                0x0 => match subop {
                    0x0 => self.running = false, // halt
                    0x1 => println!(),           // nl
                    0x2 => {                     // out char (imm)
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
                        if a + 8 <= self.memory.len() {
                            self.registers[r1] = u64::from_be_bytes(self.memory[a..a + 8].try_into().unwrap());
                        } else {
                            println!("\n[Err] Out of bounds load at {:#X}", a);
                            self.running = false;
                        }
                    }
                    0x5 => { // store [Vy], Vx (64-bit)
                        let a = self.registers[r1] as usize;
                        if a + 8 <= self.memory.len() {
                            self.memory[a..a + 8].copy_from_slice(&self.registers[r2].to_be_bytes());
                        } else {
                            println!("\n[Err] Out of bounds store at {:#X}", a);
                            self.running = false;
                        }
                    }
                    0x6 => { // load Vx, [addr] (64-bit)
                        if addr + 8 <= self.memory.len() {
                            self.registers[r1] = u64::from_be_bytes(self.memory[addr..addr + 8].try_into().unwrap());
                        } else {
                            println!("\n[Err] Out of bounds load at {:#X}", addr);
                            self.running = false;
                        }
                    }
                    0x7 => { // store [addr], Vx (64-bit)
                        if addr + 8 <= self.memory.len() {
                            self.memory[addr..addr + 8].copy_from_slice(&self.registers[r2].to_be_bytes());
                        } else {
                            println!("\n[Err] Out of bounds store at {:#X}", addr);
                            self.running = false;
                        }
                    }
                    _ => self.running = false,
                },

                // 0x5: Stack & Subroutine Operations
                0x5 => match subop {
                    0x0 => { // push Vx (64-bit register)
                        if self.sp < 8 {
                            println!("\n[Err] Stack overflow on push");
                            self.running = false;
                        } else {
                            self.sp -= 8;
                            self.memory[self.sp..self.sp + 8].copy_from_slice(&self.registers[r1].to_be_bytes());
                        }
                    }
                    0x1 => { // pop Vx (64-bit register)
                        if self.sp + 8 > self.memory.len() {
                            println!("\n[Err] Stack underflow on pop");
                            self.running = false;
                        } else {
                            self.registers[r1] = u64::from_be_bytes(
                                self.memory[self.sp..self.sp + 8].try_into().unwrap(),
                            );
                            self.sp += 8;
                        }
                    }
                    0x2 => { // push imm (64-bit value)
                        if self.sp < 8 {
                            println!("\n[Err] Stack overflow on push");
                            self.running = false;
                        } else {
                            self.sp -= 8;
                            self.memory[self.sp..self.sp + 8].copy_from_slice(&imm.to_be_bytes());
                        }
                    }
                    0x3 => { // call addr
                        if self.sp < 8 {
                            println!("\n[Err] Stack overflow on call");
                            self.running = false;
                        } else {
                            self.sp -= 8;
                            self.memory[self.sp..self.sp + 8].copy_from_slice(&(self.ip as u64).to_be_bytes());
                            self.ip = addr;
                        }
                    }
                    0x4 => { // ret
                        if self.sp + 8 > self.memory.len() {
                            println!("\n[Err] Stack underflow on ret");
                            self.running = false;
                        } else {
                            let return_ip = u64::from_be_bytes(
                                self.memory[self.sp..self.sp + 8].try_into().unwrap(),
                            ) as usize;
                            self.sp += 8;
                            self.ip = return_ip;
                        }
                    }
                    0x5 => { // call Vx (register indirect call)
                        let target = self.registers[r1] as usize;
                        if self.sp < 8 {
                            println!("\n[Err] Stack overflow on call");
                            self.running = false;
                        } else {
                            self.sp -= 8;
                            self.memory[self.sp..self.sp + 8].copy_from_slice(&(self.ip as u64).to_be_bytes());
                            self.ip = target;
                        }
                    }
                    _ => self.running = false,
                },

                _ => {
                    println!("\n[Err] Unknown opcode: {:#010X} at IP: {}", opcode, self.ip - 4);
                    self.running = false;
                }
            }
        }
        println!("\n---  END  ---");
    }
}