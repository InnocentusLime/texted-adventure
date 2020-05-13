mod vm;
mod opcode_saver;
mod opcode_loader;
mod linker;
mod parser;
mod client;
mod translator;

use parser::parse_yaml;
use translator::translate_file;
use linker::{ Executable, link };
use vm::Program;

use std::fs;
use std::io;
use io::Write;
use std::path::Path;

use regex::Regex;
use yaml_rust::emitter::YamlEmitter;
use yaml_rust::yaml::YamlLoader;
use clap::clap_app;
use log::debug;

fn compile_file<P : AsRef<Path>>(path : P) -> Executable {
        debug!(target: "compile_file", "reading file: \"{}\"", path.as_ref().to_string_lossy());
        let s = fs::read_to_string(path.as_ref()).unwrap();

        let mut res = YamlLoader::load_from_str(&s).unwrap();
        link(translate_file(parse_yaml(res.pop().unwrap())))
}

fn write_executable<P : AsRef<Path>>(path : P, exe : Executable) {
        debug!(target: "write_executable", "Outputting to file: {}", path.as_ref().to_string_lossy());
        let mut f = fs::File::create(&path).unwrap();

        let yamls = opcode_saver::executable_into_yaml(&exe);
        let (mut s1, mut s2) = (String::new(), String::new()); 
        let mut table_emitter = YamlEmitter::new(&mut s1);
        let mut opcode_emitter = YamlEmitter::new(&mut s2);
        table_emitter.dump(&yamls[0]).unwrap();
        opcode_emitter.dump(&yamls[1]).unwrap();
        
        write!(f, "{}\n{}", s1, s2).unwrap();
}

fn run_executable(exe : Executable, force_entry_choice : bool) {
        let entry_address = {
           if exe.entry_points.contains_key("main") && !force_entry_choice {
                exe.entry_points["main"]
           } else {
                if !force_entry_choice {
                    println!("no \"main\" entry point point. Please enter the entry point name.\nPossible entry points");
                } else {
                    println!("The engine was forced to launch the entry point choice mode. Please enter the entry point name.\nPossible entry points");
                }
                for (i, x) in exe.entry_points.keys().enumerate() {
                    println!("{}.) {}", i, x);
                }
                let id;
                let mut s = String::new();
                loop {
                    s.clear();
                    io::stdin().read_line(&mut s).unwrap();
                    if exe.entry_points.contains_key(s.trim()) {
                        id = exe.entry_points[s.trim()];
                        break;
                    }
                }
                id
           }
        };
        let program = Program::new(exe.opcodes, entry_address);
        let exec = program.run();
        client::stdio_client(exec);
}

fn main() {
    simple_logger::init().unwrap();

    let matches = 
    clap_app!(texted_adevnture =>
        (version: "0.1")
        (author: "Jasmine Katzenberg")
        (about: "A small text based game")
        (@subcommand run =>
            (about: "runs the game in the module which was forwarded to the engine")
            (@arg force_entry_choice: -f --force "forces the engine into entering the entry-point choice mode")
            (@arg path: +required "the path to the file")
        )
        (@subcommand compile =>
            (about: "compiles a file into assembly")
            (@arg path: +required "the path to the file")
        )
        (@subcommand crun =>
            (about: "quickly internally compile a dialogue file and run it")
            (@arg force_entry_choice: -f --force "forces the engine into entering the entry-point choice mode")
            (@arg path: +required "the path to the file")
        )
    ).get_matches();


    if let Some(matches) = matches.subcommand_matches("compile") {
        let path = matches.value_of("path").unwrap();
        let re = Regex::new(r#"(.+)\.diag"#).unwrap();
        let caps = re.captures(path).unwrap();
        
        let exe = compile_file(path);

        let mut out = String::new();
        out.push_str(&caps[1]);
        out.push_str(".asm");
        write_executable(out, exe);
    } 

    if let Some(matches) = matches.subcommand_matches("run") {
        let path = matches.value_of("path").unwrap();
        let re = Regex::new(r#"(.+)\.asm"#).unwrap();
        let _ = re.captures(path).unwrap();


        let file_contents = fs::read_to_string(path).unwrap();
        let yaml = YamlLoader::load_from_str(&file_contents).unwrap();
        let exe = opcode_loader::parse_yaml_executable(yaml);
        run_executable(exe, matches.is_present("force_entry_choice"));
    }

    if let Some(matches) = matches.subcommand_matches("crun") {
        let path = matches.value_of("path").unwrap();
        let re = Regex::new(r#"(.+)\.diag"#).unwrap();
        let _ = re.captures(path).unwrap();

        run_executable(compile_file(path), matches.is_present("force_entry_choice"));
    }
}
