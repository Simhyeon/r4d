use clap::clap_app;
use crate::error::RadError;
use crate::processor::Processor;
use crate::models::{WriteOption, RuleFile};
use std::path::Path;
use std::fs::OpenOptions;

/// Struct to parse command line arguments and execute proper operations
pub struct Cli{}

impl Cli {
    pub fn parse() -> Result<(), RadError>{
        let cli_args = Cli::args_builder();
        Cli::parse_options(&cli_args)?;
        Ok(())
    }

    fn new_parse_options(args: &clap::ArgMatches) -> Result<(), RadError> {

        // ========
        // Sub options
        // custom rules
        let rules = if let Some(files) = args.values_of("melt")  {
            let files = files.into_iter().map(|value| Path::new(value)).collect::<Vec<&Path>>();
            Some(files)
        } else { None };

        // Write to file 
        let write_to_file = if let Some(output_file) = args.value_of("out") {
            Some(Path::new(output_file))
        } else { None };

        // Error to file 
        let error_to_file = if let Some(output_file) = args.value_of("error") {
            Some(Path::new(output_file))
        } else { None };

        // Build processor
        let mut processor = Processor::new_proc()
            .purge(args.is_present("purge"))
            .greedy(args.is_present("greedy"))
            .strict(args.is_present("greedy"))
            .silent(args.is_present("silent"))
            .unix_new_line(args.is_present("newline"))
            .custom_rules(rules)?
            .write_to_file(write_to_file)?
            .error_to_file(error_to_file)?;

        // ========
        // Main options
        // -->> Read from files
        if let Some(files) = args.values_of("FILE") {
            // Also read from stdin if given combiation option
            if args.is_present("combination") {
                processor.from_stdin(false)?;
            }

            // Read from files and write with given options
            for file in files {
                processor.from_file(Path::new(file), false)?;
            }
        } 
        // -->> Read from stdin
        else {
            processor.from_stdin(false)?;
        }

        // Print result
        processor.print_result()?;

        if let Some(file) = args.value_of("freeze") {
            RuleFile::new(Some(processor.map.custom)).freeze(&Path::new(file))?;
        }

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
        let purge_option = args.is_present("purge");
        let strict_option = args.is_present("strict");
        let greedy_option = args.is_present("greedy");
        // ========
        // Sub options
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

        // Read from frozen files and melt into processor later
        let mut rule_file = RuleFile::new(None);
        if let Some(files) = args.values_of("melt") {
            for file in files {
                let path = &Path::new(file);
                if !path.exists() {
                    return Err(RadError::InvalidCommandOption(format!("File, \"{}\" doesn't exist, therefore cannot be melted.", path.display())))
                }
                rule_file.melt(path)?;
            }
        }
        // ========
        // Main options
        // Read from files
        if let Some(files) = args.values_of("FILE") {
            // Set option : write to file
            if let Some(output_file) = args.value_of("out") {
                let out_file = OpenOptions::new()
                    .create(true)
                    .write(true)
                    .truncate(true)
                    .open(output_file)
                    .unwrap();
                processor = Processor::new(WriteOption::File(out_file), error_option, newline);
                processor.add_custom_rules(rule_file.rules);
                if purge_option { processor.set_purge() }
                else if strict_option { processor.set_strict() }
                if greedy_option { processor.set_greedy() }
            }
            // Set option : write to standard output
            else { 
                processor = Processor::new(WriteOption::Stdout, error_option, newline); 
                processor.add_custom_rules(rule_file.rules);
                if purge_option { processor.set_purge() }
                else if strict_option { processor.set_strict() }
                if greedy_option { processor.set_greedy() }
            }

            // Also read from stdin if given combiation option
            if args.is_present("combination") {
                processor.from_stdin(false)?;
            }

            // Read from files and write with given options
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
                    .truncate(true)
                    .open(output_file)
                    .unwrap();
                processor = Processor::new(WriteOption::File(out_file), error_option, newline);
                processor.add_custom_rules(rule_file.rules);
                if purge_option { processor.set_purge() }
                else if strict_option { processor.set_strict() }
                if greedy_option { processor.set_greedy() }
            } else { 
                processor = Processor::new(WriteOption::Stdout, error_option, newline); 
                processor.add_custom_rules(rule_file.rules);
                if purge_option { processor.set_purge() }
                else if strict_option { processor.set_strict() }
                if greedy_option { processor.set_greedy() }
            }

            processor.from_stdin(false)?;
        }

        // Print result
        processor.print_result()?;

        if let Some(file) = args.value_of("freeze") {
            RuleFile::new(Some(processor.map.custom)).freeze(&Path::new(file))?;
        }

        Ok(())
    }

    fn _parse_subcommands(_args: &clap::ArgMatches) -> Result<(), RadError> {
        Ok(())
    }

    fn args_builder() -> clap::ArgMatches {
        clap_app!(R4d =>
            (version: "0.4.7")
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
