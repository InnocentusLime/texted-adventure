use crate::vm::{ Instruction, BranchLeaf };
use crate::linker::Executable;

use yaml_rust::yaml::Yaml;
use linked_hash_map::LinkedHashMap;

pub fn parse_yaml_opcode_impl(ast : Yaml) -> Instruction {
    match ast {
        Yaml::String(x) if x.trim() == "wait" => Instruction::Wait,
        Yaml::String(x) if x.trim() == "ret" => Instruction::Ret,
        Yaml::Hash(mut map) if map.len() == 1 => {
            match map.pop_back() {
                Some((Yaml::String(cmd), Yaml::String(msg))) if cmd.trim() == "msg" => Instruction::Msg(msg),
                Some((Yaml::String(cmd), Yaml::Integer(place))) if cmd.trim() == "jmp" && place >= 0 => Instruction::Jmp(place as usize),
                Some((Yaml::String(cmd), Yaml::Integer(place))) if cmd.trim() == "push_ptr" && place >= 0 => Instruction::PushPtr(place as usize),
                Some((Yaml::String(cmd), Yaml::Array(options))) if cmd.trim() == "choose" => {
                    let branches =
                    options.into_iter()
                    .map(
                        |x| {
                            match x {
                                Yaml::Hash(mut map) if map.len() == 1 => {
                                    match map.pop_back() {
                                        Some((Yaml::String(option_name), Yaml::Integer(jmp_address))) if jmp_address > 0 => BranchLeaf { option_name, jmp_address : jmp_address as usize },
                                        _ => panic!("In the choice map the key must be a string and the value must be an array"),
                                    }
                                },
                                _ => panic!("the choice branch must be a hash with one key-value pair"),
                            }
                        }
                    ).collect();
                    Instruction::Branch(branches)
                },
                _ => panic!("The command doesn't satisfy an possible format"),
            }
        },
        _ => panic!(),
    }
}

pub fn parse_yaml_opcode(yaml_ast : Yaml) -> Vec<Instruction> {
    if let Yaml::Array(instrs) = yaml_ast {
        instrs.into_iter()
        .map(&parse_yaml_opcode_impl).collect()
    } else { panic!("The file's root must be an array") }
}

pub fn parse_yaml_entry_table(yaml_ast : Yaml) -> LinkedHashMap<String, usize> {
    if let Yaml::Hash(entries) = yaml_ast {
        entries.into_iter()
        .map(
            |(k, v)| {
                match (k, v) {
                    (Yaml::String(name), Yaml::Integer(address)) => (name, address as usize),
                    _ => panic!("the entry table syntax is not satisfied"),
                }
            }
        ).collect()
    } else { panic!("The file's root must be a hashmap") }
}

pub fn parse_yaml_executable(mut file : Vec<Yaml>) -> Executable {
    // The first block is the entry table
    // The second block is the opcode array
    if file.len() < 2 { panic!("Not all blocks are present") }
    let opcodes = parse_yaml_opcode(file.pop().unwrap());
    let entry_points = parse_yaml_entry_table(file.pop().unwrap());
    Executable { opcodes, entry_points }
}
