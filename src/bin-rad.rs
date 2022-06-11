#[cfg(feature = "clap")]
use rad::RadCli;
use rad::RadResult;

pub fn main() -> RadResult<()> {
    // Enable color on pager such as "less"
    // by overloading color related environment
    #[cfg(feature = "color")]
    colored::control::set_override(true);

    // Command line parse
    #[cfg(feature = "clap")]
    {
        use std::io::Write;
        if let Err(content) = RadCli::new().parse() {
            writeln!(
                std::io::stderr(),
                "Rad execution panicked with error => {}",
                content
            )?;
        }
    }

    // End
    Ok(())
}
