use clap::clap_app;
use crate::error::RadError;
use crate::basic::BasicMacro;
use crate::processor::Processor;

/// Struct to parse command line arguments and execute proper operations
pub struct Cli{}

impl Cli {
    pub fn parse() -> Result<(), RadError>{
        let cli_args = Cli::args_builder();
        Cli::parse_options(&cli_args)?;
        Cli::parse_subcommands(&cli_args)?;
        Ok(())
    }

    // Debug, TODO
    // Currently this process syntax parsing withint parse options methods
    // This is not ideal but ignored for debugging purpose
    // Plus parse options is always invoked which is not intended behaviour
    // if subcommand was given, main command should not be executed
    fn parse_options(args: &clap::ArgMatches) -> Result<(), RadError> {
        // File is given
        if let Some(files) = args.values_of("FILE") {
            unimplemented!();
        } else { // Read from stdin
            Processor::new().from_stdin()?;
        }
        Ok(())
    }

    fn parse_subcommands(args: &clap::ArgMatches) -> Result<(), RadError> {
        Cli::subcommand_direct(args)?;
        Ok(())
    }

    // TODO Add stream or file type option for main usage
    fn args_builder() -> clap::ArgMatches {
        clap_app!(R4d =>
            (version: "0.0.1")
            (author: "Simon Creek <simoncreek@tutanota.com>")
            (about: "R4d is a modern macro processro made with rust")
            (@arg FILE: ... "Files to execute processing")
            (@subcommand direct =>
                (about: "Directly call r4d macro")
                (@arg MACRO: +required "Macro to execute")
                (@arg args: -a --args +takes_value "Argument to be passed to macro")
                (@arg inc: ... -i --include +takes_value "(Not implemented)File to read macro definition from")
            )
        ).get_matches()
    }

    fn subcommand_direct(matches: &clap::ArgMatches) -> Result<(), RadError> {
        if let Some(sub_match) = matches.subcommand_matches("direct") {
            // TODO
            // Call direct macro call
            if let Some(mac) = sub_match.value_of("MACRO") {
                let mut args = "";
                if let Some(args_content) = sub_match.value_of("args") {
                    args = args_content;
                } 

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
