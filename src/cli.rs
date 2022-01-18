//! # Cli module
//! Cli module takes care of command line argument parsing and executing branches accordingly
//!
//! Cli module is only included in binary feature flag.

use std::io::Read;
use crate::RadResult;
use crate::auth::AuthType;
use crate::processor::Processor;
use crate::utils::Utils;
use crate::models::{CommentType, DiffOption};
#[cfg(feature = "signature")]
use crate::models::SignatureType;
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
            .interactive(args.is_present("interactive"));

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
        // Redirect stdin as argument
        if args.is_present("pipe") {
            let stdin = std::io::stdin();
            let mut input = String::new();
            stdin.lock().read_to_string(&mut input)?;
            processor.set_pipe(&input)
        }
        
        // -->> Read from files
        if let Some(sources) = args.values_of("INPUT") {
            // Also read from stdin if given combiation option
            if args.is_present("combination") {
                processor.from_stdin()?;
            }

            // Interpret every input source as literal text
            let literal = args.is_present("literal");

            // Read from given sources and write with given options
            for src in sources {
                let src_as_file = Path::new(src);

                if !literal && src_as_file.exists() {
                    processor.from_file(src_as_file)?;
                } else {
                    processor.from_string(src)?;
                }
            }
            #[cfg(feature = "signature")]
            self.print_signature(args, &mut processor)?;
        } else { // -->> Read from stdin

            // Print signature if such option is given
            // Signature option doesn't go with stdin option
            #[cfg(feature = "signature")]
            if self.print_signature(args, &mut processor)? {
                return Ok(());
            }
            processor.from_stdin()?;
        }

        // Print result
        processor.print_result()?;

        // Freeze to a rule file if such option was given
        if let Some(file) = args.value_of("freeze") {
            processor.freeze_to_file(Path::new(file))?;
        }
        Ok(())
    }

    /// Print signature
    ///
    /// Returns whether signature operation was executed or not
    #[cfg(feature = "signature")]
    fn print_signature(&mut self, args: &clap::ArgMatches, processor: &mut Processor) -> RadResult<bool> {
        #[cfg(feature = "signature")]
        if args.occurrences_of("signature") != 0 {
            let sig_type = SignatureType::from_str(args.value_of("sigtype").unwrap_or("all"))?;
            let sig_map = processor.get_signature_map(sig_type)?;
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
            Ok(true)
        } else { Ok(false) }
    }

    /// Parse processor options
    fn parse_options(&mut self, args: &clap::ArgMatches) {
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

    fn args_builder() -> clap::ArgMatches {
        use clap::{App,Arg};
        let app = App::new("rad")
            .version("1.6.0")
            .author("Simon creek <simoncreek@tutanota.com>")
            .about( "R4d(rad) is a modern macro processor made with rust. Refer https://github.com/simhyeon/r4d for detailed usage.")
            .long_about("R4d is a text oriented macro processor which aims to be an modern alternative to m4 macro processor. R4d procedurally follows texts and substitue macro calls with defined macro body. R4d comes with varoius useful built in macros so that user don't have to define from scratch. R4d also supports multiple debugging flags for easier error detection. Refer https://github.com/simhyeon/r4d for detailed usage.")
            .after_long_help(include_str!("../docs/macro_help.md"))
            .override_usage("rad <FILE> -o <OUT_FILE> -e <ERR_FILE>
    echo <STDIN_TEXT | rad 
    echo <STDIN_TEXT> | rad --combination <FILE> --diff
    rad <FILE> --debug --log --interactive
    rad <FILE> -f <RULE_FILE> --discard -n --silent")
            .arg(Arg::new("INPUT")
                .multiple_values(true)
                .help("INPUT source to execute processing"))
            .arg(Arg::new("pipe")
                .long("pipe")
                .conflicts_with("combination")
                .help("Send stdin as a pipe value"))
            .arg(Arg::new("literal")
                .long("literal")
                .help("Don't interpret input source as file"))
            .arg(Arg::new("out")
                .short('o')
                .long("out")
                .takes_value(true)
                .conflicts_with("discard")
                .value_name("FILE")
                .help("Save processed output to the file"))
            .arg(Arg::new("err")
                .short('e')
                .long("err")
                .takes_value(true)
                .value_name("FILE")
                .help("Save error logs to the file"))
            .arg(Arg::new("combination")
                .short('c')
                .long("combination")
                .help("Read from both stdin and file inputs. Stdin is evaluated first"))
            .arg(Arg::new("discard")
                .short('D')
                .long("discard")
                .help("Discard output"))
            .arg(Arg::new("silent")
                .short('s')
                .long("silent")
                .help("Supress warnings"))
            .arg(Arg::new("greedy")
                .short('g')
                .long("greedy")
                .help("Make all macro invocations greedy"))
            .arg(Arg::new("purge")
                .short('p')
                .long("purge")
                .help("Purge unused macros without panicking. Doesn't work in strict mode"))
            .arg(Arg::new("lenient")
                .short('l')
                .long("lenient")
                .help("Lenient mode, disables strict mode"))
            .arg(Arg::new("nopanic")
                .long("nopanic")
                .help("Don't panic in any circumstances, the most lenient mode"))
            .arg(Arg::new("debug")
                .short('d')
                .long("debug")
                .help("Debug mode"))
            .arg(Arg::new("log")
                .long("log")
                .help("Print log for every macro invocation. Only works on debug mode"))
            .arg(Arg::new("diff")
                .long("diff")
                .takes_value(true)
                .value_name("DIFF TYPE")
                .default_missing_value("all")
                .help("Show diff result (none|change|all)"))
            .arg(Arg::new("interactive")
                .short('i')
                .long("interactive")
                .help("Use interactive debug mode. This enables line wrapping."))
            .arg(Arg::new("assert")
                .long("assert")
                .help("Enable assert mode"))
            .arg(Arg::new("comment")
                .long("comment")
                .takes_value(true)
                .default_missing_value("start")
                .value_name("COMMENT TYPE")
                .help("Use comment option (none|start|any)"))
            .arg(Arg::new("allow")
                .short('a')
                .takes_value(true)
                .value_name("AUTH TYPE")
                .help("Allow permission (fin|fout|cmd|env)"))
            .arg(Arg::new("allow_warn")
                .short('w')
                .takes_value(true)
                .value_name("AUTH TYPE")
                .help("Allow permission with warnings (fin|fout|cmd|env)"))
            .arg(Arg::new("allow_all")
                .short('A')
                .conflicts_with("allow_all_warn")
                .help("Allow all permission"))
            .arg(Arg::new("allow_all_warn")
                .short('W')
                .conflicts_with("allow_all")
                .help("Allow all permission with warning"))
            .arg(Arg::new("newline")
                .short('n')
                .long("newline")
                .help("Use unix newline for formatting"))
            .arg(Arg::new("melt")
                .short('m')
                .long("melt")
                .takes_value(true)
                .value_name("FILE")
                .help("Read macros from frozen file"))
            .arg(Arg::new("freeze")
                .short('f')
                .long("freeze")
                .takes_value(true)
                .value_name("FILE")
                .help("Freeze macros into a single file"));

        #[cfg(feature = "signature")]
        let app = app.arg(Arg::new("signature")
                .long("signature")
                .takes_value(true)
                .value_name("FILE")
                .default_missing_value(" ")
                .help("Print signature to file."))
            .arg(Arg::new("sigtype")
                .long("sigtype")
                .takes_value(true)
                .value_name("SIG TYPE")
                .default_value("all")
                .help("Signature type to get"));

        app.get_matches()
    }
}
