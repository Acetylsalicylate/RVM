# RVM (64-bit Rust Virtual Machine & Assembler)

A lightweight, educational 64-bit virtual machine and custom assembler written in Rust. It features a clean RISC-style architecture with 64-bit general-purpose registers, a 32-bit fixed instruction format, grouped opcodes, byte-addressable memory, and direct memory load/store operations.

---

## Specifications & Features

- **64-bit Data Model:** All arithmetic and 16 general-purpose registers (`V0` through `V15`) operate on unsigned 64-bit integers (`u64`).
- **64 KB Flat Memory:** Byte-addressable memory space (`65,536` bytes) with programs loading at default offset `0x200` (512).
- **Fixed 32-bit Instruction Format:** Instructions are fixed-width (4 bytes, big-endian) with clean field grouping:
```text
[31..28] Group (Major Opcode)  — 4 bits
[27..24] Sub-Opcode            — 4 bits
[23..20] Register 1 (Dst)      — 4 bits
[19..16] Register 2 (Src)      — 4 bits
[15..0]  Immediate / Address   — 16 bits
```
- **Grouped Opcode Architecture:** Instructions are organized into logical groups without exhausting opcode space:
  - `0x0`: System & I/O (`halt`, `nl`, `out`, `outn`, `in`)
  - `0x1`: ALU Register-Register (`mov`, `add`, `sub`, `mul`, `div`, `cmp`)
  - `0x2`: ALU Register-Immediate (`mov`, `add`, `sub`, `mul`, `div`, `cmp` with 16-bit values or labels)
  - `0x3`: Control Flow / Jumps (`jmp`, `jz`/`je`, `jnz`/`jne`)
  - `0x4`: Memory Operations (`loadb`, `storeb`, `load`, `store`)
  - `0x5..0xF`: Reserved for future expansion.
- **Custom Assembler (`Asm`):**
  - Two-pass assembler with automatic label resolution for jumps and immediate data addresses.
  - Full-line and inline comment stripping (`;`), safely ignoring semicolons inside strings.
  - Data directive `db` for raw bytes and strings (e.g. `db "Hello", 0`).
  - Bracket notation support for memory addressing (e.g. `[V1]`, `[target]`).

---

## Instruction Set Overview

| Instruction | Syntax Example | Description |
| :--- | :--- | :--- |
| **System** | `halt` | Stop virtual machine execution |
| | `nl` | Print newline (`\n`) |
| **I/O** | `out "text"` | Print a string literal |
| | `out Vx` | Print character from register `Vx` |
| | `out imm` | Print raw character or number |
| | `outn Vx` | Print full 64-bit integer stored in `Vx` |
| | `in Vx` | Read 64-bit integer from stdin into `Vx` |
| **ALU** | `mov Vx, Vy` / `mov Vx, imm` | Copy register or load 16-bit immediate / label |
| | `add Vx, Vy` / `add Vx, imm` | 64-bit addition (wrapping) |
| | `sub Vx, Vy` / `sub Vx, imm` | 64-bit subtraction (wrapping) |
| | `mul Vx, Vy` / `mul Vx, imm` | 64-bit multiplication (wrapping) |
| | `div Vx, Vy` / `div Vx, imm` | 64-bit division (zero-division guarded) |
| | `cmp Vx, Vy` / `cmp Vx, imm` | Compare operands and set zero flag (`ZF`) |
| **Branching** | `jmp target` | Unconditional jump to address / label |
| | `jz target` / `je target` | Jump if `ZF == true` (Zero / Equal) |
| | `jnz target` / `jne target`| Jump if `ZF == false` (Not Zero / Not Equal) |
| **Memory** | `loadb Vx, [Vy]` / `[addr]` | Load 1 byte from memory into `Vx` |
| | `storeb [Vy], Vx` / `[addr]` | Store lowest byte of `Vx` into memory |
| | `load Vx, [Vy]` / `[addr]` | Load 64-bit word (8 bytes) into `Vx` |
| | `store [Vy], Vx` / `[addr]` | Store 64-bit word (8 bytes) from `Vx` into memory |
| **Directives** | `db "text", 0` | Emit raw bytes into memory |

---

## Project Structure

```text
.
└── src/
    ├── main.rs   # Entry point (loads main.asm and starts RVM)
    ├── vm.rs     # 64-bit Virtual Machine (RVM) execution engine
    └── asm.rs    # Two-pass assembler & instruction encoder
├── Cargo.toml
└── main.asm      # Assembly source code to run
```

---

## Example Program (`main.asm`)

The following program brute-forces characters one by one, writes each guess into memory, and displays an animated search until it produces `"Hello world!"`:

```assembly
start:
    mov V0, target_str      ; V0 = pointer to target string in memory
    mov V1, buffer          ; V1 = pointer to output buffer in memory

char_loop:
    loadb V2, [V0]          ; Read target character
    cmp V2, 0               ; Check for null terminator
    je done

    mov V3, 32              ; Start brute-force from ASCII 32 (' ')

guess_loop:
    storeb [V1], V3         ; Write candidate into memory

    ; Temporarily null-terminate the buffer for printing
    mov V4, V1
    add V4, 1
    mov V5, 0
    storeb [V4], V5

    ; Print current buffer from memory
    mov V5, buffer
print_buf:
    loadb V6, [V5]
    cmp V6, 0
    je print_cr
    out V6
    add V5, 1
    jmp print_buf

print_cr:
    mov V6, 13              ; Carriage return (\r) to overwrite line
    out V6

    ; Delay loop for visual effect
    mov V7, 65535
delay:
    sub V7, 1
    jne delay

    ; Check if candidate matches target character
    cmp V3, V2
    je next_char

    add V3, 1               ; Try next ASCII character
    jmp guess_loop

next_char:
    add V0, 1               ; Advance target pointer
    add V1, 1               ; Advance buffer pointer
    jmp char_loop

done:
    ; Final print of completed string
    mov V5, buffer
final_print:
    loadb V6, [V5]
    cmp V6, 0
    je finish
    out V6
    add V5, 1
    jmp final_print

finish:
    nl                      ; Print newline and halt
    halt

; Data Section
target_str:
    db "Hello world!", 0

buffer:
    db 0
```

---

## How to Run

1. Place your code in `main.asm` in the root of the project.
2. Build and run using Cargo:
```bash
cargo run --release
```

---

## 🤝 Contributing
Please note that this repository is maintained solely as an educational and portfolio project. **Pull requests are not being reviewed or accepted.**
You still can fork project or make an issue.
