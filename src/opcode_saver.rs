use crate::vm::{ Instruction, BranchLeaf };
use crate::linker::Executable;

use yaml_rust::yaml::Yaml;
use linked_hash_map::LinkedHashMap;

pub fn opcodes_into_yaml(opcodes : &[Instruction]) -> Yaml {
    let opcode_array : Vec<Yaml> =
        opcodes.iter()
        .map(
            |x| {
                match x {
                    Instruction::Ret => Yaml::String("ret".to_string()),
                    Instruction::Wait => Yaml::String("wait".to_string()),
                    Instruction::Jmp(x) => Yaml::Hash(vec![(Yaml::String("jmp".to_string()), Yaml::Integer(*x as i64))].into_iter().collect()),
                    Instruction::Msg(x) => Yaml::Hash(vec![(Yaml::String("msg".to_string()), Yaml::String(x.to_string()))].into_iter().collect()),
                    Instruction::PushPtr(x) => Yaml::Hash(vec![(Yaml::String("push_ptr".to_string()), Yaml::Integer(*x as i64))].into_iter().collect()),
                    Instruction::Branch(branches) =>
                        Yaml::Hash(
                            vec![(
                                Yaml::String("choose".to_string()),
                                Yaml::Array(
                                    branches.iter()
                                    .map(
                                        |BranchLeaf { jmp_address, option_name }|
                                        Yaml::Hash(vec![(Yaml::String(option_name.clone()), Yaml::Integer(*jmp_address as i64))].into_iter().collect())
                                    )
                                    .collect()
                                )
                            )].into_iter().collect()
                        )
                    ,
                }
            }
        )
        .collect()
    ;
    Yaml::Array(opcode_array)
}

pub fn entry_points_into_yaml(opcodes : &LinkedHashMap<String, usize>) -> Yaml {
    let the_map =
        opcodes.iter()
        .map(
            |x|
            (
                Yaml::String(x.0.clone()),
                Yaml::Integer(*x.1 as i64)
            )
        )
        .collect()
    ;
    Yaml::Hash(the_map)
}

pub fn executable_into_yaml(exe : &Executable) -> Vec<Yaml> {
    vec![entry_points_into_yaml(&exe.entry_points), opcodes_into_yaml(&exe.opcodes)]
}
