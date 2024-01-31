//! Rad is a main binary executable for processing

use r4d::{RadError, RadResult};

/// Main entry for rad binary
pub fn main() -> RadResult<()> {
    // Enable color on pager such as "less"
    // by overloading color related environment
    #[cfg(feature = "color")]
    colored::control::set_override(true);

    // Command line parse
    #[cfg(feature = "basic")]
    {
        use r4d::RadCli;
        use std::io::Write;
        let mut cli = RadCli::new();
        if let Err(err) = cli.parse() {
            match err {
                RadError::SaneExit => {
                    std::process::exit(0);
                }
                _ => {
                    cli.print_error(&err.to_string())?;
                    writeln!(
                        std::io::stderr(),
                        "Int: Rad panicked with unrecoverable error."
                    )?;
                    std::process::exit(1);
                }
            }
        }
    }

    // End
    Ok(())
}
