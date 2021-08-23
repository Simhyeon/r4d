use clap::clap_app;
use crate::error::RadError;
use crate::processor::Processor;
use crate::models::WriteOption;
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

    fn parse_options(args: &clap::ArgMatches) -> Result<(), RadError> {

        let newline: String = 
            if cfg!(target_os = "windows") && !args.is_present("newline"){
                "\r\n".to_owned()
            } else {
                "\n".to_owned()
            };
        // Processor
        let mut processor: Processor;
        let mut error_option : Option<WriteOption> = Some(WriteOption::Stdout);
        // Set error write option
        if args.is_present("silent") {
            error_option = None; 
        } else if let Some(file) = args.value_of("err") {
            let err_file = OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(file)
                .unwrap();
            error_option = Some(WriteOption::File(err_file));
        }
        // Read from files
        if let Some(files) = args.values_of("FILE") {
            // Write to file
            if let Some(output_file) = args.value_of("out") {
                let out_file = OpenOptions::new()
                    .create(true)
                    .write(true)
                    .truncate(true)
                    .open(output_file)
                    .unwrap();
                processor = Processor::new(WriteOption::File(out_file), error_option, newline);
            }
            // Write to standard output
            else { processor = Processor::new(WriteOption::Stdout, error_option, newline); }

            for file in files {
                processor.set_file(file);
                processor.from_file(Path::new(file), false)?;
            }
        } 
        // Read from stdin
        else {
            if let Some(output_file) = args.value_of("out") {
                let out_file = OpenOptions::new()
                    .create(true)
                    .write(true)
                    .truncate(true)
                    .open(output_file)
                    .unwrap();
                processor = Processor::new(WriteOption::File(out_file), error_option, newline);
            } else { processor = Processor::new(WriteOption::Stdout, error_option, newline); }

            processor.from_stdin(false)?;
        }
        Ok(())
    }

    fn _parse_subcommands(_args: &clap::ArgMatches) -> Result<(), RadError> {
        Ok(())
    }

    fn args_builder() -> clap::ArgMatches {
        clap_app!(R4d =>
            (version: "0.2.1")
            (author: "Simon Creek <simoncreek@tutanota.com>")
            (about: "R4d is a modern macro processor made with rust")
            (@arg FILE: ... "Files to execute processing")
            (@arg out: -o --out +takes_value "File to print out macro")
            (@arg err: -e --err +takes_value "File to save error")
            (@arg silent: -s --silent "Supress error and warning")
            (@arg newline: -n --newline "Use unix newline for formatting")
        ).get_matches()
    }
}
