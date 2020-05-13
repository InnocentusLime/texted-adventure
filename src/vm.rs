#[derive(Debug)]
pub struct BranchLeaf {
    /// The string which will be seen by the user
    /// when they are asked to pick an option
    pub option_name : String,
    /// The address where VM will jump
    pub jmp_address : usize,
}

#[derive(Debug)]
pub enum Instruction {
    /// This a basic return. It either jump to
    /// the location pointed by the top of the
    /// `frame_stack`, or makes the vm terminate
    Ret,                        

    /// A simple unconditional jump
    Jmp(usize),                 

    /// This instruction makes the vm ask the
    /// client to print a message
    Msg(String),                

    /// This instruction makes the vm as the 
    /// client to "pause"
    Wait,                       

    /// This is a simple branching. It offers
    /// the client to pic an option. When they
    /// pick the option, the vm jumps to the
    /// corresponding piece of code (see the `BranchLeaf` struct)
    Branch(Vec<BranchLeaf>),    

    /// This instruction puts a point on the 
    /// frame stuck
    PushPtr(usize),             
}

/// A `Program` is what our VM runs. To run a program an entry point
/// must be given --- place in the code where VM must start
pub struct Program {
    entry_point : usize,
    opcodes : Vec<Instruction>,
}

impl Program {
    /// The constructor
    pub fn new(opcodes : Vec<Instruction>, entry_point : usize) -> Program {
        Program {
            opcodes,
            entry_point,
        }
    }

    /// Create a VM instance
    pub fn run(&self) -> ProgramExecutor {
        ProgramExecutor {
            my_program : self,
            instruction_ptr : self.entry_point,
            frame_stack : Vec::new(),
            state : ProgramState::Paused,
        }
    }
}

// Specification
//  IF state = WaitingForChoice then the vm's 
//      instruction ptr is points at a `Branch` instruction
//  ELSE nothing
#[derive(Clone, Copy, PartialEq, Eq)]
enum ProgramState {
    Waiting,
    WaitingForChoice,
    Paused,
    Terminated,
}

/// A request is what the VM wants the client to do
pub enum Request<'a> {
    /// The client must shutdown all the systems
    /// which are waiting for commands from the VM.
    /// The sessions has ended.
    Drop,

    /// The VM was paused for whatever reason. The
    /// client just should send a "ok" signal. Such
    /// requests appear only when the VM pauses itself
    /// because of instruction limit.
    Resume,

    /// The client must print a message. 
    PrintMessage(&'a str),

    /// The client must "pause". This is a different concept: 
    /// In terms of CLI
    /// they must wait for the user to press "Enter".
    /// In case of some kind of GUI that should be
    /// interpreted as showing an "OK" button.
    /// The client must send the VM a special singal
    /// indicating that the user "unspaused" it.
    Wait,

    /// The VM has encouterd a branch and needs
    /// the client to pick an option. It will be waiting
    /// for a signal with a branch id.
    PerformChoice(&'a [BranchLeaf]),
}

/// The VM instance
pub struct ProgramExecutor<'a> {
    my_program : &'a Program,
    instruction_ptr : usize,
    frame_stack : Vec<usize>,
    state : ProgramState,
}

impl<'a> ProgramExecutor<'a> {
    // The heart of our VM. The user will never see this
    // function
    fn execute(&mut self, limit : Option<usize>) -> Request<'a> {
        // The limit of the opcodes is thse `usize` max if the user said
        // that there's no limit. :)
        let mut limit = limit.unwrap_or(std::usize::MAX);
        // Let me note that I am creating an auxillary variable for the
        // instruction ptr... Why not?
        let mut instruction_ptr = self.instruction_ptr;
        let mut request = None;

        // The user tried to wake us up when we did all the job!
        if self.state == ProgramState::Terminated { panic!("Can't continue") }

        // the main loop.
        // Loop invariant: 
        //  if the looping condition is true, then the VM is pointing
        //  on a new opcode or a new pointer was pushed on the frame stack
        while limit > 0 && request.is_none() {
            // setting up some aliases.
            let opcodes = &self.my_program.opcodes;
            let frame_stack = &mut self.frame_stack;

            // fetching an opcode
            match opcodes.get(instruction_ptr) {
                // okay. We succeded
                Some(instruction) => {
                    // Note that not all instructions move
                    // the `instruction_ptr`. This is not
                    // an oversight.
                    match instruction {
                        Instruction::Ret => {
                            match frame_stack.pop() {
                                // There's a frame on stack. Jump there
                                Some(x) => { instruction_ptr = x; },
                                // No frames left. End of exection!
                                None => { request = Some(Request::Drop); },
                            }
                        },
                        Instruction::Jmp(x) => {
                            // It is hard to explain why this condition is here
                            // and you will probably not get it.
                            // But that lets me verify this VM on paper XD
                            // Why do you need an instruction that jumps to itself
                            // anyway?!
                            if *x == instruction_ptr { panic!("Self jumps are not allowed"); }
                            instruction_ptr = *x;
                        },
                        Instruction::Msg(x) => {
                            request = Some(Request::PrintMessage(x));
                            instruction_ptr += 1;
                        },
                        Instruction::Wait => {
                            request = Some(Request::Wait);
                            instruction_ptr += 1;
                        },
                        Instruction::Branch(data) => {
                            // Don't update the pointer. Keep it on the choice
                            // instruction, follow the specification1
                            request = Some(Request::PerformChoice(data));
                        },
                        Instruction::PushPtr(x) => {
                            frame_stack.push(*x);
                            instruction_ptr += 1;
                        },
                    }
                },
                // Fell out of the opcode array somehow.
                None => panic!("Instruction ptr out of range ({})", instruction_ptr),
            }
            // Decrease the limti
            limit -= 1;
        }
        
        // I don't forget to update the field on the struct so why not?!
        self.instruction_ptr = instruction_ptr;
        
        // Now let's look if captured any requests. We need to update
        // our inner state accordingly 
        match request {
            // No request? Then pause!
            None => { self.state = ProgramState::Paused; Request::Resume }
            // Otherwise...
            Some(x) => {
                match x {
                    Request::Drop => { self.state = ProgramState::Terminated; },
                    Request::Resume => { self.state = ProgramState::Paused; },
                    Request::PrintMessage(_) => { self.state = ProgramState::Paused; },
                    Request::Wait => { self.state = ProgramState::Waiting; },
                    Request::PerformChoice(_) => { self.state = ProgramState::WaitingForChoice; },
                };
                x
            },
        }
    }

    /// Send the "unpause" signal to the VM. This signal should be sent
    /// as an asnwer to the "Resume" request.
    pub fn unpause(&mut self, limit : Option<usize>) -> Request<'a> {
        match self.state.clone() {
            ProgramState::Paused => self.execute(limit),
            _ => panic!("Can't unpause in current state"),
        }
    }

    /// Send the "accepted" signal to the VM. This signal should be sent
    /// as an answer to the "FlushAndWait" request.
    pub fn done_printing(&mut self, limit : Option<usize>) -> Request<'a> {
        match self.state.clone() {
            ProgramState::Waiting => self.execute(limit),
            _ => panic!("I wasn't waiting for you to print"),
        }
    }

    /// Send the "choice(id)" signal to the VM. This signal should be sent
    /// as an answer to the "PerformChoice(x)" request.
    pub fn choose(&mut self, option_id : usize, limit : Option<usize>) -> Request<'a> {
        match self.state.clone() {
            ProgramState::WaitingForChoice => {
                // Right now the pointer is pointing at the choice instruction
                // This is an assumption which I am mentioning third time now :3
                let choice_opcode_ptr = self.instruction_ptr;
                // Where will we go now?
                let new_ptr = {
                    match self.my_program.opcodes.get(choice_opcode_ptr) {
                        Some(Instruction::Branch(data)) => {
                            match data.get(option_id) {
                                Some(x) => x.jmp_address,
                                // Now nobody said that the instructions will
                                // be correct :/
                                None => panic!("Choice out of range. The opcodes must be corrupted."),
                            }
                        },
                        // Now, tbh. I don't know how to complain if something
                        // goes wrong. In theory we can't even get there, since
                        // there's a guarantee, which was states in the `ProgramState`
                        // definition. My only guess is that this branch can only be
                        // triggered if the opcodes somehow get changed (which is impossible by
                        // Rust's borrowing rules). Hence there is a memory corruption
                        _ => unreachable!("Detected a memory corruption"),
                    }
                };
                // Jump there and execute
                self.instruction_ptr = new_ptr;
                self.execute(limit)
            },
            _ => panic!("I wasn't waiting for you to pick an option"),
        }
    }
}
