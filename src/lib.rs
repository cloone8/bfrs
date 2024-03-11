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
    instruction_ptr: usize,
    data_ptr: usize,
    data: Vec<u8>
}

impl BrainfuckVM {
    fn exec(&mut self, instr: &Instruction) -> Result<(), ()> {
        match instr {
            Instruction::IncrDP => {
                match self.data_ptr.checked_add(1) {
                    Some(res) => self.data_ptr = res,
                    None => return Err(()),
                }
                Ok(())
            }
            Instruction::DecrDP => {
                match self.data_ptr.checked_sub(1) {
                    Some(res) => self.data_ptr = res,
                    None => return Err(()),
                }
                Ok(())
            }
            Instruction::Incr => todo!(),
            Instruction::Decr => todo!(),
            Instruction::Output => todo!(),
            Instruction::Input => todo!(),
            Instruction::JumpFwd => todo!(),
            Instruction::JumpBack => todo!(),
        }
    }

    fn run_program(&mut self, program: &Program) -> Result<(), ()> {
        for instruction in &program.instructions {
            self.exec(instruction)?;
        }

        Ok(())
    }
}

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
