use r4d::RadResult;
#[cfg(feature = "clap")]
use r4d::{RadCli, RadError};

pub fn main() -> RadResult<()> {
    // Enable color on pager such as "less"
    // by overloading color related environment
    #[cfg(feature = "color")]
    colored::control::set_override(true);

    // Command line parse
    #[cfg(feature = "clap")]
    {
        use std::io::Write;
        let mut cli = RadCli::new();
        if let Err(err) = cli.parse() {
            // This is a totally sane behaviour
            if let RadError::Exit = err {
                return Ok(());
            }
            cli.print_error(&err.to_string())?;
            writeln!(std::io::stderr(), "=== Processor panicked ===",)?;
        }
    }

    // End
    Ok(())
}
