//! # Cli module
//! Cli module takes care of command line argument parsing and executing branches accordingly
//!
//! Cli module is only included in binary feature flag.

use clap::clap_app;
use crate::RadResult;
use crate::auth::AuthType;
use crate::processor::Processor;
use crate::utils::Utils;
use crate::models::{CommentType, DiffOption};
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
    pub fn parse(&mut self) -> RadResult<()> {
        let cli_args = Cli::args_builder();
        self.run_processor(&cli_args)?;
        Ok(())
    }

    /// Parse arguments and run processor
    fn run_processor(&mut self, args: & clap::ArgMatches) -> RadResult<()> {
        self.parse_options(args);
        // Build processor
        let mut processor = Processor::new()
            .set_comment_type(
                CommentType::from_str(
                    if args.occurrences_of("comment") == 0 {
                        "none" // None when no runtime flag
                    } else {
                        args.value_of("comment").unwrap() // default is start
                    }
                )?
            )
            .purge(args.is_present("purge"))
            .greedy(args.is_present("greedy"))
            .lenient(args.is_present("lenient"))
            .silent(args.is_present("silent"))
            .nopanic(args.is_present("nopanic"))
            .assert(args.is_present("assert"))
            .allow(std::mem::replace(&mut self.allow_auth,None))
            .allow_with_warning(std::mem::replace(&mut self.allow_auth_warn,None))
            .unix_new_line(args.is_present("newline"))
            .custom_rules(std::mem::replace(&mut self.rules,None))?
            .write_to_file(std::mem::replace(&mut self.write_to_file,None))?
            .discard(args.is_present("discard"))
            .error_to_file(std::mem::replace(&mut self.error_to_file,None))?
            .debug(args.is_present("debug"))
            .log(args.is_present("log"))
            .diff(
                DiffOption::from_str(
                    if args.occurrences_of("diff") == 0 {
                        "none" // None when no runtime flag
                    } else {
                        args.value_of("diff").unwrap() // default is all
                    }
                )?
            )?
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

        // Freeze to a rule file if such option was given
        if let Some(file) = args.value_of("freeze") {
            processor.freeze_to_file(Path::new(file))?;
        }

        #[cfg(feature = "signature")]
        if args.occurrences_of("signature") != 0 {
            let sig_map = processor.get_signature_map()?;
            // TODO
            let sig_json = serde_json::to_string(&sig_map.object).expect("Failed to create sig map");

            // This is file name
            let file_name = args.value_of("signature").unwrap();

            // This is default empty value should not be "" because it is ignored by clap
            if file_name != " " {
                std::fs::write(Path::new(file_name), sig_json.as_bytes())?;
            } else {
                println!("{}", &sig_json);
            }
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
            (version: "1.3.0")
            (author: "Simon Creek <simoncreek@tutanota.com>")
            (about: "R4d(rad) is a modern macro processor made with rust. Refer https://github.com/simhyeon/r4d for detailed usage.")
            (long_about: "R4d is a text oriented macro processor which aims to be an modern alternative to m4 macro processor. R4d procedurally follows texts and substitue macro calls with defined macro body. R4d comes with varoius useful built in macros so that user don't have to define from scratch. R4d also supports multiple debugging flags for easier error detection. Refer https://github.com/simhyeon/r4d for detailed usage.")
            (after_long_help: include_str!("../docs/macro_help.md"))
            (override_usage: "rad <FILE> -o <OUT_FILE> -e <ERR_FILE>
    echo <STDIN_TEXT | rad 
    echo <STDIN_TEXT> | rad --combination <FILE> --diff
    rad <FILE> --debug --log --interactive
    rad <FILE> -f <RULE_FILE> --discard -n --silent")
            (@arg FILE: ... "Files to execute processing")
            (@arg signature: --signature +takes_value value_name["FILE"] default_value[" "] "Print signature to file.")
            (@arg out: -o --out +takes_value conflicts_with[discard] value_name["FILE"] "Save processed output to the file")
            (@arg err: -e --err +takes_value value_name["FILE"] "Save error logs to the file")
            (@arg greedy: -g --greedy "Make all macro invocations greedy")
            (@arg melt: ... -m --melt +takes_value value_name["FILE"] "Read macros from frozen file")
            (@arg freeze: -f --freeze +takes_value value_name["FILE"] "Freeze macros into a single file")
            (@arg purge: -p --purge "Purge unused macros without panicking. Doesn't work in strict mode")
            (@arg lenient: -l --lenient "Lenient mode, disables strict mode")
            (@arg nopanic: --nopanic "Don't panic in any circumstances, the most lenient mode")
            (@arg assert: --assert "Enable assert mode")
            (@arg debug: -d --debug "Debug mode")
            (@arg log: --log "Print log for every macro invocation. Only works on debug mode")
            (@arg diff: --diff +takes_value default_value["all"] value_name["DIFF TYPE"] "Show diff result (none|change|all)")
            (@arg comment: --comment +takes_value default_value["start"] value_name["COMMENT TYPE"] "Use comment option (none|start|any)")
            (@arg interactive: -i --interactive "Use interactive debug mode. This enables line wrapping.")
            (@arg combination: -c "Read from both stdin and file inputs. Stdin is evaluated first")
            (@arg discard: -D --discard "Discard output")
            (@arg allow: -a +takes_value value_name["AUTH TYPE"] "Allow permission (fin|fout|cmd|env)")
            (@arg allow_warn: -w +takes_value value_name["AUTH TYPE"] "Allow permission with warnings (fin|fout|cmd|env)")
            (@arg allow_all: -A conflicts_with[allow_all_warn] "Allow all permission")
            (@arg allow_all_warn: -W "Allow all permission with warning")
            (@arg silent: -s --silent "Supress warnings")
            (@arg newline: -n "Use unix newline for formatting")
        ).get_matches()
    }
}
