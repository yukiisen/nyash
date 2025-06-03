mod interpreter;
mod utils;
mod error;
mod args_parser;
mod readline;
mod trie;

use std::ffi::CString;

use interpreter::Interpreter;
use args_parser::parse_args;
use readline::Reader;

fn main() {
    let mut reader = Reader::new(); 
    let interpreter = Interpreter::new("./history");

    reader.update_trie(&interpreter.get_builtins());
    reader.update_trie(&utils::get_system_binaries());
    
    loop {
        // Wait for user input
        let mut input = reader.read_line("$ ");

        let command = parse_args(&input);

        if let Some(cmd) = command.first() {
            let found = interpreter.interpret_command(cmd, &command.iter().map(|e| e.as_str()).collect());
            
            if !found { println!("{}: command not found", cmd); }
            else {
                unsafe {
                    let history = interpreter.history;
                        input.push('\n');
                    let cmd = CString::new(input).unwrap();
                    let buf = cmd.as_bytes();

                    let written = libc::write(history, buf.as_ptr().cast(), buf.len());
                    if written == -1 {
                        let err = *libc::__errno_location();
                        eprintln!("Failed to write: {}", std::io::Error::from_raw_os_error(err));
                    }
                }
            }
        }
    }
}
