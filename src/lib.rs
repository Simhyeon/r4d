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
//! - evalexpr : "eval" macro
//! - chrono   : "date", "time" macro
//! - lipsum   : "lipsum" macro
//! - csv      : "from", "table" macro
//! 
//! - debug    : Enable debug method
//! - color    : Enable color prompt
//!
//! - full     : evalexpr+chrono+lipsum+csv
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
//! use rad::Processor;
//! use std::path::Path;
//! 
//! let mut processor = Processor::new()
//!     .purge(true)
//!     .greedy(true)
//!     .write_to_file(Some(Path::new("cache.txt")))?
//!     .build(); 
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
pub(crate) mod basic;
pub(crate) mod closure_map;
pub(crate) mod keyword_map;
pub(crate) mod consts;
pub(crate) mod lexor;
pub(crate) mod logger;
pub(crate) mod models;
pub(crate) mod utils;

pub use error::RadError;
pub use basic::MacroType;
pub use processor::Processor;
pub use auth::AuthType;

// Optional

// Binary option
#[cfg(feature = "clap")]
mod cli;
#[cfg(feature = "clap")]
pub use cli::Cli;

// Only for csv
#[cfg(feature = "csv")]
pub(crate) mod formatter;
