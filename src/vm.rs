#[allow(deprecated)]
use std::{io::{self, Write}, usize};

pub struct RVM {
    memory: [u8; 4096],
    ip: usize,
    running: bool,
    registres: [u8; 16],
}

impl RVM {
    pub fn new(regs: [u8; 16]) -> Self {
        RVM {
            memory: [0; 4096],
            ip: 512,
            running: true,
            registres: regs,
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
                0x00 => {
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
                        _ => {
                            println!("\n[Err] Unknown opcode: {:#06X} at IP: {}", opcode, self.ip - 2);
                            self.running = false;
                        }
                    }
                }
                0x06 => { 
                    // MOV reg, imm
                    let x = ((opcode & 0x0F00) >> 8) as usize;
                    let kk = (opcode & 0x00FF) as u8;
                    self.registres[x] = kk;
                }
                0x07 => { 
                    // ADD reg, imm
                    let x = ((opcode & 0x0F00) >> 8) as usize;
                    let kk = (opcode & 0x00FF) as u8;
                    self.registres[x] = self.registres[x].wrapping_add(kk);
                }
                _ => {
                    println!("\n[Err] Unknown opcode: {} at IP: {}", opcode, self.ip - 2);
                    self.running = false;
                }
            }
        }
        println!("\n---  END  ---");
    }
}