use std::{convert::{TryFrom, TryInto}, fs::File, io::{self, Read}, iter::repeat, marker::PhantomData, path::Path};
use num::{traits::{WrappingAdd, WrappingSub}, Unsigned};

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

pub trait BrainfuckCell: Unsigned + Copy + Default + TryInto<u32> + From<u8> + WrappingAdd + WrappingSub {}
impl<T: Unsigned + Copy + Default + TryInto<u32> + From<u8> + WrappingAdd + WrappingSub> BrainfuckCell for T {}

pub trait BrainfuckAllocator {
    fn ensure_capacity<T: BrainfuckCell>(data: &mut Vec<T>, min_size: usize) -> Result<(), ()>;
}

pub struct DynamicAllocator;

impl BrainfuckAllocator for DynamicAllocator {
    fn ensure_capacity<T: BrainfuckCell>(data: &mut Vec<T>, min_size: usize) -> Result<(), ()> {
        // Ensure we allocate the required amount of memory
        if data.len() < min_size {
            data.resize(min_size, T::default());
        }

        Ok(())
    }
}

pub struct BoundsCheckingStaticAllocator;

impl BrainfuckAllocator for BoundsCheckingStaticAllocator {
    fn ensure_capacity<T: BrainfuckCell>(data: &mut Vec<T>, min_size: usize) -> Result<(), ()> {
        if min_size > data.len() {
            Err(())
        } else {
            Ok(())
        }
    }
}

pub struct StaticAllocator;

impl BrainfuckAllocator for StaticAllocator {
    fn ensure_capacity<T: BrainfuckCell>(_: &mut Vec<T>, _: usize) -> Result<(), ()> {
        Ok(())
    }
}

pub struct VirtualMachine<T: BrainfuckCell, A: BrainfuckAllocator> {
    data_ptr: usize,
    data: Vec<T>,
    alloc: PhantomData<A>
}

pub struct VMBuilder<T: BrainfuckCell = u8, A: BrainfuckAllocator = DynamicAllocator> {
    initial_size: usize,
    celltype: PhantomData<T>,
    allocator: PhantomData<A>
}

impl VMBuilder {
    pub fn new() -> VMBuilder {
        VMBuilder {
            initial_size: 0,
            celltype: PhantomData::default(),
            allocator: PhantomData::default()
        }
    }
}

impl<T: BrainfuckCell, A: BrainfuckAllocator> VMBuilder<T, A> {
    pub fn with_cell_type<U: BrainfuckCell>(self) -> VMBuilder<U, A> {
        VMBuilder {
            initial_size: self.initial_size,
            celltype: PhantomData::<U>::default(),
            allocator: self.allocator
        }
    }

    pub fn with_allocator<U: BrainfuckAllocator>(self) -> VMBuilder<T, U> {
        VMBuilder {
            initial_size: self.initial_size,
            celltype: self.celltype,
            allocator: PhantomData::<U>::default()
        }
    }

    pub fn with_preallocated_cells(self, num_preallocated: usize) -> VMBuilder<T, A> {
        VMBuilder {
            initial_size: num_preallocated,
            ..self
        }
    }

    pub fn build(self) -> impl BrainfuckVM {
        VirtualMachine::<T, A>::new(self.initial_size)
    }
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

impl<T: BrainfuckCell, Alloc: BrainfuckAllocator> VirtualMachine<T, Alloc> {
    fn new(init_size: usize) -> Self {
        VirtualMachine {
            data_ptr: 0,
            data: repeat(T::default()).take(init_size).collect(),
            alloc: PhantomData::default()
        }
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
                Alloc::ensure_capacity(&mut self.data, self.data_ptr + 1)?;
                self.data[self.data_ptr] = self.data[self.data_ptr].wrapping_add(&T::one());
                Ok(instr_ptr + 1)
            },
            Instruction::Decr => {
                Alloc::ensure_capacity(&mut self.data, self.data_ptr + 1)?;
                self.data[self.data_ptr] = self.data[self.data_ptr].wrapping_sub(&T::one());
                Ok(instr_ptr + 1)
            },
            Instruction::Output => {
                let val = self.data.get(self.data_ptr).cloned().unwrap_or_default();
                let as_char: char = val.try_into().ok().map(char::from_u32).flatten().unwrap_or(char::REPLACEMENT_CHARACTER);

                print!("{}", as_char);
                Ok(instr_ptr + 1)
            },
            Instruction::Input => {
                let mut buf = [0_u8; 1];
                let num_read = io::stdin().read(&mut buf)?;

                if num_read == 1 {
                    Alloc::ensure_capacity(&mut self.data, self.data_ptr + 1)?;
                    self.data[self.data_ptr] = buf[0].into();
                }

                Ok(instr_ptr + 1)
            },
            Instruction::JumpFwd => {
                let val = self.data.get(self.data_ptr).cloned().unwrap_or_default();

                if val != T::zero() {
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
                let val = self.data.get(self.data_ptr).cloned().unwrap_or_default();

                if val == T::zero() {
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
}

pub type BfResult = Result<(), BrainfuckExecutionError>;

pub trait BrainfuckVM {
    fn run_program(&mut self, program: &Program) -> Result<(), BrainfuckExecutionError>;

    fn run_string(&mut self, bf_str: &str) -> BfResult {
        let program: Program = bf_str.into();

        self.run_program(&program)
    }

    fn run_file(&mut self, file: &mut File) -> BfResult {
        let mut program_str = String::new();
        file.read_to_string(&mut program_str)?;

        self.run_string(&program_str)
    }

    fn run_from_path(&mut self, path: &Path) -> BfResult {
        let mut file = File::open(path)?;

        self.run_file(&mut file)
    }
}

impl<T: BrainfuckCell, A: BrainfuckAllocator> BrainfuckVM for VirtualMachine<T, A> {
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
