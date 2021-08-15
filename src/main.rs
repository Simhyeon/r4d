mod basic;
mod cli;
mod error;
mod fileio;
mod processor;
mod parser;
mod utils;
mod models;

use cli::Cli;
use error::RadError;
use parser::Parser;

pub fn main() -> Result<(), RadError> {

    // Command line parse
    //Cli::parse()?;

    Parser::from_stdin();

    // End 
    Ok(())
}
