//! Parser module for multiple text formats

mod arg_parser;
mod define_parser;

pub use arg_parser::{ArgParser, SplitVariant};
pub use define_parser::DefineParser;
