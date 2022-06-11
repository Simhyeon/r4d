//! # Cli module
//! Cli module takes care of command line argument parsing and executing branches accordingly
//!
//! Cli module is only included in binary feature flag.

mod cli_rad;
mod cli_rado;
#[cfg(feature = "clap")]
pub use cli_rad::RadCli;
#[cfg(feature = "clap")]
pub use cli_rado::RadoCli;
