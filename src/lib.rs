//! # R4d(rad)
//! R4d is a text oriented macro processor that tries to solve inconveniences of well-known m4
//! macro processor.
//!
//! R4d is provided as both binary and library. Binary includes all features of optional
//! dependencies. Library doesn't provide any features by default so you can set them manually.
//!
//! # Simple usage
//!
//! **Binary**
//! ```bash
//! # Usage : rad [OPTIONS] [FILE]...
//! 
//! # Read from file and save to file
//! rad input_file.txt -o out_file.txt
//! 
//! # Read from file and print to stdout 
//! rad input_file.txt
//! 
//! # Read from standard input and print to file
//! printf '...text...' | rad -o out_file.txt
//! 
//! # Read from stdin and print to stdout 
//! printf '...text...' | rad 
//! ```
//!
//! **Library**
//! ```rust
//! use rad::RadError;
//! use rad::Processor;
//! use rad::MacroType;
//! 
//! // Every option is not mendatory
//! let processor = Processor::new()
//!     .purge(true)
//!     .greedy(true)
//!     .silent(true)
//!     .strict(true)
//!     .custom_rules(Some(vec![pathbuf])) // Read from frozen rule files
//!     .write_to_file(Some(pathbuf))? // default is stdout
//!     .error_to_file(Some(pathbuf))? // default is stderr
//!     .unix_new_line(true) // use unix new line for formatting
//!     // Debugging options
//!     .debug(true) // Turn on debug mode
//!     .log(true) // Use logging to terminal
//!     .interactive(true) // Use interactive mode
//!     // Create unreferenced instance
//!     .build(); 
//! 
//! // Use Processor::empty() instead of Processor::new()
//! // if you don't want any default macros
//! 
//! // Add basic rules(= register functions)
//! processor.add_basic_rules(vec![("test", test as MacroType)]);
//! 
//! // Add custom rules(in order of "name, args, body") 
//! processor.add_custom_rules(vec![("test","a_src a_link","$a_src() -> $a_link()")]);
//! 
//! processor.from_string(r#"$define(test=Test)"#);
//! processor.from_stdin();
//! processor.from_file(&path);
//! processor.freeze_to_file(&path); // Create frozen file
//! processor.print_result(); // Print out warning and errors count
//! ```
//!
//! Detailed r4d usage is illustrated in [github page](https://github.com/simhyeon/r4d)

mod error;
mod processor;

pub(crate) mod basic;
pub(crate) mod consts;
pub(crate) mod lexor;
pub(crate) mod models;
pub(crate) mod utils;
pub(crate) mod arg_parser;
pub(crate) mod logger;

pub use error::RadError;
pub use processor::Processor;
pub use basic::MacroType;

// Optional

// Binary option
#[cfg(feature = "clap")]
mod cli;
#[cfg(feature = "clap")]
pub use cli::Cli;

// Only for csv
#[cfg(feature = "csv")]
pub(crate) mod formatter;
