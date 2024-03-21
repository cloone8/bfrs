mod cli_args;

use std::fs::File;
use std::process::ExitCode;

use clap::Parser;
use cli_args::CLIArgs;
use cpr_bf::{allocators::*, VMBuilder};
use simplelog::{ColorChoice, ConfigBuilder, TermLogger, TerminalMode};

macro_rules! assign_allocator_and_build {
    ($args:expr, $builder:expr) => {
        match $args.allocator {
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

macro_rules! assign_cellsize_and_build {
    ($args:expr, $builder:expr) => {
        match $args.cellsize {
            cli_args::CellSize::U8 => {
                assign_allocator_and_build!($args, $builder.with_cell_type::<u8>())
            }
            cli_args::CellSize::U16 => {
                assign_allocator_and_build!($args, $builder.with_cell_type::<u16>())
            }
            cli_args::CellSize::U32 => {
                assign_allocator_and_build!($args, $builder.with_cell_type::<u32>())
            }
            cli_args::CellSize::U64 => {
                assign_allocator_and_build!($args, $builder.with_cell_type::<u64>())
            }
            cli_args::CellSize::U128 => {
                assign_allocator_and_build!($args, $builder.with_cell_type::<u128>())
            }
        }
    };
}

macro_rules! assign_input_and_build {
    ($args:expr, $builder:expr) => {
        match $args.input {
            Some(input) => {
                assign_cellsize_and_build!(
                    $args,
                    $builder.with_reader(File::open(input).expect("Could not open input file"))
                )
            }
            None => assign_cellsize_and_build!($args, $builder),
        }
    };
}

macro_rules! assign_output_and_build {
    ($args:expr, $builder:expr) => {
        match $args.output {
            Some(output) => {
                let output_file = File::options()
                    .create(true)
                    .truncate(true)
                    .write(true)
                    .open(output)
                    .expect("Could not open output file");
                assign_input_and_build!($args, $builder.with_writer(output_file))
            }
            None => assign_input_and_build!($args, $builder),
        }
    };
}

macro_rules! process_args_and_build_vm {
    ($args:expr) => {{
        let vm_builder = VMBuilder::new().with_preallocated_cells($args.preallocated);
        assign_output_and_build!($args, vm_builder)
    }};
}

fn main() -> ExitCode {
    let args = CLIArgs::parse();

    let logconfig = ConfigBuilder::new()
        .set_time_format_rfc3339()
        .set_time_offset_to_local()
        .expect("Could not set time offset to local")
        .build();

    TermLogger::init(
        args.verbosity.clone().into(),
        logconfig,
        TerminalMode::Stderr,
        ColorChoice::Auto,
    )
    .expect("Could not initialize logger");

    log::info!("Assigning VM options and building");

    let mut vm = process_args_and_build_vm!(args);

    log::info!("Running program");
    if let Err(e) = vm.run_from_path(&args.filename) {
        log::error!("Error during brainfuck execution: {}", e);
        return ExitCode::FAILURE;
    }

    log::info!("Program execution finished successfully");
    ExitCode::SUCCESS
}
