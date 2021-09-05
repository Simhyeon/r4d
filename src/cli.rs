use clap::clap_app;
use crate::error::RadError;
use crate::processor::Processor;
use std::path::{Path, PathBuf};

/// Struct to parse command line arguments and execute proper operations
pub struct Cli{
    rules: Option<Vec<PathBuf>>,
    write_to_file : Option<PathBuf>,
    error_to_file : Option<PathBuf>,
}

impl Cli {

    pub fn new() -> Self {
        Self {
            rules: None,
            write_to_file : None,
            error_to_file : None,
        }
    }

    pub fn parse(&mut self) -> Result<(), RadError>{
        let cli_args = Cli::args_builder();
        self.run_processor(&cli_args)?;
        Ok(())
    }

    /// Parse arguments and run processor
    fn run_processor(&mut self, args: & clap::ArgMatches) -> Result<(), RadError> {

        self.parse_options(args);

        // Build processor
        let mut processor = Processor::new()
            .purge(args.is_present("purge"))
            .greedy(args.is_present("greedy"))
            .strict(args.is_present("greedy"))
            .silent(args.is_present("silent"))
            .unix_new_line(args.is_present("newline"))
            .custom_rules(std::mem::replace(&mut self.rules,None))?
            .write_to_file(std::mem::replace(&mut self.write_to_file,None))?
            .error_to_file(std::mem::replace(&mut self.error_to_file,None))?
            .build();

        // ========
        // Main options
        // -->> Read from files
        if let Some(files) = args.values_of("FILE") {
            // Also read from stdin if given combiation option
            if args.is_present("combination") {
                processor.from_stdin()?;
            }

            // Read from files and write with given options
            for file in files {
                processor.from_file(Path::new(file))?;
            }
        } else { // -->> Read from stdin
            processor.from_stdin()?;
        }

        // Print result
        processor.print_result()?;

        // Freeze to file if option was given
        if let Some(file) = args.value_of("freeze") {
            processor.freeze_to_file(Path::new(file))?;
        }

        Ok(())
    }

    fn parse_options(&mut self, args: & clap::ArgMatches) {
        // ========
        // Sub options
        // custom rules
        self.rules = if let Some(files) = args.values_of("melt")  {
            let files = files.into_iter().map(|value| PathBuf::from(value)).collect::<Vec<PathBuf>>();
            Some(files)
        } else { None };

        // Write to file 
        self.write_to_file = if let Some(output_file) = args.value_of("out") {
            Some(PathBuf::from(output_file))
        } else { None };

        // Error to file 
        self.error_to_file = if let Some(output_file) = args.value_of("error") {
            Some(PathBuf::from(output_file))
        } else { None };
    }

    /// Creates argument template
    fn args_builder() -> clap::ArgMatches {
        clap_app!(rad =>
            (version: "0.5.2")
            (author: "Simon Creek <simoncreek@tutanota.com>")
            (about: "R4d is a modern macro processor made with rust")
            (@arg FILE: ... "Files to execute processing")
            (@arg out: -o --out +takes_value "File to print out macro")
            (@arg err: -e --err +takes_value "File to save logs")
            (@arg greedy: -g "Make all macro invocation greedy")
            (@arg melt: ... -m +takes_value "Frozen file to reads")
            (@arg freeze: -f +takes_value "Freeze to file")
            (@arg purge: -p "Purge unused macros")
            (@arg strict: -S "Strict mode")
            (@arg combination: -c "Read from both stdin and file inputs")
            (@arg silent: -s "Supress error and warning")
            (@arg newline: -n "Use unix newline for formatting")
        ).get_matches()
    }
}
