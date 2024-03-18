//! Contains various default memory allocators for the Brainfuck Virtual Machine trait

use crate::{BrainfuckAllocator, BrainfuckCell, OutOfBoundsAccess, VMMemoryError};

/// A dynamically allocating Brainfuck allocator.
/// If accessing an unallocated cell is attempted,
/// the VM memory is expanded to be abble to support that cell.
pub struct DynamicAllocator;

impl BrainfuckAllocator for DynamicAllocator {
    fn ensure_capacity<T: BrainfuckCell>(
        data: &mut Vec<T>,
        min_size: usize,
    ) -> Result<(), VMMemoryError> {
        // Ensure we allocate the required amount of memory
        if data.len() < min_size {
            data.resize(min_size, T::default());
        }

        Ok(())
    }
}

/// A non-allocating Brainfuck allocator
/// that checks whether the attempted access
/// lies within the bounds of the currently available memory.
/// If not, it returns an error
pub struct BoundsCheckingStaticAllocator;

impl BrainfuckAllocator for BoundsCheckingStaticAllocator {
    fn ensure_capacity<T: BrainfuckCell>(
        data: &mut Vec<T>,
        min_size: usize,
    ) -> Result<(), VMMemoryError> {
        if min_size > data.len() {
            Err(VMMemoryError::OutOfBounds(OutOfBoundsAccess {
                capacity: data.len(),
                access: min_size,
            }))
        } else {
            Ok(())
        }
    }
}

/// A non-allocating Brainfuck allocator that does not do any checking.
/// Any Brainfuck program that accesses cells beyond the preallocated
/// memory will lead to undefined behaviour.
///
/// This allocator is unsafe. Use [`BoundsCheckingStaticAllocator`] instead,
/// unless the input program is known to be safe.
pub struct StaticAllocator;

impl BrainfuckAllocator for StaticAllocator {
    fn ensure_capacity<T: BrainfuckCell>(_: &mut Vec<T>, _: usize) -> Result<(), VMMemoryError> {
        Ok(())
    }
}
