//! A simple Brainfuck interpretation library
//!
//! The library exposes the [`BrainfuckVM`] trait, representing an object
//! that is able to run Brainfuck programs either from source code represented
//! as a string, or from a Brainfuck source file.
//!
//! In addition to this general trait, it also provides the [`VMBuilder`] struct,
//! that can be used to create a Brainfuck VM that is customizable through various
//! means.
//!
//! # Examples
//!
//! To simply create a basic spec-compliant Brainfuck runner, and run some Brainfuck code:
//! ```
//! let code = "++++++++[>++++[>++>+++>+++>+<<<<-]>+>+>->>+[<]<-]>>.>---.+++++++..+++.>>.<-.<.+++.------.--------.>>+.>++.";
//!
//! let vm = cpr_bf::VMBuilder::new().build();
//! vm.run_string(code);
//! ```

pub mod allocators;

use allocators::DynamicAllocator;
use num::{
    traits::{WrappingAdd, WrappingSub},
    Unsigned,
};
use std::{
    convert::{TryFrom, TryInto},
    fmt::Display,
    fs::File,
    io::{self, stdin, stdout, Read, Stdin, Stdout, Write},
    iter::repeat,
    marker::PhantomData,
    path::Path,
};

/// Represents a single Brainfuck instruction
#[derive(Clone, Copy, Debug)]
pub enum Instruction {
    /// Increment the current data pointer by one
    IncrDP,

    /// Decrement the current data pointer by one
    DecrDP,

    /// Increment the value in the cell that the data pointer currently points to by one
    Incr,

    /// Decrements the value in the cell that the data pointer currently points to by one
    Decr,

    /// Writes the value in the cell that the data pointer currently points to, to the VM writer
    Output,

    /// Reads one byte from the VM reader and stores it in the cell that the data pointer currently points to
    Input,

    /// If the value in the currently pointed-to cell is zero, jumps forwards to the next matching [`Instruction::JumpBack`] instruction.
    JumpFwd,

    /// If the value in the currently pointer-to cell is not zero, jumps backwards to the previous matching [`Instruction::JumpFwd`] instruction.
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

/// Struct representing a complete Brainfuck program.
/// The program does not need to be constructed directly,
/// and is instead constructed automatically through the various `run_*` methods
/// defined on the [`BrainfuckVM`] trait.
///
/// If desired, however, one can be constructed through the [`From<&str>`] trait
/// implementation defined for [`Program`]
pub struct Program {
    instructions: Vec<Instruction>,
}

impl From<&str> for Program {
    fn from(input: &str) -> Self {
        let instructions = input
            .chars()
            .filter_map(|c| Instruction::try_from(c).ok())
            .collect();

        Program { instructions }
    }
}

/// This trait defines types that can be used as the datatype for a single cell of
/// a Brainfuck VM. Can be implemented manually (although not recommended), but is
/// already implemented for the default unsigned int types ([`u8`], [`u16`], etc.)
pub trait BrainfuckCell:
    Unsigned + Copy + Default + TryInto<u32> + From<u8> + WrappingAdd + WrappingSub
{
}

impl<T: Unsigned + Copy + Default + TryInto<u32> + From<u8> + WrappingAdd + WrappingSub>
    BrainfuckCell for T
{
}

/// An out-of-bounds access error returned by the
/// Brainfuck VM if an access is attempted outside the
/// allocated memory region, without dynamic allocation being enabled
#[derive(Debug)]
pub struct OutOfBoundsAccess {
    /// The current maximum capacity of the VM, in number of cells
    pub capacity: usize,

    /// The index of the attempted access
    pub access: usize,
}

/// A general memory error encountered during runtime by the VM
#[derive(Debug)]
pub enum VMMemoryError {
    /// An out-of-bounds access
    OutOfBounds(OutOfBoundsAccess),
}

impl From<VMMemoryError> for BrainfuckExecutionError {
    fn from(value: VMMemoryError) -> Self {
        BrainfuckExecutionError::MemoryError(value)
    }
}

/// A trait representing an object that is capable of
/// allocating memory for a Brainfuck VM
pub trait BrainfuckAllocator {
    /// Ensures that `data` has at least `min_size` cells available for
    /// both reading and writing. If this function returns [`Result::Ok`],
    /// `data[min_size - 1]` can be safely read from and written to.
    ///
    /// Any new cells created by this function shall be initialized
    /// to the default value of `T`
    fn ensure_capacity<T: BrainfuckCell>(
        data: &mut Vec<T>,
        min_size: usize,
    ) -> Result<(), VMMemoryError>;
}

struct VirtualMachine<T: BrainfuckCell, A: BrainfuckAllocator, R: Read, W: Write> {
    data_ptr: usize,
    data: Vec<T>,
    alloc: PhantomData<A>,
    reader: R,
    writer: W,
}

/// A builder struct for the default implementation of [`BrainfuckVM`]
/// Create the default configuration with [`VMBuilder::new()`] or [`VMBuilder::default()`],
/// customize with the member functions, and build the final VM with [`VMBuilder::build()`]
pub struct VMBuilder<
    T: BrainfuckCell = u8,
    A: BrainfuckAllocator = DynamicAllocator,
    R: Read = Stdin,
    W: Write = Stdout,
> {
    initial_size: usize,
    celltype: PhantomData<T>,
    allocator: PhantomData<A>,
    reader: R,
    writer: W,
}

impl VMBuilder {
    /// Construct a new VMBuilder with the default initial configuration
    pub fn new() -> VMBuilder {
        VMBuilder::default()
    }
}

impl Default for VMBuilder {
    /// Construct a new VMBuilder with the default initial configuration
    fn default() -> Self {
        VMBuilder {
            initial_size: 0,
            celltype: PhantomData,
            allocator: PhantomData,
            reader: stdin(),
            writer: stdout(),
        }
    }
}

impl<T: BrainfuckCell + 'static, A: BrainfuckAllocator + 'static, R: Read, W: Write>
    VMBuilder<T, A, R, W>
{
    /// Changes the type of the memory cells to `U`
    pub fn with_cell_type<U: BrainfuckCell>(self) -> VMBuilder<U, A, R, W> {
        VMBuilder {
            initial_size: self.initial_size,
            celltype: PhantomData::<U>,
            allocator: self.allocator,
            reader: self.reader,
            writer: self.writer,
        }
    }

    /// Changes the used allocator to `U`
    pub fn with_allocator<U: BrainfuckAllocator>(self) -> VMBuilder<T, U, R, W> {
        VMBuilder {
            initial_size: self.initial_size,
            celltype: self.celltype,
            allocator: PhantomData::<U>,
            reader: self.reader,
            writer: self.writer,
        }
    }

    /// Changes the amount of pre-allocated cells to `num_preallocated`
    pub fn with_preallocated_cells(self, num_preallocated: usize) -> VMBuilder<T, A, R, W> {
        VMBuilder {
            initial_size: num_preallocated,
            ..self
        }
    }

    /// Changes the reader used by the VM as input for the running Brainfuck
    /// programs to `reader`
    pub fn with_reader<U: Read>(self, reader: U) -> VMBuilder<T, A, U, W> {
        VMBuilder {
            initial_size: self.initial_size,
            celltype: self.celltype,
            allocator: self.allocator,
            reader,
            writer: self.writer,
        }
    }

    /// Changes the writer used by the VM as output for the running Brainfuck programs
    /// to `writer`
    pub fn with_writer<U: Write>(self, writer: U) -> VMBuilder<T, A, R, U> {
        VMBuilder {
            initial_size: self.initial_size,
            celltype: self.celltype,
            allocator: self.allocator,
            reader: self.reader,
            writer,
        }
    }

    /// Builds the [`BrainfuckVM`] with the currently
    /// stored configuration of this builder
    pub fn build(self) -> Box<dyn BrainfuckVM> {
        Box::new(VirtualMachine::<T, A, Stdin, Stdout>::new(
            self.initial_size,
            stdin(),
            stdout(),
        ))
    }
}

/// The kind of missing bracket
#[derive(Debug)]
pub enum MissingKind {
    Open,
    Close,
}

/// A fatal error encountered by the Brainfuck VM during program execution.
#[derive(Debug)]
pub enum BrainfuckExecutionError {
    /// An unknown error
    UnknownError,

    /// An error during input or output
    IOError(io::Error),

    /// Mismatched brackets
    BracketMismatchError(MissingKind),

    /// An error during memory allocation or access
    MemoryError(VMMemoryError),

    /// Overflow in the data pointer
    DataPointerOverflow,

    /// Underflow in the data pointer
    DataPointerUnderflow,
}

impl Display for BrainfuckExecutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BrainfuckExecutionError::UnknownError => write!(f, "Unknown error"),
            BrainfuckExecutionError::IOError(e) => write!(f, "I/O Error: {}", e),
            BrainfuckExecutionError::BracketMismatchError(MissingKind::Close) => {
                write!(f, "Too few closing brackets")
            }
            BrainfuckExecutionError::BracketMismatchError(MissingKind::Open) => {
                write!(f, "Too few opening brackets")
            }
            BrainfuckExecutionError::MemoryError(VMMemoryError::OutOfBounds(a)) => write!(
                f,
                "Out of bounds memory access at index {} (max size {})",
                a.access, a.capacity
            ),
            BrainfuckExecutionError::DataPointerOverflow => write!(f, "Data pointer overflow!"),
            BrainfuckExecutionError::DataPointerUnderflow => write!(f, "Data pointer underflow!"),
        }
    }
}

impl std::error::Error for BrainfuckExecutionError {
    fn cause(&self) -> Option<&dyn std::error::Error> {
        match self {
            BrainfuckExecutionError::IOError(e) => Some(e),
            _ => None,
        }
    }
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

impl<T: BrainfuckCell, Alloc: BrainfuckAllocator, R: Read, W: Write>
    VirtualMachine<T, Alloc, R, W>
{
    fn new(init_size: usize, reader: R, writer: W) -> Self {
        VirtualMachine {
            data_ptr: 0,
            data: repeat(T::default()).take(init_size).collect(),
            alloc: PhantomData,
            reader,
            writer,
        }
    }

    fn exec(
        &mut self,
        instrs: &[Instruction],
        instr_ptr: usize,
    ) -> Result<usize, BrainfuckExecutionError> {
        let instr = instrs[instr_ptr];

        match instr {
            Instruction::IncrDP => {
                self.data_ptr = self
                    .data_ptr
                    .checked_add(1)
                    .ok_or(BrainfuckExecutionError::DataPointerOverflow)?;
                Ok(instr_ptr + 1)
            }
            Instruction::DecrDP => {
                self.data_ptr = self
                    .data_ptr
                    .checked_sub(1)
                    .ok_or(BrainfuckExecutionError::DataPointerUnderflow)?;
                Ok(instr_ptr + 1)
            }
            Instruction::Incr => {
                Alloc::ensure_capacity(&mut self.data, self.data_ptr + 1)?;
                self.data[self.data_ptr] = self.data[self.data_ptr].wrapping_add(&T::one());
                Ok(instr_ptr + 1)
            }
            Instruction::Decr => {
                Alloc::ensure_capacity(&mut self.data, self.data_ptr + 1)?;
                self.data[self.data_ptr] = self.data[self.data_ptr].wrapping_sub(&T::one());
                Ok(instr_ptr + 1)
            }
            Instruction::Output => {
                let val = self.data.get(self.data_ptr).cloned().unwrap_or_default();
                let as_char: char = val
                    .try_into()
                    .ok()
                    .and_then(char::from_u32)
                    .unwrap_or(char::REPLACEMENT_CHARACTER);
                write!(self.writer, "{}", as_char)?;
                Ok(instr_ptr + 1)
            }
            Instruction::Input => {
                let mut buf = [0_u8; 1];
                let num_read = self.reader.read(&mut buf)?;

                if num_read == 1 {
                    Alloc::ensure_capacity(&mut self.data, self.data_ptr + 1)?;
                    self.data[self.data_ptr] = buf[0].into();
                }

                Ok(instr_ptr + 1)
            }
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
                        }
                        _ => {}
                    }

                    closing_tag += 1;
                }

                Err(BrainfuckExecutionError::BracketMismatchError(
                    MissingKind::Close,
                ))
            }
            Instruction::JumpBack => {
                let val = self.data.get(self.data_ptr).cloned().unwrap_or_default();

                if val == T::zero() {
                    return Ok(instr_ptr + 1);
                }

                if instr_ptr == 0 {
                    return Err(BrainfuckExecutionError::BracketMismatchError(
                        MissingKind::Open,
                    ));
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
                        }
                        Instruction::JumpBack => tag_stack += 1,
                        _ => {}
                    }

                    opening_tag -= 1;
                }

                Err(BrainfuckExecutionError::BracketMismatchError(
                    MissingKind::Open,
                ))
            }
        }
    }
}

/// The result of the execution of a Brainfuck program
pub type BfResult = Result<(), BrainfuckExecutionError>;

/// This trait represents an object that is able to
/// run Brainfuck programs, either from a string
/// of Brainfuck source code or by reading a Brainfuck source file
///
/// A default implementation can be constructed using [`VMBuilder`]
pub trait BrainfuckVM {
    /// Runs the given Brainfuck program on this VM.
    /// After the program has been run, the memory of the VM
    /// is *not* automatically reset back to zero. (see [`BrainfuckVM::reset_memory`])
    ///
    /// Note that the VM might not be new, so the VM must take
    /// care of resetting the data pointer back to zero before
    /// running the program
    fn run_program(&mut self, program: &Program) -> BfResult;

    /// Resets all currently allocated memory cells back to their default
    /// value, as if no program has been run on the VM before.
    /// This does not free any cells that were allocated during the execution
    /// of any previous Brainfuck programs.
    fn reset_memory(&mut self);

    /// Compiles and runs the given string of Brainfuck source code.
    /// See [`BrainfuckVM::run_program`]
    fn run_string(&mut self, bf_str: &str) -> BfResult {
        let program: Program = bf_str.into();

        self.run_program(&program)
    }

    /// Reads the given file into a string, and
    /// runs the string on this VM.
    ///
    /// See [`BrainfuckVM::run_string`]
    fn run_file(&mut self, file: &mut File) -> BfResult {
        let mut program_str = String::new();
        file.read_to_string(&mut program_str)?;

        self.run_string(&program_str)
    }

    /// Opens the file pointed to by the given path,
    /// and attempts to run its contents on this VM.
    ///
    /// See [`BrainfuckVM::run_file`]
    fn run_from_path(&mut self, path: &Path) -> BfResult {
        let mut file = File::open(path)?;

        self.run_file(&mut file)
    }
}

impl<T: BrainfuckCell, A: BrainfuckAllocator, R: Read, W: Write> BrainfuckVM
    for VirtualMachine<T, A, R, W>
{
    fn reset_memory(&mut self) {
        self.data.iter_mut().for_each(|cell| *cell = T::default());
    }

    fn run_program(&mut self, program: &Program) -> Result<(), BrainfuckExecutionError> {
        if program.instructions.is_empty() {
            return Ok(());
        }

        self.data_ptr = 0;
        let mut instr_ptr = 0;

        while instr_ptr < program.instructions.len() {
            instr_ptr = self.exec(&program.instructions, instr_ptr)?;
        }

        self.writer.flush()?;

        Ok(())
    }
}
