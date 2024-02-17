use std::io;
use io::Read;

use crate::vm::{ ProgramExecutor, Request };

/// Stdio client is a simple implementation of the engine's
/// which is capable of running in the console.
pub fn stdio_client(exec : ProgramExecutor) {
    let mut exec = exec;
    let mut request = exec.unpause(None);

    // Welp, it's pretty much an event loop!
    loop {
        match request {
            Request::Drop => break,
            Request::Resume => { request = exec.unpause(None); },
            Request::PrintMessage(msg) => {
                println!("{}", msg);
                request = exec.unpause(None);
            },
            Request::Wait => {
                println!("\n[Ok]");
                io::stdin().read_exact(&mut [0]).unwrap();
                request = exec.done_printing(None);
            },
            Request::PerformChoice(choice_slice) => {
                println!("Pick an option");
                for (i, x) in choice_slice.iter().enumerate() {
                    println!("{}.) {}", i, x.option_name);
                }
                let mut s = String::new();
                let id;
                loop {
                    s.clear();
                    io::stdin().read_line(&mut s).unwrap();

                    if let Ok(x) = s.trim().parse() {
                        id = x;
                        break;
                    }
                }
                request = exec.choose(id, None);
            }
        }
    }
}
