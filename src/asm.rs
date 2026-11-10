use std::collections::HashMap;

pub struct Asm;

impl Asm {
    pub fn parse(&self, code: &str) -> Result<Vec<u8>, String> {
        let base_offset: usize = 0x200; // 512
        let mut labels = HashMap::new();
        let mut program_bytes = Vec::new();

        // ------------------ 1. Сбор меток (инструкция = 4 байта) ------------------
        let mut current_addr = base_offset;
        for raw_line in code.lines() {
            let line = raw_line.trim();
            if line.is_empty() || line.starts_with(';') {
                continue;
            }

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

            let mut parts = rest_line.splitn(2, |c: char| c.is_whitespace());
            let instr = parts.next().unwrap_or("").trim();
            let rest = parts.next().unwrap_or("").trim();

            if instr == "out" && rest.starts_with('"') && rest.ends_with('"') {
                let content = &rest[1..rest.len() - 1];
                let unescaped_len = unescape(content).len();
                current_addr += unescaped_len * 4;
            } else {
                current_addr += 4;
            }
        }

        // ------------------ 2. Генерация байт-кода ------------------
        for raw_line in code.lines() {
            let line = raw_line.trim();
            if line.is_empty() || line.starts_with(';') {
                continue;
            }

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
                    vpush_u32(&mut program_bytes, encode(0x0, 0x0, 0, 0, 0));
                }
                "nl" => {
                    vpush_u32(&mut program_bytes, encode(0x0, 0x1, 0, 0, 0));
                }
                "out" => {
                    if rest.starts_with('"') && rest.ends_with('"') {
                        let content = &rest[1..rest.len() - 1];
                        for c in unescape(content) {
                            vpush_u32(&mut program_bytes, encode(0x0, 0x2, 0, 0, c as u32));
                        }
                    } else if let Ok(val) = rest.parse::<u16>() {
                        vpush_u32(&mut program_bytes, encode(0x0, 0x5, 0, 0, val as u32));
                    } else {
                        let reg = parse_reg(rest)?;
                        vpush_u32(&mut program_bytes, encode(0x0, 0x3, reg, 0, 0));
                    }
                }
                "outn" => {
                    let reg = parse_reg(rest)?;
                    vpush_u32(&mut program_bytes, encode(0x0, 0x4, reg, 0, 0));
                }
                "in" => {
                    let reg = parse_reg(rest)?;
                    vpush_u32(&mut program_bytes, encode(0x0, 0x6, reg, 0, 0));
                }
                "mov" | "add" | "sub" | "mul" | "div" => {
                    let subop = match instr {
                        "mov" => 0x0,
                        "add" => 0x1,
                        "sub" => 0x2,
                        "mul" => 0x3,
                        "div" => 0x4,
                        _ => unreachable!(),
                    };

                    let operands: Vec<&str> = rest.split(',').map(|s| s.trim()).collect();
                    if operands.len() != 2 {
                        return Err(format!("Invalid syntax for '{}': '{}'", instr, line));
                    }

                    let reg1 = parse_reg(operands[0])?;
                    if is_reg(operands[1]) {
                        let reg2 = parse_reg(operands[1])?;
                        vpush_u32(&mut program_bytes, encode(0x1, subop, reg1, reg2, 0));
                    } else {
                        let val: u16 = operands[1].parse().map_err(|_| "Invalid immediate (0-65535)")?;
                        vpush_u32(&mut program_bytes, encode(0x2, subop, reg1, 0, val as u32));
                    }
                }
                "cmp" => {
                    let operands: Vec<&str> = rest.split(',').map(|s| s.trim()).collect();
                    if operands.len() != 2 {
                        return Err(format!("Invalid cmp syntax: '{}'", line));
                    }

                    let reg1 = parse_reg(operands[0])?;
                    if is_reg(operands[1]) {
                        let reg2 = parse_reg(operands[1])?;
                        vpush_u32(&mut program_bytes, encode(0x1, 0x5, reg1, reg2, 0));
                    } else {
                        let val: u16 = operands[1].parse().map_err(|_| "Invalid immediate (0-65535)")?;
                        vpush_u32(&mut program_bytes, encode(0x2, 0x5, reg1, 0, val as u32));
                    }
                }
                "jmp" | "jz" | "je" | "jnz" | "jne" => {
                    let subop = match instr {
                        "jmp" => 0x0,
                        "jz" | "je" => 0x1,
                        "jnz" | "jne" => 0x2,
                        _ => unreachable!(),
                    };

                    let target = if let Some(&addr) = labels.get(rest) {
                        addr
                    } else if let Ok(addr) = rest.parse::<u16>() {
                        addr
                    } else {
                        return Err(format!("Unknown label or address '{}' in: '{}'", rest, line));
                    };

                    vpush_u32(&mut program_bytes, encode(0x3, subop, 0, 0, target as u32));
                }
                _ => return Err(format!("Unknown instruction: '{}'", line)),
            }
        }

        Ok(program_bytes)
    }
}

// Упаковщик 32-битной инструкции
fn encode(group: u32, subop: u32, r1: u32, r2: u32, imm: u32) -> u32 {
    ((group & 0x0F) << 28)
        | ((subop & 0x0F) << 24)
        | ((r1 & 0x0F) << 20)
        | ((r2 & 0x0F) << 16)
        | (imm & 0xFFFF)
}

fn vpush_u32(vec: &mut Vec<u8>, val: u32) {
    vec.extend_from_slice(&val.to_be_bytes());
}

fn is_reg(s: &str) -> bool {
    let s = s.trim();
    (s.starts_with('V') || s.starts_with('v')) && s[1..].parse::<u8>().is_ok()
}

fn parse_reg(s: &str) -> Result<u32, String> {
    let s = s.trim();
    if is_reg(s) {
        let reg: u32 = s[1..].parse().map_err(|_| "Invalid register number")?;
        if reg >= 16 {
            return Err("Register index out of range (0-15)".to_string());
        }
        Ok(reg)
    } else {
        Err(format!("Expected register (e.g. V0), got '{}'", s))
    }
}

fn unescape(s: &str) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => bytes.push(b'\n'),
                Some('r') => bytes.push(b'\r'),
                Some('t') => bytes.push(b'\t'),
                Some('\\') => bytes.push(b'\\'),
                Some('"') => bytes.push(b'"'),
                Some(other) => {
                    bytes.push(b'\\');
                    bytes.push(other as u8);
                }
                None => bytes.push(b'\\'),
            }
        } else {
            bytes.push(c as u8);
        }
    }

    bytes
}