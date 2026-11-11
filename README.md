# RVM (64-bit Rust Virtual Machine & Assembler)

A lightweight, educational 64-bit virtual machine and custom assembler written in Rust. It features a clean RISC-style architecture with 64-bit general-purpose registers, a 32-bit fixed instruction format, grouped opcodes, byte-addressable memory, direct memory load/store operations, and hardware stack support with subroutine calling conventions.

---

## Specifications & Features

- **64-bit Data Model:** All arithmetic and 16 general-purpose registers (`V0` through `V15`) operate on unsigned 64-bit integers (`u64`).
- **64 KB Flat Memory:** Byte-addressable memory space (`65,536` bytes) with programs loading at default offset `0x200` (512).
- **Hardware Stack & Subroutines:**
  - Dedicated internal Stack Pointer (`SP`) initialized to the memory ceiling (`0x10000` / 65536).
  - Stack grows downwards, aligned to 8 bytes (64-bit values and return addresses).
  - Runtime safety checks for **Stack Overflow** and **Stack Underflow**.
  - Built-in support for nested function calls (`call` and `ret`) saving the return address onto the stack.
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
  - `0x5`: Stack & Subroutines (`push`, `pop`, `call`, `ret`)
  - `0x6..0xF`: Reserved for future expansion.
- **Custom Assembler (`Asm`):**
  - Two-pass assembler with automatic label resolution for jumps, subroutines, and immediate data addresses.
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
| **Stack** | `push Vx` / `push imm` | Push 64-bit register value or 16-bit immediate onto stack |
| | `pop Vx` | Pop 64-bit value from stack into register `Vx` |
| | `call target` / `call Vx` | Push return address onto stack and branch to target |
| | `ret` | Pop return address from stack and jump to it |
| **Directives** | `db "text", 0` | Emit raw bytes into memory |

---

## Project Structure

```text
.
└── src/
    ├── main.rs   # Entry point (loads main.asm and starts RVM)
    ├── vm.rs     # 64-bit Virtual Machine (RVM) execution engine with SP
    └── asm.rs    # Two-pass assembler & instruction encoder
├── Cargo.toml
└── main.asm      # Assembly source code to run
```

---

## Example Program (`main.asm`)

The following program brute-forces characters one by one, writes each guess into memory, and displays an animated search until it produces `"Hello world!"`. It demonstrates modular programming using **subroutines (`call`/`ret`)** and **stack register preservation (`push`/`pop`)**:

```assembly
; ========================================================
; main.asm: iteration until "Hello world!" using stack ops
; ========================================================

start:
    mov V0, target_str      ; V0 = pointer to target string
    mov V1, buffer          ; V1 = pointer to buffer

char_loop:
    loadb V2, [V0]          ; Read target char from memory into V2
    cmp V2, 0               ; End of string (\0)?
    je done

    mov V3, 32              ; V3 = ASCII candidate starting from 32 (' ')

guess_loop:
    ; 1. Store candidate into buffer
    storeb [V1], V3

    ; 2. Temporarily write null terminator after candidate
    mov V4, V1
    add V4, 1
    mov V5, 0
    storeb [V4], V5

    ; 3. Print current buffer state using subroutine
    call print_buffer_sub

    ; Return cursor to beginning of line (carriage return, \r)
    mov V6, 13
    out V6

    ; 4. Call delay subroutine
    call delay_sub

    ; 5. Check if guessed
    cmp V3, V2
    je next_char

    ; Next candidate
    add V3, 1
    jmp guess_loop

next_char:
    add V0, 1
    add V1, 1
    jmp char_loop

done:
    ; Final clean output
    call print_buffer_sub
    nl                      
    halt

; --------------------------------------------------------
; Subroutine: print_buffer_sub
; Saves registers to stack, prints null-terminated buffer
; --------------------------------------------------------
print_buffer_sub:
    push V5
    push V6
    mov V5, buffer

pb_loop:
    loadb V6, [V5]
    cmp V6, 0
    je pb_done
    out V6
    add V5, 1
    jmp pb_loop

pb_done:
    pop V6
    pop V5
    ret

; --------------------------------------------------------
; Subroutine: delay_sub
; Simple busy-wait delay loop preserving V7 on stack
; --------------------------------------------------------
delay_sub:
    push V7
    mov V7, 65535
delay_loop:
    sub V7, 1
    jne delay_loop
    pop V7
    ret

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
**Pull requests are not being reviewed or accepted.**
You still can fork project or make an issue.
```
