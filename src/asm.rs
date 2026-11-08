pub struct Asm;

impl Asm {
    pub fn parse(&self, code: &str) -> Result<Vec<u8>, String> {
        let mut program_bytes = Vec::new();

        for raw_line in code.lines() {
            let line = raw_line.trim();
            if line.is_empty() || line.starts_with(';') {
                continue;
            }

            // Parse
            let mut parts = line.splitn(2, |c: char| c.is_whitespace());
            let instr = parts.next().unwrap_or("").trim();
            let rest = parts.next().unwrap_or("").trim();

            match instr {
                "out" => {
                    // out "string"
                    if rest.starts_with('"') && rest.ends_with('"') {
                        let content = &rest[1..rest.len() - 1];
                        for c in content.chars() {
                            vpush_u16(&mut program_bytes, 0x01, c as u8);
                        }
                    } 
                    // out num
                    else if let Ok(val) = rest.parse::<u8>() {
                        vpush_u16(&mut program_bytes, 0x05, val);
                    }
                    // out Vreg
                    else if rest.starts_with('V') || rest.starts_with('v') {
                        let reg_str = &rest[1..];
                        let reg: u8 = reg_str.parse().map_err(|_| "Invalid register number")?;
                        if reg >= 16 {
                            return Err("Register index out of range (0-15)".to_string());
                        }
                        vpush_u16(&mut program_bytes, 0x03, reg);
                    } else {
                        return Err(format!("Invalid out syntax: '{}'", line));
                    }
                }
                "outn" => {
                    // outn num from Vreg
                    if rest.starts_with('V') || rest.starts_with('v') {
                        let reg_str = &rest[1..];
                        let reg: u8 = reg_str.parse().map_err(|_| "Invalid register number")?;
                        if reg >= 16 {
                            return Err("Register index out of range (0-15)".to_string());
                        }
                        vpush_u16(&mut program_bytes, 0x04, reg);
                    } else {
                        return Err(format!("Invalid outn syntax: '{}'", line));
                    }
                }
                "in" => {
                    // in Vreg
                    if rest.starts_with('V') || rest.starts_with('v') {
                        let reg_str = &rest[1..];
                        let reg: u8 = reg_str.parse().map_err(|_| "Invalid register number")?;
                        if reg >= 16 {
                            return Err("Register index out of range (0-15)".to_string());
                        }
                        let opcode: u16 = 0xB000 | ((reg as u16) << 8);
                        program_bytes.push((opcode >> 8) as u8);
                        program_bytes.push((opcode & 0xFF) as u8);
                    } else {
                        return Err(format!("Invalid in syntax: '{}'", line));
                    }
                }
                "mov" | "add" | "sub" | "mul" | "div" => {
                    let operands: Vec<&str> = rest.split(',').map(|s| s.trim()).collect();
                    if operands.len() == 2 && (operands[0].starts_with('V') || operands[0].starts_with('v')) {
                        let reg1: usize = operands[0][1..].parse().map_err(|_| "Invalid register")?;
                        if reg1 >= 16 {
                            return Err("Register index out of range (0-15)".to_string());
                        }

                        // Check if second is reg
                        if operands[1].starts_with('V') || operands[1].starts_with('v') {
                            let reg2: usize = operands[1][1..].parse().map_err(|_| "Invalid register")?;
                            if reg2 >= 16 {
                                return Err("Register index out of range (0-15)".to_string());
                            }
                            let opcode_base = match instr {
                                "mov" => 0x1000,
                                "add" => 0x2000,
                                "div" => 0x3000,
                                "mul" => 0x4000,
                                "sub" => 0x5000,
                                _ => unreachable!(),
                            };
                            let opcode: u16 = opcode_base | ((reg1 as u16) << 8) | ((reg2 as u16) << 4);
                            program_bytes.push((opcode >> 8) as u8);
                            program_bytes.push((opcode & 0xFF) as u8);
                        } else {
                            // else imm
                            let val: u8 = operands[1].parse().map_err(|_| "Invalid immediate value")?;
                            let opcode_base = match instr {
                                "mov" => 0x6000,
                                "add" => 0x7000,
                                "div" => 0x8000,
                                "mul" => 0x9000,
                                "sub" => 0xA000,
                                _ => unreachable!(),
                            };
                            let opcode: u16 = opcode_base | ((reg1 as u16) << 8) | (val as u16);
                            program_bytes.push((opcode >> 8) as u8);
                            program_bytes.push((opcode & 0xFF) as u8);
                        }
                    } else {
                        return Err(format!("Invalid {} syntax: '{}'", instr, line));
                    }

                }
                _ => {
                    return Err(format!("Unknown instruction: '{}'", line));
                }
            }
        }

        Ok(program_bytes)
    }
}

fn vpush_u16(vec: &mut Vec<u8>, high: u8, low: u8) {
    vec.push(high);
    vec.push(low);
}