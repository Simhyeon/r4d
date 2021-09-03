pub mod error;
pub mod processor;

pub(crate) mod basic;
pub(crate) mod consts;
pub(crate) mod lexor;
pub(crate) mod models;
pub(crate) mod utils;
pub(crate) mod arg_parser;

// Optional

// Binary option
#[cfg(feature = "clap")]
pub mod cli;

// Only for csv
#[cfg(feature = "csv")]
pub(crate) mod formatter;
