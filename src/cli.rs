use clap::clap_app;
use crate::error::RadError;
use crate::processor::{Processor, WriteOption};
use std::path::Path;
use std::fs::OpenOptions;

/// Struct to parse command line arguments and execute proper operations
pub struct Cli{}

impl Cli {
    pub fn parse() -> Result<(), RadError>{
        let cli_args = Cli::args_builder();
        Cli::parse_options(&cli_args)?;
        // Cli::parse_subcommands(&cli_args)?;
        Ok(())
    }

    // Debug, TODO
    // Currently this process syntax parsing withint parse options methods
    // This is not ideal but ignored for debugging purpose
    // Plus parse options is always invoked which is not intended behaviour
    // if subcommand was given, main command should not be executed
    fn parse_options(args: &clap::ArgMatches) -> Result<(), RadError> {
        // Processor
        let mut processor: Processor;
        // Read from files
        if let Some(files) = args.values_of("FILE") {
            // Write to file
            if let Some(output_file) = args.value_of("out") {
                let out_file = OpenOptions::new()
                    .create(true)
                    .write(true)
                    .append(true)
                    .open(output_file)
                    .unwrap();
                out_file.set_len(0)?;
                processor = Processor::new(WriteOption::File(out_file));
            }
            // Write to standard output
            else { processor = Processor::new(WriteOption::Stdout); }

            for file in files {
                processor.from_file(Path::new(file), false)?;
            }
        } 
        // Read from stdin
        else {
            if let Some(output_file) = args.value_of("out") {
                let out_file = OpenOptions::new()
                    .create(true)
                    .write(true)
                    .append(true)
                    .open(output_file)
                    .unwrap();
                out_file.set_len(0)?;
                processor = Processor::new(WriteOption::File(out_file));
            } else { processor = Processor::new(WriteOption::Stdout); }

            processor.from_stdin(false)?;
        }
        Ok(())
    }

    fn _parse_subcommands(_args: &clap::ArgMatches) -> Result<(), RadError> {
        //Cli::subcommand_direct(args)?;
        Ok(())
    }

    // TODO Add stream or file type option for main usage
    fn args_builder() -> clap::ArgMatches {
        clap_app!(R4d =>
            (version: "0.0.1")
            (author: "Simon Creek <simoncreek@tutanota.com>")
            (about: "R4d is a modern macro processor made with rust")
            (@arg FILE: ... "Files to execute processing")
            (@arg out: -o --out +takes_value "File to print out macro")
        ).get_matches()
    }
}
