use r4d::RadResult;

pub fn main() -> RadResult<()> {
    // Enable color on pager such as "less"
    // by overloading color related environment
    #[cfg(feature = "color")]
    colored::control::set_override(true);

    // Command line parse
    #[cfg(feature = "clap")]
    {
        use r4d::RadoCli;
        use std::io::Write;
        if let Err(content) = RadoCli::new().parse() {
            writeln!(std::io::stderr(), "{}", content)?;
        }
    }
    Ok(())
}
