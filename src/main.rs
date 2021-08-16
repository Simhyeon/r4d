mod basic;
mod cli;
mod error;
mod fileio;
mod processor;
mod utils;
mod models;

use cli::Cli;
use error::RadError;

pub fn main() -> Result<(), RadError> {

    // Command line parse
    Cli::parse()?;

    // End 
    Ok(())
}
