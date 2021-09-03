use rad::cli::Cli;
use rad::error::RadError;

pub fn main() -> Result<(), RadError> {

    // Command line parse
    if let Err(content) = Cli::new().parse() {
        eprintln!("{}", content);
    }

    // End 
    Ok(())
}
