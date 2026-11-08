#[allow(deprecated)]
use std::io::{self, Write};

pub struct RVM {
    memory: [u8; 4096],
    ip: usize,
    running: bool,
    registres: [u8; 16],
    zero_flag: bool, // <-- Флаг для условий
}

impl RVM {
    pub fn new(regs: [u8; 16]) -> Self {
        RVM {
            memory: [0; 4096],
            ip: 512, // 0x200
            running: true,
            registres: regs,
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
            // 1. Fetch
            let opcode = ((self.memory[self.ip] as u16) << 8) | (self.memory[self.ip + 1] as u16);
            self.ip += 2;

            // 2. Decode & Execute
            match (opcode & 0xF000) >> 12 {
                0x0 => {
                    match (opcode & 0xFF00) >> 8 {
                        0x00 => { // HALT
                            self.running = false;
                        }
                        0x01 => { // PRINT_CHAR
                            let char_code = (opcode & 0xFF) as u8;
                            print!("{}", char_code as char);
                            io::stdout().flush().unwrap();
                        }
                        0x02 => { // NEWLINE
                            println!();
                        }
                        0x03 => { // PRINT_REG
                            let reg = (opcode & 0xFF) as usize;
                            if reg < 16 {
                                print!("{}", self.registres[reg] as char);
                                io::stdout().flush().unwrap();
                            }
                        }
                        0x04 => { // PRINT_REG_NUM
                            let reg = (opcode & 0xFF) as usize;
                            if reg < 16 {
                                print!("{}", self.registres[reg]);
                                io::stdout().flush().unwrap();
                            }
                        }
                        0x05 => { // PRINT_IMM_NUM
                            let val = (opcode & 0xFF) as u8;
                            print!("{}", val);
                            io::stdout().flush().unwrap();
                        }
                        0x06 => { // CMP reg, reg (0x06XY)
                            let x = ((opcode & 0x00F0) >> 4) as usize;
                            let y = (opcode & 0x000F) as usize;
                            self.zero_flag = self.registres[x] == self.registres[y];
                        }
                        _ => {
                            println!("\n[Err] Unknown opcode: {:#06X} at IP: {}", opcode, self.ip - 2);
                            self.running = false;
                        }
                    }
                }
                0x1 => { // MOV reg, reg
                    let x = ((opcode & 0x0F00) >> 8) as usize;
                    let y = ((opcode & 0x00F0) >> 4) as usize;
                    self.registres[x] = self.registres[y];
                }
                0x2 => { // ADD reg, reg
                    let x = ((opcode & 0x0F00) >> 8) as usize;
                    let y = ((opcode & 0x00F0) >> 4) as usize;
                    self.registres[x] = self.registres[x].wrapping_add(self.registres[y]);
                    self.zero_flag = self.registres[x] == 0;
                }
                0x3 => { // DIV reg, reg
                    let x = ((opcode & 0x0F00) >> 8) as usize;
                    let y = ((opcode & 0x00F0) >> 4) as usize;
                    if self.registres[y] == 0 {
                        println!("\n[Err] Division by zero at IP: {}", self.ip - 2);
                        self.running = false;
                    } else {
                        self.registres[x] = self.registres[x].wrapping_div(self.registres[y]);
                        self.zero_flag = self.registres[x] == 0;
                    }
                }
                0x4 => { // MUL reg, reg
                    let x = ((opcode & 0x0F00) >> 8) as usize;
                    let y = ((opcode & 0x00F0) >> 4) as usize;
                    self.registres[x] = self.registres[x].wrapping_mul(self.registres[y]);
                    self.zero_flag = self.registres[x] == 0;
                }
                0x5 => { // SUB reg, reg
                    let x = ((opcode & 0x0F00) >> 8) as usize;
                    let y = ((opcode & 0x00F0) >> 4) as usize;
                    self.registres[x] = self.registres[x].wrapping_sub(self.registres[y]);
                    self.zero_flag = self.registres[x] == 0;
                }
                0x6 => { // MOV reg, imm
                    let x = ((opcode & 0x0F00) >> 8) as usize;
                    let kk = (opcode & 0x00FF) as u8;
                    self.registres[x] = kk;
                }
                0x7 => { // ADD reg, imm
                    let x = ((opcode & 0x0F00) >> 8) as usize;
                    let kk = (opcode & 0x00FF) as u8;
                    self.registres[x] = self.registres[x].wrapping_add(kk);
                    self.zero_flag = self.registres[x] == 0;
                }
                0x8 => { // DIV reg, imm
                    let x = ((opcode & 0x0F00) >> 8) as usize;
                    let kk = (opcode & 0x00FF) as u8;
                    if kk == 0 {
                        println!("\n[Err] Division by zero at IP: {}", self.ip - 2);
                        self.running = false;
                    } else {
                        self.registres[x] = self.registres[x].wrapping_div(kk);
                        self.zero_flag = self.registres[x] == 0;
                    }
                }
                0x9 => { // MUL reg, imm
                    let x = ((opcode & 0x0F00) >> 8) as usize;
                    let kk = (opcode & 0x00FF) as u8;
                    self.registres[x] = self.registres[x].wrapping_mul(kk);
                    self.zero_flag = self.registres[x] == 0;
                }
                0xA => { // SUB reg, imm
                    let x = ((opcode & 0x0F00) >> 8) as usize;
                    let kk = (opcode & 0x00FF) as u8;
                    self.registres[x] = self.registres[x].wrapping_sub(kk);
                    self.zero_flag = self.registres[x] == 0;
                }
                0xB => { // IN reg
                    let x = ((opcode & 0x0F00) >> 8) as usize;
                    let mut input = String::new();
                    io::stdout().flush().unwrap();

                    if io::stdin().read_line(&mut input).is_ok() {
                        self.registres[x] = match input.trim().parse() {
                            Ok(num) => num,
                            Err(_) => {
                                println!("\n[Err] Invalid input. Expected number 0-255.");
                                self.running = false;
                                0
                            }
                        };
                    } else {
                        println!("\n[Err] Failed to read input.");
                        self.running = false;
                    }
                }
                0xC => { // JMP imm12 
                    self.ip = (opcode & 0x0FFF) as usize;
                }
                0xD => { // JZ imm12 
                    let addr = (opcode & 0x0FFF) as usize;
                    if self.zero_flag {
                        self.ip = addr;
                    }
                }
                0xE => { // JNZ imm12 
                    let addr = (opcode & 0x0FFF) as usize;
                    if !self.zero_flag {
                        self.ip = addr;
                    }
                }
                0xF => { // CMP reg, imm 
                    let x = ((opcode & 0x0F00) >> 8) as usize;
                    let kk = (opcode & 0x00FF) as u8;
                    self.zero_flag = self.registres[x] == kk;
                }
                _ => {
                    println!("\n[Err] Unknown opcode: {:#06X} at IP: {}", opcode, self.ip - 2);
                    self.running = false;
                }
            }
        }
        println!("\n---  END  ---");
    }
}