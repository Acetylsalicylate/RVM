mod vm;
mod asm;

use vm::RVM;
use asm::Asm;

fn main() {
    let code = r#"
        mov V0, 1

    loop:
        outn V0
        nl
        add V0, 1
        cmp V0, 6
        jnz loop

        halt
    "#;

    let mut vm = RVM::new([0; 16]);
    let asm = Asm;

    match asm.parse(code) {
        Ok(program_bytes) => {
            vm.load_program(&program_bytes, 0x200);
            vm.run();
        }
        Err(e) => {
            println!("Compile error: {}", e);
        }
    }
}
