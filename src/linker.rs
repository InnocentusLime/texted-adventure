use crate::vm::{ BranchLeaf, Instruction };
use crate::translator::{ BranchPreLeaf, PreInstruction, ObjectFiles };

use log::debug;
use linked_hash_map::LinkedHashMap;

pub struct Executable {
    pub entry_points : LinkedHashMap<String, usize>,
    pub opcodes : Vec<Instruction>,
}

pub fn link(mut files : ObjectFiles) -> Executable {
    debug!(target: "linker", "Linking...");
    let entry_point_ordering = 
        files.objects.keys()
        .map(|x| (*x).clone())
        .collect::<Vec<_>>()
    ;
    
    debug!(target: "linker", "Creating entry point table...");
    let mut ptr = 0; // this ptr points at the last untouched opcode location
    // setting up entry points
    let entry_points : LinkedHashMap<String, usize> =
        entry_point_ordering.iter()
        .map(
            |name| {
                debug!(target: "linker", "entry point \"{}\" at {}", name, ptr);
                let my_address = ptr;
                ptr += files.objects[name].len();
                (name.clone(), my_address)
            }
        )
        .collect::<LinkedHashMap<_, _>>()
    ;

    debug!(target: "linker", "Resolving symbols...");
    // resolution
    let mut opcodes = Vec::with_capacity(ptr);
    for name in entry_point_ordering.iter() {
        let pre_opcodes = files.objects.remove(name).unwrap();
        for pre_opcode in pre_opcodes.into_iter() {
            let opcode = 
                match pre_opcode {
                    PreInstruction::Ret => Instruction::Ret,
                    PreInstruction::Msg(x) => Instruction::Msg(x),
                    PreInstruction::Wait => Instruction::Wait,
                    PreInstruction::Branch(x) => 
                        Instruction::Branch(
                            x.into_iter()
                            .map(
                                |BranchPreLeaf {option_name, jmp_address}| 
                                BranchLeaf {option_name, jmp_address : jmp_address + entry_points[name]}
                            )
                            .collect()
                        ),
                    PreInstruction::PushPtr(x) => Instruction::PushPtr(x + entry_points[name]),
                    PreInstruction::Jmp(x) => Instruction::Jmp(x + entry_points[name]),
                    PreInstruction::UnresolvedCall(x) => {
                        match entry_points.get(&x) {
                            Some(x) => Instruction::Jmp(*x),
                            None => panic!("Unknown dialogue \"{}\"", x),
                        }
                    },
                }
            ;
            opcodes.push(opcode);
        }
    }

    debug!(target: "linker", "Done linking. {} opcodes processed", opcodes.len());
    Executable {
        entry_points,
        opcodes,
    }
}
