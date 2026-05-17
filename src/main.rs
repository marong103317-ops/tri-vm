use std::env;
use std::fs;
use std::process;

use tri_vm::assembler;
use tri_vm::vm::VM;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: tri-vm <file.tri|file.tribin>");
        process::exit(1);
    }

    let path = &args[1];

    if path.ends_with(".tri") {
        // Assemble from source
        let source = match fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Error reading '{}': {}", path, e);
                process::exit(1);
            }
        };
        let binary = match assembler::assemble(&source) {
            Ok(b) => b,
            Err(e) => {
                eprintln!("Assembly error: {}", e);
                process::exit(1);
            }
        };
        let mut vm = VM::new();
        for (i, t) in binary.iter().enumerate() {
            if i < 59049 {
                vm.memory[i] = *t;
            } else {
                eprintln!("Program too large (max 59049 trytes)");
                process::exit(1);
            }
        }
        vm.run();
        print!("{}", vm.output);
        process::exit(vm.exit_code);
    } else {
        // Load raw binary (.tribin)
        let data = match fs::read(path) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("Error reading '{}': {}", path, e);
                process::exit(1);
            }
        };

        if data.len() < 2 || data.len() % 2 != 0 {
            eprintln!("File must contain an even number of bytes (i16 LE values)");
            process::exit(1);
        }

        let mut vm = VM::new();
        let mut pc = 0i32;
        for chunk in data.chunks(2) {
            let word = i16::from_le_bytes([chunk[0], chunk[1]]);
            if let Some(t) = tri_vm::tryte::Tryte::from_i16(word) {
                if pc < 59049 {
                    vm.memory[pc as usize] = t;
                    pc += 1;
                } else {
                    eprintln!("Program too large (max 59049 trytes)");
                    process::exit(1);
                }
            } else {
                eprintln!("Invalid tryte value: {} at offset {}", word, pc * 2);
                process::exit(1);
            }
        }
        vm.run();
        print!("{}", vm.output);
        process::exit(vm.exit_code);
    }
}
