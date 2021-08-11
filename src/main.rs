mod basic;
mod cli;
mod error;
mod fileio;
mod parser;
mod utils;

use cli::Cli;
use error::MainError;

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
