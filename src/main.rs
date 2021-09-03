#[cfg(feature = "clap")]
use rad::cli::Cli;
use rad::error::RadError;

pub fn main() -> Result<(), RadError> {

    // Command line parse
    #[cfg(feature = "clap")]
    if let Err(content) = Cli::new().parse() {
        eprintln!("{}", content);
    }

    // End 
    Ok(())
}
