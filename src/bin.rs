#[cfg(feature = "clap")]
use rad::Cli;
use rad::RadResult;

pub fn main() -> RadResult<()> {

    // Enable color on pager such as "less" 
    // by overloading color related environment
    #[cfg(feature = "color")]
    colored::control::set_override(true);

    // Command line parse
    #[cfg(feature = "clap")]
    if let Err(content) = Cli::new().parse() {
        eprintln!("{}", content);
    }

    // End 
    Ok(())
}
