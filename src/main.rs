mod parser;
mod fileio;
mod cli;
mod error;
mod basic;

use parser::Parser;
use cli::Cli;
use error::MainError;

pub fn main() -> Result<(), MainError> {
    // Command line parse
    Cli::parse()?;
    
    // End 
    Ok(())
}
