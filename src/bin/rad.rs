use r4d::RadResult;

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
            cli.print_error(&err.to_string())?;
            writeln!(std::io::stderr(), "=== Processor panicked ===",)?;
        }
    }

    // End
    Ok(())
}
