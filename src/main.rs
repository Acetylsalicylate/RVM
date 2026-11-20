mod vm;
mod asm;

use vm::RVM;
use asm::Asm;
use std::{env, fs};

fn main() {

    let args: Vec<String> = env::args().collect();

    let file_path = if args.len() > 1 {
        &args[1]
    } else {
        "main.asm"
    };

    let code = fs::read_to_string(file_path).expect("Can't find main.asm");

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
    // vm.dump_memory_nonzero();
    // vm.dump_memory(512, 1024);
}