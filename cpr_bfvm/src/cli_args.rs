use std::path::PathBuf;

use clap::{Parser, ValueEnum};

#[derive(Parser, Debug)]
#[command(author, about, version)]
pub(crate) struct CLIArgs {
    #[arg()]
    pub filename: PathBuf,

    #[arg(value_enum, short, long, default_value_t = CellSize::U8)]
    pub cellsize: CellSize,

    #[arg(short, long, default_value_t = 16)]
    pub preallocated: usize,

    #[arg(value_enum, short, long, default_value_t = Allocator::Dynamic)]
    pub allocator: Allocator,

    #[cfg(not(debug_assertions))]
    #[arg(value_enum, short, long, default_value_t = LogLevel::Warn)]
    pub verbosity: LogLevel,

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
