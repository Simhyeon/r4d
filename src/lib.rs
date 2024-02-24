//! # R4d(rad)
//! R4d is a text oriented macro processor that tries to solve inconveniences of well-known m4
//! macro processor.
//!
//! R4d is provided as both binary and library. Binary includes all features of optional
//! dependencies. Library doesn't provide any features by default so you can set them manually.
//!
//! # Features
//!
//! ```text
//! - evalexpr  : eval related macros
//! - chrono    : time related macros
//! - textwrap  : "wrap" macro
//! - cindex    : Query related macros
//! - full      : all features above
//!
//! - debug     : Enable debug method
//! - color     : Enable color prompt
//! - hook      : Enable hook macro
//! - signature : Enable signature option
//! ```
//!
//! # Simple usage
//!
//! **Binary**
//! ```text
//! # Read from file and print to stdout
//! rad input_file.txt
//! # Read from standard input and print to file
//! printf '...text...' | rad -o out_file.txt
//! ```
//!
//! **Library**
//! ```no_run
//! use r4d::{Processor, RadResult};
//! use std::path::Path;
//!
//! let mut processor = Processor::new()
//!     .purge(true)
//!     .write_to_file(Path::new("cache.txt"))
//!     .expect("Failed to open a file");
//!
//! processor.process_file(Path::new("input.txt"))
//!     .expect("Failed to process file");
//! processor.print_result().expect("Failed to print result");
//! ```
//!
//! Detailed r4d usage is illustrated in [github
//! page](https://github.com/simhyeon/r4d/blob/master/docs/usage.md) or in
//! [Processor](crate::Processor)

mod argument;
mod error;
mod package;
mod process;

mod parser;
pub(crate) use parser::{ArgParser, SplitVariant};

mod map;
pub(crate) use argument::{Argable, Parameter};
pub(crate) use map::deterred_map;
pub(crate) use map::function_map;
#[cfg(feature = "hook")]
pub(crate) use map::hookmap;
pub(crate) use map::runtime_map;
pub(crate) use map::sigmap;
pub(crate) use utils::RadStr;

pub(crate) mod auth;
pub(crate) mod common;
pub(crate) mod consts;
#[cfg(feature = "debug")]
pub(crate) mod debugger;
pub(crate) mod extension;
pub(crate) mod formatter;
pub(crate) mod lexor;
pub(crate) mod logger;
pub(crate) mod storage;
#[macro_use]
pub(crate) mod utils;

pub use auth::AuthType;
pub use common::{CommentType, DiffOption, Hygiene, MacroType, RadResult, WriteOption};
pub use error::RadError;
pub use extension::ExtMacroBuilder;
#[cfg(feature = "hook")]
pub use hookmap::HookType;
pub use logger::WarningType;
pub use process::Processor;
pub use storage::{RadStorage, StorageOutput, StorageResult};

// Optional

// User configurable script execution
#[cfg(feature = "template")]
mod script;

// Binary option
#[cfg(feature = "basic")]
mod cli;
#[cfg(feature = "basic")]
pub use cli::RadCli;
#[cfg(feature = "basic")]
pub use cli::RadoCli;

// Re-export macro
#[cfg(feature = "template")]
pub use rad_ext_template;

#[cfg(test)]
mod test;

mod env;
