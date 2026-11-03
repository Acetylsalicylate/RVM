pub struct Asm;

impl Asm {
    pub fn parse(&self, code: &str) -> Result<Vec<u8>, String> {
        let mut program_bytes = Vec::new();

        for raw_line in code.lines() {
            let line = raw_line.trim();
            if line.is_empty() || line.starts_with(';') {
                continue;
            }

            // Разделяем строку на инструкцию и аргументы по любому количеству пробелов/табуляций
            let mut parts = line.splitn(2, |c: char| c.is_whitespace());
            let instr = parts.next().unwrap_or("").trim();
            let rest = parts.next().unwrap_or("").trim();

            match instr {
                "out" => {
                    // Вывод строки в кавычках: out "Hello"
                    if rest.starts_with('"') && rest.ends_with('"') {
                        let content = &rest[1..rest.len() - 1];
                        for c in content.chars() {
                            vpush_u16(&mut program_bytes, 0x01, c as u8);
                        }
                    } 
                    // Вывод числа напрямую: out 42
                    else if let Ok(val) = rest.parse::<u8>() {
                        vpush_u16(&mut program_bytes, 0x05, val);
                    }
                    // Вывод из регистра как символа: out V0
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
                    // Вывод числа из регистра: outn V0
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
                    // Ввод числа в регистр: in V0
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
                    let opcode_base = match instr {
                        "mov" => 0x6000,
                        "add" => 0x7000,
                        "div" => 0x8000,
                        "mul" => 0x9000,
                        "sub" => 0xA000,
                        _ => unreachable!(),
                    };
                    let operands: Vec<&str> = rest.split(',').map(|s| s.trim()).collect();
                    if operands.len() == 2 && (operands[0].starts_with('V') || operands[0].starts_with('v')) {
                        let reg: usize = operands[0][1..].parse().map_err(|_| "Invalid register")?;
                        if reg >= 16 {
                            return Err("Register index out of range (0-15)".to_string());
                        }
                        let val: u8 = operands[1].parse().map_err(|_| "Invalid immediate value")?;
                        let opcode: u16 = opcode_base | ((reg as u16) << 8) | (val as u16);
                        program_bytes.push((opcode >> 8) as u8);
                        program_bytes.push((opcode & 0xFF) as u8);
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