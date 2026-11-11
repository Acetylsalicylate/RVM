use std::collections::HashMap;

pub struct Asm;

enum AddrOp {
    Reg(u32),
    Imm(u16),
}

impl Asm {
    pub fn parse(&self, code: &str) -> Result<Vec<u8>, String> {
        let base_offset: usize = 0x200; // 512
        let mut labels = HashMap::new();
        let mut program_bytes = Vec::new();

        // ------------------ 1. First pass: collect labels ------------------
        let mut current_addr = base_offset;
        for raw_line in code.lines() {
            // Strip comments and surrounding whitespace
            let line = strip_comment(raw_line).trim();
            if line.is_empty() {
                continue;
            }

            let (label, rest_line) = split_label(line);

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

            if instr == "db" {
                let db_bytes = parse_db(rest)?;
                current_addr += db_bytes.len();
            } else if instr == "out" && rest.starts_with('"') && rest.ends_with('"') {
                let content = &rest[1..rest.len() - 1];
                current_addr += unescape(content).len() * 4;
            } else {
                // All other instructions take 4 bytes (including push, pop, call, ret)
                current_addr += 4;
            }
        }

        // ------------------ 2. Second pass: generate bytecode ------------------
        for raw_line in code.lines() {
            let line = strip_comment(raw_line).trim();
            if line.is_empty() {
                continue;
            }

            let (_, line) = split_label(line);
            if line.is_empty() {
                continue;
            }

            let mut parts = line.splitn(2, |c: char| c.is_whitespace());
            let instr = parts.next().unwrap_or("").trim();
            let rest = parts.next().unwrap_or("").trim();

            match instr {
                "db" => {
                    let db_bytes = parse_db(rest)?;
                    program_bytes.extend(db_bytes);
                }
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
                    } else if let Ok(reg) = parse_reg(rest) {
                        vpush_u32(&mut program_bytes, encode(0x0, 0x3, reg, 0, 0));
                    } else {
                        let val = parse_imm_or_label(rest, &labels)?;
                        vpush_u32(&mut program_bytes, encode(0x0, 0x5, 0, 0, val as u32));
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
                        let val = parse_imm_or_label(operands[1], &labels)?;
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
                        let val = parse_imm_or_label(operands[1], &labels)?;
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

                    let target = parse_imm_or_label(rest, &labels)?;
                    vpush_u32(&mut program_bytes, encode(0x3, subop, 0, 0, target as u32));
                }
                "loadb" | "load" => {
                    let is_byte = instr == "loadb";
                    let operands: Vec<&str> = rest.split(',').map(|s| s.trim()).collect();
                    if operands.len() != 2 {
                        return Err(format!("Invalid syntax for {}: '{}'", instr, line));
                    }

                    let reg_x = parse_reg(operands[0])?;
                    let addr_op = parse_addr_op(operands[1], &labels)?;

                    match addr_op {
                        AddrOp::Reg(reg_y) => {
                            let subop = if is_byte { 0x0 } else { 0x4 };
                            vpush_u32(&mut program_bytes, encode(0x4, subop, reg_x, reg_y, 0));
                        }
                        AddrOp::Imm(addr) => {
                            let subop = if is_byte { 0x2 } else { 0x6 };
                            vpush_u32(&mut program_bytes, encode(0x4, subop, reg_x, 0, addr as u32));
                        }
                    }
                }
                "storeb" | "store" => {
                    let is_byte = instr == "storeb";
                    let operands: Vec<&str> = rest.split(',').map(|s| s.trim()).collect();
                    if operands.len() != 2 {
                        return Err(format!("Invalid syntax for {}: '{}'", instr, line));
                    }

                    let addr_op = parse_addr_op(operands[0], &labels)?;
                    let reg_x = parse_reg(operands[1])?;

                    match addr_op {
                        AddrOp::Reg(reg_y) => {
                            let subop = if is_byte { 0x1 } else { 0x5 };
                            vpush_u32(&mut program_bytes, encode(0x4, subop, reg_y, reg_x, 0));
                        }
                        AddrOp::Imm(addr) => {
                            let subop = if is_byte { 0x3 } else { 0x7 };
                            vpush_u32(&mut program_bytes, encode(0x4, subop, 0, reg_x, addr as u32));
                        }
                    }
                }
                // ------------------ Stack Operations ------------------
                "push" => {
                    if is_reg(rest) {
                        let reg = parse_reg(rest)?;
                        vpush_u32(&mut program_bytes, encode(0x5, 0x0, reg, 0, 0));
                    } else {
                        let val = parse_imm_or_label(rest, &labels)?;
                        vpush_u32(&mut program_bytes, encode(0x5, 0x2, 0, 0, val as u32));
                    }
                }
                "pop" => {
                    let reg = parse_reg(rest)?;
                    vpush_u32(&mut program_bytes, encode(0x5, 0x1, reg, 0, 0));
                }
                "call" => {
                    if is_reg(rest) {
                        let reg = parse_reg(rest)?;
                        vpush_u32(&mut program_bytes, encode(0x5, 0x5, reg, 0, 0));
                    } else {
                        let target = parse_imm_or_label(rest, &labels)?;
                        vpush_u32(&mut program_bytes, encode(0x5, 0x3, 0, 0, target as u32));
                    }
                }
                "ret" => {
                    vpush_u32(&mut program_bytes, encode(0x5, 0x4, 0, 0, 0));
                }
                _ => return Err(format!("Unknown instruction: '{}'", line)),
            }
        }

        Ok(program_bytes)
    }
}

// Strip line comments starting from ';' up to end of line (respecting quotes)
fn strip_comment(line: &str) -> &str {
    let mut in_quotes = false;
    for (i, c) in line.char_indices() {
        if c == '"' {
            in_quotes = !in_quotes;
        } else if c == ';' && !in_quotes {
            return &line[..i];
        }
    }
    line
}

// Split label definition (respecting colons inside quotes)
fn split_label(line: &str) -> (Option<&str>, &str) {
    let mut in_quotes = false;
    for (i, c) in line.char_indices() {
        if c == '"' {
            in_quotes = !in_quotes;
        } else if c == ':' && !in_quotes {
            return (Some(line[..i].trim()), line[i + 1..].trim());
        }
    }
    (None, line)
}

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

fn parse_imm_or_label(s: &str, labels: &HashMap<String, u16>) -> Result<u16, String> {
    let s = s.trim();
    if let Some(&addr) = labels.get(s) {
        Ok(addr)
    } else if let Ok(val) = s.parse::<u16>() {
        Ok(val)
    } else if s.starts_with("0x") || s.starts_with("0X") {
        u16::from_str_radix(&s[2..], 16).map_err(|_| format!("Invalid hex: '{}'", s))
    } else {
        Err(format!("Invalid immediate value or unknown label: '{}'", s))
    }
}

fn parse_addr_op(s: &str, labels: &HashMap<String, u16>) -> Result<AddrOp, String> {
    let clean = s.trim().trim_start_matches('[').trim_end_matches(']').trim();
    if is_reg(clean) {
        let reg = parse_reg(clean)?;
        Ok(AddrOp::Reg(reg))
    } else {
        let addr = parse_imm_or_label(clean, labels)?;
        Ok(AddrOp::Imm(addr))
    }
}

fn parse_db(rest: &str) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;

    for c in rest.chars() {
        if c == '"' {
            in_quotes = !in_quotes;
            current.push(c);
        } else if c == ',' && !in_quotes {
            tokens.push(current.trim().to_string());
            current.clear();
        } else {
            current.push(c);
        }
    }
    if !current.trim().is_empty() {
        tokens.push(current.trim().to_string());
    }

    for token in tokens {
        let t = token.trim();
        if t.starts_with('"') && t.ends_with('"') && t.len() >= 2 {
            let content = &t[1..t.len() - 1];
            bytes.extend(unescape(content));
        } else if let Ok(val) = t.parse::<u64>() {
            bytes.push(val as u8);
        } else {
            return Err(format!("Invalid db item: '{}'", t));
        }
    }
    Ok(bytes)
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