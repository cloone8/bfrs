use clap::{Parser, ValueEnum};

#[derive(Parser, Debug)]
#[command(author, about, version)]
pub(crate) struct CLIArgs {
    // CLI commands that will execute a thing and then exit
    #[arg(long, default_value_t = false)]
    pub list_backends: bool,

    // Runtime configuration options
    #[arg(short, long)]
    pub backend: Option<String>,

    #[cfg(not(debug_assertions))]
    #[arg(value_enum, short, long, default_value_t = LogLevel::Warn)]
    pub verbosity: LogLevel,

    #[cfg(debug_assertions)]
    #[arg(value_enum, short, long, default_value_t = LogLevel::Info)]
    pub verbosity: LogLevel,
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
