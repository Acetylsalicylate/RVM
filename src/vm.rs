#[allow(deprecated)]
use std::io::{self, Write};

pub struct RVM {
    memory: [u8; 65536],   // 64 Kb
    ip: usize,
    running: bool,
    registers: [u64; 16],  
    zero_flag: bool,
}

impl RVM {
    pub fn new(regs: [u64; 16]) -> Self {
        RVM {
            memory: [0; 65536],
            ip: 0x200, // 512
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
                println!("\n[Err] IP out of memory bounds: {}", self.ip);
                break;
            }

            // 1. Fetch (32-битная инструкция / 4 байта)
            let opcode = u32::from_be_bytes([
                self.memory[self.ip],
                self.memory[self.ip + 1],
                self.memory[self.ip + 2],
                self.memory[self.ip + 3],
            ]);
            self.ip += 4;

            // 2. Распаковка полей
            let group  = ((opcode >> 28) & 0x0F) as u8;
            let subop  = ((opcode >> 24) & 0x0F) as u8;
            let r1     = ((opcode >> 20) & 0x0F) as usize;
            let r2     = ((opcode >> 16) & 0x0F) as usize;
            let imm    = (opcode & 0xFFFF) as u64;
            let addr   = (opcode & 0xFFFF) as usize;

            // 3. Decode & Execute
            match group {
                // 0x0: Системные и ввод/вывод
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
                    0x4 => {                     // outn reg (u64 число)
                        print!("{}", self.registers[r1]);
                        io::stdout().flush().unwrap();
                    }
                    0x5 => {                     // out imm (число)
                        print!("{}", imm);
                        io::stdout().flush().unwrap();
                    }
                    0x6 => {                     // in reg (64-битное число)
                        let mut input = String::new();
                        io::stdout().flush().unwrap();
                        if io::stdin().read_line(&mut input).is_ok() {
                            match input.trim().parse::<u64>() {
                                Ok(num) => self.registers[r1] = num,
                                Err(_) => {
                                    println!("\n[Err] Expected 64-bit integer (0-{}).", u64::MAX);
                                    self.running = false;
                                }
                            }
                        } else {
                            println!("\n[Err] Failed to read input.");
                            self.running = false;
                        }
                    }
                    _ => {
                        println!("\n[Err] Unknown SYS subop: {:#02X} at IP: {}", subop, self.ip - 4);
                        self.running = false;
                    }
                },

                // 0x1: АЛУ Регистр-Регистр (64 бита)
                0x1 => match subop {
                    0x0 => self.registers[r1] = self.registers[r2], // mov
                    0x1 => {                                        // add
                        self.registers[r1] = self.registers[r1].wrapping_add(self.registers[r2]);
                        self.zero_flag = self.registers[r1] == 0;
                    }
                    0x2 => {                                        // sub
                        self.registers[r1] = self.registers[r1].wrapping_sub(self.registers[r2]);
                        self.zero_flag = self.registers[r1] == 0;
                    }
                    0x3 => {                                        // mul
                        self.registers[r1] = self.registers[r1].wrapping_mul(self.registers[r2]);
                        self.zero_flag = self.registers[r1] == 0;
                    }
                    0x4 => {                                        // div
                        if self.registers[r2] == 0 {
                            println!("\n[Err] Division by zero at IP: {}", self.ip - 4);
                            self.running = false;
                        } else {
                            self.registers[r1] = self.registers[r1].wrapping_div(self.registers[r2]);
                            self.zero_flag = self.registers[r1] == 0;
                        }
                    }
                    0x5 => {                                        // cmp
                        self.zero_flag = self.registers[r1] == self.registers[r2];
                    }
                    _ => {
                        println!("\n[Err] Unknown ALU_RR subop: {:#02X} at IP: {}", subop, self.ip - 4);
                        self.running = false;
                    }
                },

                // 0x2: АЛУ Регистр-Константа (64 бита)
                0x2 => match subop {
                    0x0 => self.registers[r1] = imm,                // mov
                    0x1 => {                                        // add
                        self.registers[r1] = self.registers[r1].wrapping_add(imm);
                        self.zero_flag = self.registers[r1] == 0;
                    }
                    0x2 => {                                        // sub
                        self.registers[r1] = self.registers[r1].wrapping_sub(imm);
                        self.zero_flag = self.registers[r1] == 0;
                    }
                    0x3 => {                                        // mul
                        self.registers[r1] = self.registers[r1].wrapping_mul(imm);
                        self.zero_flag = self.registers[r1] == 0;
                    }
                    0x4 => {                                        // div
                        if imm == 0 {
                            println!("\n[Err] Division by zero at IP: {}", self.ip - 4);
                            self.running = false;
                        } else {
                            self.registers[r1] = self.registers[r1].wrapping_div(imm);
                            self.zero_flag = self.registers[r1] == 0;
                        }
                    }
                    0x5 => {                                        // cmp
                        self.zero_flag = self.registers[r1] == imm;
                    }
                    _ => {
                        println!("\n[Err] Unknown ALU_RI subop: {:#02X} at IP: {}", subop, self.ip - 4);
                        self.running = false;
                    }
                },

                // 0x3: Переходы (Jumps)
                0x3 => match subop {
                    0x0 => self.ip = addr,                               // jmp
                    0x1 => if self.zero_flag { self.ip = addr; },       // jz / je
                    0x2 => if !self.zero_flag { self.ip = addr; },      // jnz / jne
                    _ => {
                        println!("\n[Err] Unknown JMP subop: {:#02X} at IP: {}", subop, self.ip - 4);
                        self.running = false;
                    }
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