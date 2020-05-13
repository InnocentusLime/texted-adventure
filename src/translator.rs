// TODO: tail call optimization

use crate::parser::{ Ast, File };

use log::debug;
use linked_hash_map::LinkedHashMap;

pub struct BranchPreLeaf {
    pub option_name : String,
    pub jmp_address : usize,
}

pub enum PreInstruction {
    Ret,                            // a `restore` instruction. Jumps to the location pointed by the top
    Msg(String),                    // asks the host to write a message
    Jmp(usize),                    
    Wait,                           // asks the host to "flush" the messages (show them to the user) with "press X to continue"
    Branch(Vec<BranchPreLeaf>),     // offer the user to choose the branch. The vm then simple-jumps to the location
    PushPtr(usize),                 // put a pointer on the stack
    UnresolvedCall(String),
}

pub struct ObjectFiles {
    pub objects : LinkedHashMap<String, Vec<PreInstruction>>,
}

fn translate_ast_impl(ast : Ast, pre_opcodes : &mut Vec<PreInstruction>) {
    match ast {
        Ast::Msg(x) => pre_opcodes.push(PreInstruction::Msg(x)),
        Ast::Choice(choice_arr) => {
            let choice_instr_place = pre_opcodes.len(); // remember the location where to put the jump instruction
            pre_opcodes.push(PreInstruction::Ret); // Some dummy value which we'll update later

            let (leaves, place_holders) : (Vec<_>, Vec<_>) =  
                choice_arr.into_iter()
                .map(
                    |(option_name, code)| {
                        let jmp_address = pre_opcodes.len();
                        code.into_iter().for_each(|x| translate_ast_impl(x, pre_opcodes));
                        let aftermath_address = pre_opcodes.len();
                        pre_opcodes.push(PreInstruction::Ret);
                        (
                            BranchPreLeaf { option_name, jmp_address },
                            aftermath_address
                        )
                    }
                )
                .unzip()
            ;
            pre_opcodes[choice_instr_place] = PreInstruction::Branch(leaves);
            let after_choice = pre_opcodes.len();
            place_holders.into_iter().for_each(|x| pre_opcodes[x] = PreInstruction::Jmp(after_choice));
        },
        Ast::Wait => pre_opcodes.push(PreInstruction::Wait),
        Ast::Call(x) => {
            /*
                pre_opcodes.len()       points at `push_ptr`
                pre_opcodes.len() + 1   points at the `call`
                pre_opcodes.len() + 2   poinrs at the instruction we are interested in
            */
            let after_call = pre_opcodes.len() + 2;
            pre_opcodes.push(PreInstruction::PushPtr(after_call));
            pre_opcodes.push(PreInstruction::UnresolvedCall(x));
        },
    }
}

pub fn translate_ast(ast : Vec<Ast>) -> Vec<PreInstruction> {
    let mut pre_opcodes = Vec::new();
    for x in ast.into_iter() {
        translate_ast_impl(x, &mut pre_opcodes);
    }
    pre_opcodes.push(PreInstruction::Ret);
    debug!(target: "translator", "Done translating. {} pre opcodes processed", pre_opcodes.len());
    pre_opcodes
}

pub fn translate_file(file : File) -> ObjectFiles {
    ObjectFiles {
        objects : file.procs.into_iter()
        .map(
            |(k, v)| {
                debug!(target: "translator", "Translating \"{}\"...", k);
                (k, translate_ast(v))
            }
        )
        .collect(),
    }
}
