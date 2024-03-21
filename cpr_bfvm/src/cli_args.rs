use std::path::PathBuf;

use clap::{Parser, ValueEnum};

#[derive(Parser, Debug)]
#[command(author, about, version)]
pub(crate) struct CLIArgs {
    /// The file to run
    #[arg()]
    pub filename: PathBuf,

    /// The file from which running programs take their input. Defaults to stdin if empty
    #[arg(short, long)]
    pub input: Option<PathBuf>,

    /// The file to which running programs write their output. Defaults to stdout if empty
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// The size of each individual memory cell
    #[arg(value_enum, short, long, default_value_t = CellSize::U8)]
    pub cellsize: CellSize,

    /// The amount of preallocated memory cells. If a static allocator is used, this is also the total amount of available memory
    #[arg(short, long, default_value_t = 16)]
    pub preallocated: usize,

    /// The memory allocator to use
    #[arg(value_enum, short, long, default_value_t = Allocator::Dynamic)]
    pub allocator: Allocator,

    /// The verbosity of the logger
    #[cfg(not(debug_assertions))]
    #[arg(value_enum, short, long, default_value_t = LogLevel::Warn)]
    pub verbosity: LogLevel,

    /// The verbosity of the logger
    #[cfg(debug_assertions)]
    #[arg(value_enum, short, long, default_value_t = LogLevel::Info)]
    pub verbosity: LogLevel,
}

#[derive(Debug, Clone, ValueEnum)]
pub(crate) enum CellSize {
    U8,
    U16,
    U32,
    U64,
    U128,
}

#[derive(Debug, Clone, ValueEnum)]
pub(crate) enum Allocator {
    Dynamic,
    StaticChecked,
    StaticUnchecked,
}

#[derive(Debug, Clone, ValueEnum)]
pub(crate) enum LogLevel {
    Error,
    Warn,
    Info,
    #[cfg(debug_assertions)]
    Debug,
    #[cfg(debug_assertions)]
    Trace,
}

impl From<LogLevel> for log::Level {
    fn from(log_level: LogLevel) -> Self {
        match log_level {
            LogLevel::Error => log::Level::Error,
            LogLevel::Warn => log::Level::Warn,
            LogLevel::Info => log::Level::Info,
            #[cfg(debug_assertions)]
            LogLevel::Debug => log::Level::Debug,
            #[cfg(debug_assertions)]
            LogLevel::Trace => log::Level::Trace,
        }
    }
}

impl From<LogLevel> for log::LevelFilter {
    fn from(log_level: LogLevel) -> Self {
        match log_level {
            LogLevel::Error => log::LevelFilter::Error,
            LogLevel::Warn => log::LevelFilter::Warn,
            LogLevel::Info => log::LevelFilter::Info,
            #[cfg(debug_assertions)]
            LogLevel::Debug => log::LevelFilter::Debug,
            #[cfg(debug_assertions)]
            LogLevel::Trace => log::LevelFilter::Trace,
        }
    }
}
