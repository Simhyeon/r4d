//! # Cli module
//! Cli module takes care of command line argument parsing and executing branches accordingly
//!
//! Cli module is only included in binary feature flag.

mod cli_rad;
mod cli_rado;
pub use cli_rad::RadCli;
pub use cli_rado::RadoCli;
