mod parser;
mod fileio;
mod cli;
mod error;
mod basic;

use basic::BasicMacro;
use cli::Cli;
use error::MainError;
use parser::Parser;

pub fn main() -> Result<(), MainError> {
    // Command line parse
    Cli::parse()?;

    // End 
    Ok(())
}

// TESTS
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        let basic = BasicMacro::new();
        let result = basic.call("test", "args,content");
    }
}
