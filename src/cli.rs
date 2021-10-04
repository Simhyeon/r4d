//! # Cli module
//! Cli module takes care of command line argument parsing and executing branches accordingly
//!
//! Cli module is only included in binary feature flag.

use clap::clap_app;
use crate::error::RadError;
use crate::auth::AuthType;
use crate::processor::Processor;
use crate::utils::Utils;
use std::path::{Path, PathBuf};

/// Struct to parse command line arguments and execute proper operations
pub struct Cli{
    rules: Option<Vec<PathBuf>>,
    write_to_file : Option<PathBuf>,
    error_to_file : Option<PathBuf>,
    allow_auth: Option<Vec<AuthType>>,
    allow_auth_warn: Option<Vec<AuthType>>,
}

impl Cli {

    pub fn new() -> Self {
        Self {
            rules: None,
            write_to_file : None,
            error_to_file : None,
            allow_auth : None,
            allow_auth_warn : None,
        }
    }

    /// User method to call cli workflow
    ///
    /// This sequentially parse command line arguments and execute necessary operations
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
            .nopanic(args.is_present("nopanic"))
            .allow(std::mem::replace(&mut self.allow_auth,None))
            .allow_with_warning(std::mem::replace(&mut self.allow_auth_warn,None))
            .unix_new_line(args.is_present("newline"))
            .custom_rules(std::mem::replace(&mut self.rules,None))?
            .write_to_file(std::mem::replace(&mut self.write_to_file,None))?
            .discard(args.is_present("discard"))
            .error_to_file(std::mem::replace(&mut self.error_to_file,None))?
            .debug(args.is_present("debug"))
            .log(args.is_present("log"))
            .diff(args.is_present("diff"))?
            .interactive(args.is_present("interactive"))
            .build();

        // Debug
        // Clear terminal cells
        #[cfg(feature = "debug")]
        if args.is_present("debug") {
            Utils::clear_terminal()?;
        }

        // ========
        // Main options
        // print permission
        processor.print_permission()?;
        
        // Process
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

    /// Parse processor options
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
        self.error_to_file = if let Some(error_file) = args.value_of("err") {
            Some(PathBuf::from(error_file))
        } else { None };

        // Permission
        self.allow_auth = if let Some(auths) = args.value_of("allow") {
            auths.split("+").map(|s| AuthType::from(s)).collect()
        } else { None };

        // Permission with warning
        self.allow_auth_warn = if let Some(auths) = args.value_of("allow_warn") {
            auths.split("+").map(|s| AuthType::from(s)).collect()
        } else { None };

        // Permission all
        if args.is_present("allow_all") {
            self.allow_auth = Some(
                vec![
                    AuthType::FIN,
                    AuthType::FOUT,
                    AuthType::ENV,
                    AuthType::CMD
                ]
            );
        }

        // Permission all with warning
        if args.is_present("allow_all_warn") {
            self.allow_auth_warn = Some(
                vec![
                    AuthType::FIN,
                    AuthType::FOUT,
                    AuthType::ENV,
                    AuthType::CMD
                ]
            );
        }
    }

    /// Creates argument template wich clap macro
    fn args_builder() -> clap::ArgMatches {
        clap_app!(rad =>
            (version: "0.9.4")
            (author: "Simon Creek <simoncreek@tutanota.com>")
            (about: "R4d is a modern macro processor made with rust")
            (@arg FILE: ... "Files to execute processing")
            (@arg out: -o --out +takes_value conflicts_with[discard] "File to print out macro")
            (@arg err: -e --err +takes_value "File to save logs")
            (@arg greedy: -g --greedy "Make all macro invocation greedy")
            (@arg melt: ... -m --melt +takes_value "Frozen file to reads")
            (@arg freeze: -f --freeze +takes_value "Freeze to file")
            (@arg purge: -p --purge "Purge unused macros")
            (@arg nopanic: --nopanic "Don't panic in any circumstances")
            (@arg strict: -S --strict conflicts_with[nopanic purge] "Strict mode")
            (@arg debug: -d --debug "Debug mode")
            (@arg log: -l --log "Debug log mode")
            (@arg diff: --diff "Show diff result")
            (@arg interactive: -i --interactive "Use interactive debug mode")
            (@arg combination: -c "Read from both stdin and file inputs")
            (@arg discard: -D --discard "Discard output without print out")
            (@arg allow: -a +takes_value "Allow permission (fin|fout|cmd|env)")
            (@arg allow_warn: -w +takes_value "Allow permission with warnings (fin|fout|cmd|env)")
            (@arg allow_all: -A conflicts_with[allow_all_warn] "Allow all permission")
            (@arg allow_all_warn: -W "Allow all permission with warning")
            (@arg silent: -s "Supress warnings")
            (@arg newline: -n "Use unix newline for formatting")
        ).get_matches()
    }
}
