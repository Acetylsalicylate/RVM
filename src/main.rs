mod vm;
mod asm;

use vm::RVM;
use asm::Asm;
use std::fs;

fn main() {
    let code = fs::read_to_string("main.asm").expect("Can't find file");

    let mut vm = RVM::new([0u64; 16]);
    let asm = Asm;

    match asm.parse(&code) {
        Ok(program_bytes) => {
            vm.load_program(&program_bytes, 0x200);
            vm.run();
        }
        Err(e) => {
            println!("Compile error: {}", e);
        }
    }
}