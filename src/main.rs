mod basic;
mod cli;
mod error;
mod fileio;
mod processor;
mod utils;
mod models;

use cli::Cli;
use error::RadError;
use processor::Processor;

pub fn main() -> Result<(), RadError> {

    // Command line parse
    //Cli::parse()?;

    Processor::new().from_stdin();

    // End 
    Ok(())
}
