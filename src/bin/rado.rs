//! Rado is a wrapper binary executable for a processing
//!
//! Rado provides multiple ergonomic functionalities

use r4d::RadResult;

/// Main entry
pub fn main() -> RadResult<()> {
    // Enable color on pager such as "less"
    // by overloading color related environment
    #[cfg(feature = "color")]
    colored::control::set_override(true);

    // Command line parse
    #[cfg(feature = "basic")]
    {
        use r4d::RadoCli;
        use std::io::Write;
        if let Err(content) = RadoCli::new().parse() {
            writeln!(std::io::stderr(), "{}", content)?;
            writeln!(
                std::io::stderr(),
                "Int: Rad panicked with unrecoverable error."
            )?;
            std::process::exit(1);
        }
    }
    Ok(())
}
