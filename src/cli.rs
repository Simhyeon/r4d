use clap::clap_app;
use crate::error::CliError;
use crate::basic::BasicMacro;

/// Struct to parse command line arguments and execute proper operations
pub struct Cli{}

impl Cli {
    pub fn parse() -> Result<(), CliError>{
        let cli_args = Cli::args_builder();
        Cli::parse_subcommands(&cli_args)?;
        Ok(())
    }
    fn parse_subcommands(args: &clap::ArgMatches) -> Result<(), CliError> {
        Cli::subcommand_direct(args)?;
        Ok(())
    }

    // TODO Add stream or file type option for main usage
    fn args_builder() -> clap::ArgMatches {
        clap_app!(Rif =>
            (version: "0.0.1")
            (author: "Simon Creek <simoncreek@tutanota.com>")
            (about: "R4d is a modern macro processro made with rust")
            (@setting ArgRequiredElseHelp)
            (@subcommand direct =>
                (about: "Directly call r4d macro")
                (@arg MACRO: +required "Macro to execute")
                (@arg args: -a --args +takes_value "Argument to be passed to macro")
                (@arg inc: ... -i --include +takes_value "(Not implemented)File to read macro definition from")
            )
        ).get_matches()
    }

    fn subcommand_direct(matches: &clap::ArgMatches) -> Result<(), CliError> {
        if let Some(sub_match) = matches.subcommand_matches("direct") {
            // TODO
            // Call direct macro call
            if let Some(mac) = sub_match.value_of("MACRO") {
                let mut args = "";
                if let Some(args_content) = sub_match.value_of("args") {
                    args = args_content;
                } 
                println!("Given macro name is {}", mac);

                let basic = BasicMacro::new();
                basic.call(mac, args).expect("Test failed");

                // TODO
                // Create hashamp with has macro name as key
                // and function pointer as value if possible
                // and try getting function according to given macro name
            }
        } 
        Ok(())
    }

}