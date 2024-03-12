use std::io::{self, Read};

#[derive(Clone, Copy, Debug)]
pub enum Instruction {
    IncrDP,
    DecrDP,
    Incr,
    Decr,
    Output,
    Input,
    JumpFwd,
    JumpBack,
}

impl TryFrom<char> for Instruction {
    type Error = ();

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            '>' => Ok(Instruction::IncrDP),
            '<' => Ok(Instruction::DecrDP),
            '+' => Ok(Instruction::Incr),
            '-' => Ok(Instruction::Decr),
            '.' => Ok(Instruction::Output),
            ',' => Ok(Instruction::Input),
            '[' => Ok(Instruction::JumpFwd),
            ']' => Ok(Instruction::JumpBack),
            _ => Err(()),

        }
    }
}

pub struct Program {
    instructions: Vec<Instruction>
}

impl From<&str> for Program {
    fn from(input: &str) -> Self {
        let instructions = input.chars()
            .filter_map(|c| Instruction::try_from(c).ok())
            .collect();

        Program { instructions }
    }
}

struct BrainfuckVM {
    data_ptr: usize,
    data: Vec<u8>
}

#[derive(Debug)]
pub enum MissingKind {
    Open,
    Close
}

#[derive(Debug)]
pub enum BrainfuckExecutionError {
    UnknownError,
    IOError(io::Error),
    BracketMismatchError(MissingKind),
    DataPointerOverflow,
    DataPointerUnderflow,
}

impl From<()> for BrainfuckExecutionError {
    fn from(_: ()) -> Self {
        BrainfuckExecutionError::UnknownError
    }
}

impl From<io::Error> for BrainfuckExecutionError {
    fn from(value: io::Error) -> Self {
        BrainfuckExecutionError::IOError(value)
    }
}

impl BrainfuckVM {
    fn new(init_size: usize) -> Self {
        BrainfuckVM {
            data_ptr: 0,
            data: Vec::with_capacity(init_size)
        }
    }

    fn ensure_mem(&mut self, min_size: usize) -> Result<(), ()> {
        // Ensure we allocate the required amount of memory
        if self.data.len() < min_size {
            self.data.resize(min_size, 0);
        }

        Ok(())
    }

    fn exec(&mut self, instrs: &[Instruction], instr_ptr: usize) -> Result<usize, BrainfuckExecutionError> {
        let instr = instrs[instr_ptr];

        match instr {
            Instruction::IncrDP => {
                self.data_ptr = self.data_ptr.checked_add(1).ok_or(BrainfuckExecutionError::DataPointerOverflow)?;
                Ok(instr_ptr + 1)
            }
            Instruction::DecrDP => {
                self.data_ptr = self.data_ptr.checked_sub(1).ok_or(BrainfuckExecutionError::DataPointerUnderflow)?;
                Ok(instr_ptr + 1)
            }
            Instruction::Incr => {
                self.ensure_mem(self.data_ptr + 1)?;
                self.data[self.data_ptr] = self.data[self.data_ptr].wrapping_add(1);
                Ok(instr_ptr + 1)
            },
            Instruction::Decr => {
                self.ensure_mem(self.data_ptr + 1)?;
                self.data[self.data_ptr] = self.data[self.data_ptr].wrapping_sub(1);
                Ok(instr_ptr + 1)
            },
            Instruction::Output => {
                let val: char = self.data.get(self.data_ptr).cloned().unwrap_or(0).into();
                print!("{}", val);
                Ok(instr_ptr + 1)
            },
            Instruction::Input => {
                let mut buf = [0_u8; 1];
                let num_read = io::stdin().read(&mut buf)?;

                if num_read == 1 {
                    self.ensure_mem(self.data_ptr + 1)?;
                    self.data[self.data_ptr] = buf[0];
                }

                Ok(instr_ptr + 1)
            },
            Instruction::JumpFwd => {
                let val = self.data.get(self.data_ptr).cloned().unwrap_or(0);

                if val != 0 {
                    return Ok(instr_ptr + 1);
                }

                let mut closing_tag = instr_ptr + 1;
                let mut tag_stack: usize = 1;

                while closing_tag < instrs.len() {

                    match instrs[closing_tag] {
                        Instruction::JumpFwd => tag_stack += 1,
                        Instruction::JumpBack => {
                            tag_stack -= 1;
                            if tag_stack == 0 {
                                return Ok(closing_tag);
                            }
                        },
                        _ => {}
                    }

                    closing_tag += 1;
                }

                Err(BrainfuckExecutionError::BracketMismatchError(MissingKind::Close))
            },
            Instruction::JumpBack => {
                let val = self.data.get(self.data_ptr).cloned().unwrap_or(0);

                if val == 0 {
                    return Ok(instr_ptr + 1);
                }

                if instr_ptr == 0 {
                    return Err(BrainfuckExecutionError::BracketMismatchError(MissingKind::Open))
                }

                let mut opening_tag = instr_ptr - 1;
                let mut tag_stack: usize = 1;

                while opening_tag > 0 {
                    match instrs[opening_tag] {
                        Instruction::JumpFwd => {
                            tag_stack -= 1;
                            if tag_stack == 0 {
                                return Ok(opening_tag);
                            }
                        },
                        Instruction::JumpBack => tag_stack += 1,
                        _ => {}
                    }

                    opening_tag -= 1;
                }

                Err(BrainfuckExecutionError::BracketMismatchError(MissingKind::Open))
            },
        }
    }

    fn run_program(&mut self, program: &Program) -> Result<(), BrainfuckExecutionError> {
        if program.instructions.len() == 0 {
            return Ok(());
        }

        let mut instr_ptr = 0;

        while instr_ptr < program.instructions.len() {
            instr_ptr = self.exec(&program.instructions, instr_ptr)?;
        }

        Ok(())
    }
}

pub fn run_string(bf_str: &str) -> Result<(), BrainfuckExecutionError> {
    let program: Program = bf_str.into();
    let mut vm = BrainfuckVM::new(16);

    vm.run_program(&program)
}
