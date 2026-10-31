# RVM (Rust Virtual Machine & Assembler)

A lightweight, educational toy virtual machine and custom assembler written in Rust. It features a simple architecture with general-purpose registers, byte-addressable memory, and a basic assembler parser.

---

## Features

- **16 Registers:** General-purpose registers (`V0` through `V15`).
- **4KB Memory:** Flat memory space with a dedicated program offset (`0x200` / 512).
- **Custom Assembler (`Asm`):** Line-by-line parser supporting literals, registers, comments (`;`), and multi-line code blocks.
- **Supported Instructions:**
  - `mov reg, imm` — Move an immediate value into a register.
  - `out "string"` — Print a string literal.
  - `out reg` — Print the character value stored in a register.

---

## Project Structure

```text
src/
├── main.rs   # Entry point and program execution flow
├── vm.rs     # Virtual Machine implementation (RVM)
└── asm.rs    # Assembler parser implementation (Asm)
```

## Example Code
```rust
let code = r#"
    mov V0, 72   ; Load ASCII code for 'H' into V0
    out V0       ; Print value from V0
    out "ello World!"
"#;
```

## How to Run
1. Initialize or open your Cargo project, and place the files into the src/ directory.
2. Run the project using:
```bash
cargo run
```
