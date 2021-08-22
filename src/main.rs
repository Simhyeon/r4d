mod basic;
mod cli;
mod consts;
mod error;
mod formatter;
mod lexor;
mod models;
mod processor;
mod utils;
mod arg_parser;

use cli::Cli;
use error::RadError;

pub fn main() -> Result<(), RadError> {

    // Command line parse
    Cli::parse()?;

    // End 
    Ok(())
}
