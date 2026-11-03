mod vm;
mod asm;

use vm::RVM;
use asm::Asm;

fn main() {
    let code =
        r#"
        out "Enter a number: "
        in V0
        
        mov V1, 10
        add V0, 10
        outn V0
        out 10
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
