mod cli_args;

use std::process::ExitCode;

use clap::Parser;
use cli_args::CLIArgs;
use cpr_bf::allocators::*;
use cpr_bf::VMBuilder;

macro_rules! assign_allocator_and_build {
    ($args:expr, $builder:expr) => {
        match $args {
            cli_args::Allocator::Dynamic => $builder.with_allocator::<DynamicAllocator>().build(),
            cli_args::Allocator::StaticChecked => $builder
                .with_allocator::<BoundsCheckingStaticAllocator>()
                .build(),
            cli_args::Allocator::StaticUnchecked => {
                $builder.with_allocator::<StaticAllocator>().build()
            }
        }
    };
}

fn main() -> ExitCode {
    let args = CLIArgs::parse();

    simple_logger::init_with_level(args.verbosity.clone().into()).unwrap();

    log::info!("Assigning VM options and building");

    let vm_builder = VMBuilder::new().with_preallocated_cells(args.preallocated);

    let mut vm = match args.cellsize {
        cli_args::CellSize::U8 => {
            assign_allocator_and_build!(args.allocator, vm_builder.with_cell_type::<u8>())
        }
        cli_args::CellSize::U16 => {
            assign_allocator_and_build!(args.allocator, vm_builder.with_cell_type::<u16>())
        }
        cli_args::CellSize::U32 => {
            assign_allocator_and_build!(args.allocator, vm_builder.with_cell_type::<u32>())
        }
        cli_args::CellSize::U64 => {
            assign_allocator_and_build!(args.allocator, vm_builder.with_cell_type::<u64>())
        }
        cli_args::CellSize::U128 => {
            assign_allocator_and_build!(args.allocator, vm_builder.with_cell_type::<u128>())
        }
    };

    log::info!("Running program");
    if let Err(e) = vm.run_from_path(&args.filename) {
        log::error!("Error during brainfuck execution: {}", e);
        return ExitCode::FAILURE;
    }

    log::info!("Program execution finished successfully");
    ExitCode::SUCCESS
}
