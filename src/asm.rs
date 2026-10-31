pub struct Asm;

impl Asm {
    pub fn parse(&self, code: &str) -> Result<Vec<u8>, String> {
        let mut program_bytes = Vec::new();

        for raw_line in code.lines() {
            let line = raw_line.trim();
            if line.is_empty() || line.starts_with(';') {
                continue;
            }

            if line.starts_with("out ") {
                let rest = line[4..].trim();
                
                // str
                if rest.starts_with('"') && rest.ends_with('"') {
                    let content = &rest[1..rest.len() - 1];
                    for c in content.chars() {
                        vpush_u16(&mut program_bytes, 0x01, c as u8);
                    }
                } 
                // from reg
                else if rest.starts_with('V') || rest.starts_with('v') {
                    let reg_str = &rest[1..];
                    let reg: u8 = reg_str.parse().map_err(|_| "Invalid register number")?;
                    if reg >= 16 {
                        return Err("Register index out of range (0-15)".to_string());
                    }
                    vpush_u16(&mut program_bytes, 0x03, reg);
                } else {
                    return Err(format!("Invalid syntax in line: '{}'", line));
                }
            } 
            else if line.starts_with("mov ") {
                let parts: Vec<&str> = line[4..].split(',').map(|s| s.trim()).collect();
                if parts.len() == 2 && (parts[0].starts_with('V') || parts[0].starts_with('v')) {
                    let reg: usize = parts[0][1..].parse().map_err(|_| "Invalid register")?;
                    if reg >= 16 {
                        return Err("Register index out of range (0-15)".to_string());
                    }
                    let val: u8 = parts[1].parse().map_err(|_| "Invalid immediate value")?;
                    let opcode: u16 = 0x6000 | ((reg as u16) << 8) | (val as u16);
                    program_bytes.push((opcode >> 8) as u8);
                    program_bytes.push((opcode & 0xFF) as u8);
                } else {
                    return Err(format!("Invalid mov syntax: '{}'", line));
                }
            }
            else {
                return Err(format!("Unknown instruction: '{}'", line));
            }
        }

        Ok(program_bytes)
    }
}

fn vpush_u16(vec: &mut Vec<u8>, high: u8, low: u8) {
    vec.push(high);
    vec.push(low);
}