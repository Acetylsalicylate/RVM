use std::collections::HashMap;

pub struct Asm;

impl Asm {
    pub fn parse(&self, code: &str) -> Result<Vec<u8>, String> {
        let base_offset: usize = 0x200; // 512
        let mut labels = HashMap::new();
        let mut program_bytes = Vec::new();

        // ------------------ collect labels ------------------
        let mut current_addr = base_offset;
        for raw_line in code.lines() {
            let line = raw_line.trim();
            if line.is_empty() || line.starts_with(';') {
                continue;
            }

            // check if label
            let (label, rest_line) = if let Some(colon_pos) = line.find(':') {
                if !line.starts_with("out") {
                    (Some(line[..colon_pos].trim()), line[colon_pos + 1..].trim())
                } else {
                    (None, line)
                }
            } else {
                (None, line)
            };

            if let Some(lbl) = label {
                if !lbl.is_empty() {
                    labels.insert(lbl.to_string(), current_addr as u16);
                }
            }

            if rest_line.is_empty() {
                continue;
            }

            // instruct size
            let mut parts = rest_line.splitn(2, |c: char| c.is_whitespace());
            let instr = parts.next().unwrap_or("").trim();
            let rest = parts.next().unwrap_or("").trim();

            if instr == "out" && rest.starts_with('"') && rest.ends_with('"') {
                let content = &rest[1..rest.len() - 1];
                current_addr += content.chars().count() * 2;
            } else {
                current_addr += 2;
            }
        }

        // ------------------ bytecode gen ------------------
        for raw_line in code.lines() {
            let line = raw_line.trim();
            if line.is_empty() || line.starts_with(';') {
                continue;
            }

            // cut label if in string
            let line = if let Some(colon_pos) = line.find(':') {
                if !line.starts_with("out") {
                    line[colon_pos + 1..].trim()
                } else {
                    line
                }
            } else {
                line
            };

            if line.is_empty() {
                continue;
            }

            let mut parts = line.splitn(2, |c: char| c.is_whitespace());
            let instr = parts.next().unwrap_or("").trim();
            let rest = parts.next().unwrap_or("").trim();

            match instr {
                "halt" => {
                    vpush_u16(&mut program_bytes, 0x00, 0x00);
                }
                "nl" => {
                    vpush_u16(&mut program_bytes, 0x02, 0x00);
                }
                "out" => {
                    if rest.starts_with('"') && rest.ends_with('"') {
                        let content = &rest[1..rest.len() - 1];
                        for c in content.chars() {
                            vpush_u16(&mut program_bytes, 0x01, c as u8);
                        }
                    } else if let Ok(val) = rest.parse::<u8>() {
                        vpush_u16(&mut program_bytes, 0x05, val);
                    } else if rest.starts_with('V') || rest.starts_with('v') {
                        let reg: u8 = rest[1..].parse().map_err(|_| "Invalid register number")?;
                        if reg >= 16 {
                            return Err("Register index out of range (0-15)".to_string());
                        }
                        vpush_u16(&mut program_bytes, 0x03, reg);
                    } else {
                        return Err(format!("Invalid out syntax: '{}'", line));
                    }
                }
                "outn" => {
                    if rest.starts_with('V') || rest.starts_with('v') {
                        let reg: u8 = rest[1..].parse().map_err(|_| "Invalid register number")?;
                        if reg >= 16 {
                            return Err("Register index out of range (0-15)".to_string());
                        }
                        vpush_u16(&mut program_bytes, 0x04, reg);
                    } else {
                        return Err(format!("Invalid outn syntax: '{}'", line));
                    }
                }
                "in" => {
                    if rest.starts_with('V') || rest.starts_with('v') {
                        let reg: u8 = rest[1..].parse().map_err(|_| "Invalid register number")?;
                        if reg >= 16 {
                            return Err("Register index out of range (0-15)".to_string());
                        }
                        let opcode: u16 = 0xB000 | ((reg as u16) << 8);
                        vpush_u16(&mut program_bytes, (opcode >> 8) as u8, (opcode & 0xFF) as u8);
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
                            vpush_u16(&mut program_bytes, (opcode >> 8) as u8, (opcode & 0xFF) as u8);
                        } else {
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
                            vpush_u16(&mut program_bytes, (opcode >> 8) as u8, (opcode & 0xFF) as u8);
                        }
                    } else {
                        return Err(format!("Invalid {} syntax: '{}'", instr, line));
                    }
                }
                "cmp" => {
                    let operands: Vec<&str> = rest.split(',').map(|s| s.trim()).collect();
                    if operands.len() == 2 && (operands[0].starts_with('V') || operands[0].starts_with('v')) {
                        let reg1: usize = operands[0][1..].parse().map_err(|_| "Invalid register")?;
                        if reg1 >= 16 {
                            return Err("Register index out of range (0-15)".to_string());
                        }

                        if operands[1].starts_with('V') || operands[1].starts_with('v') {
                            let reg2: usize = operands[1][1..].parse().map_err(|_| "Invalid register")?;
                            if reg2 >= 16 {
                                return Err("Register index out of range (0-15)".to_string());
                            }
                            // 0x06XY
                            vpush_u16(&mut program_bytes, 0x06, ((reg1 as u8) << 4) | (reg2 as u8));
                        } else {
                            let val: u8 = operands[1].parse().map_err(|_| "Invalid immediate value")?;
                            let opcode: u16 = 0xF000 | ((reg1 as u16) << 8) | (val as u16);
                            vpush_u16(&mut program_bytes, (opcode >> 8) as u8, (opcode & 0xFF) as u8);
                        }
                    } else {
                        return Err(format!("Invalid cmp syntax: '{}'", line));
                    }
                }
                "jmp" | "jz" | "je" | "jnz" | "jne" => {
                    let target = if let Some(&addr) = labels.get(rest) {
                        addr
                    } else if let Ok(addr) = rest.parse::<u16>() {
                        addr
                    } else {
                        return Err(format!("Unknown label or invalid address '{}' in: '{}'", rest, line));
                    };

                    if target >= 4096 {
                        return Err(format!("Jump target address out of bounds (0-4095): {}", target));
                    }

                    let prefix = match instr {
                        "jmp" => 0xC000,
                        "jz" | "je" => 0xD000,
                        "jnz" | "jne" => 0xE000,
                        _ => unreachable!(),
                    };

                    let opcode: u16 = prefix | (target & 0x0FFF);
                    vpush_u16(&mut program_bytes, (opcode >> 8) as u8, (opcode & 0xFF) as u8);
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