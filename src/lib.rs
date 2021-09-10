mod error;
mod processor;

pub(crate) mod basic;
pub(crate) mod consts;
pub(crate) mod lexor;
pub(crate) mod models;
pub(crate) mod utils;
pub(crate) mod arg_parser;

pub use error::RadError;
pub use processor::Processor;

// Optional

// Binary option
#[cfg(feature = "clap")]
mod cli;
#[cfg(feature = "clap")]
pub use cli::Cli;

// Only for csv
#[cfg(feature = "csv")]
pub(crate) mod formatter;
