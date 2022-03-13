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
//! - evalexpr  : "eval", "evalk" macro
//! - chrono    : "date", "time" macro
//! - lipsum    : "lipsum" macro
//! - csv       : "from", "table" macro
//! - textwrap  : "wrap" macro
//! - cindex    : Query related macros
//! - full      : all features above except cindex
//!
//! - debug     : Enable debug method
//! - color     : Enable color prompt
//! - hook      : Enable hook macro
//! - signature : Enable signature option
//! - storage   : Enable storage feature
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
//! ```rust
//! use rad::{Processor, RadResult};
//! use std::path::Path;
//!
//! let mut processor = Processor::new()
//!     .purge(true)
//!     .write_to_file(Some(Path::new("cache.txt")))?
//!
//! processor.from_file(Path::new("input.txt"))?;
//! processor.print_result()?;
//! ```
//!
//! Detailed r4d usage is illustrated in [github
//! page](https://github.com/simhyeon/r4d/blob/master/docs/usage.md) or in [processor
//! module](crate::processor)

mod error;
// This is necessary for docs.rs documentation
pub mod processor;

pub(crate) mod arg_parser;
pub(crate) mod auth;
pub(crate) mod consts;
#[cfg(feature = "debug")]
pub(crate) mod debugger;
pub(crate) mod define_parser;
pub(crate) mod deterred_map;
pub(crate) mod function_map;
#[cfg(feature = "hook")]
pub(crate) mod hookmap;
pub(crate) mod lexor;
pub(crate) mod logger;
pub(crate) mod models;
pub(crate) mod runtime_map;
#[cfg(feature = "signature")]
pub(crate) mod sigmap;
pub(crate) mod utils;

pub use auth::AuthType;
pub use error::RadError;
#[cfg(feature = "hook")]
pub use hookmap::HookType;
pub use logger::WarningType;
pub use models::{CommentType, DiffOption, RadResult, WriteOption};
#[cfg(feature = "storage")]
pub use models::{ExtMacroBuilder, MacroType, RadStorage, StorageOutput, StorageResult};
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

// Re-export macro
#[cfg(feature = "template")]
pub use rad_ext_template;

#[cfg(feature = "wasm")]
pub mod wasm;
