//! Cli processor for rad binary

use crate::auth::AuthType;
use crate::common::CommentType;
#[cfg(feature = "debug")]
use crate::common::DiffOption;
#[cfg(feature = "signature")]
use crate::common::SignatureType;
#[cfg(feature = "signature")]
use crate::consts::LINE_ENDING;
use crate::logger::WarningType;
#[cfg(feature = "template")]
use crate::script;
#[cfg(feature = "debug")]
use crate::utils::Utils;
use crate::Processor;
use crate::{Hygiene, RadError, RadResult};
#[cfg(feature = "signature")]
use std::fmt::Write as _;
use std::io::Read;
#[cfg(feature = "signature")]
use std::io::Write;
use std::path::{Path, PathBuf};
use std::str::FromStr;

/// Struct to parse command line arguments and execute proper operations
pub struct RadCli<'cli> {
    rules: Vec<PathBuf>,
    write_to_file: Option<PathBuf>,
    error_to_file: Option<PathBuf>,
    allow_auth: Vec<AuthType>,
    allow_auth_warn: Vec<AuthType>,
    processor: Processor<'cli>,
}

impl<'cli> Default for RadCli<'cli> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'cli> RadCli<'cli> {
    /// Print an error with processor
    pub fn print_error(&mut self, error: &str) -> RadResult<()> {
        self.processor.print_error_no_line(error)?;
        Ok(())
    }

    /// Print an error with processor but no line number
    pub fn print_error_with_line(&mut self, error: &str) -> RadResult<()> {
        self.processor.print_error(error)?;
        Ok(())
    }

    /// Create a new instance
    pub fn new() -> Self {
        Self {
            rules: vec![],
            write_to_file: None,
            error_to_file: None,
            allow_auth: vec![],
            allow_auth_warn: vec![],
            processor: Processor::new(),
        }
    }

    /// User method to call cli workflow
    ///
    /// This sequentially parse command line arguments and execute necessary operations
    pub fn parse(&mut self) -> RadResult<()> {
        let cli_args = Self::args_builder(None);
        self.run_processor(&cli_args)?;
        Ok(())
    }

    /// Parse arguments from string
    pub(crate) fn parse_from(&mut self, source: &[&str]) -> RadResult<()> {
        let cli_args = Self::args_builder(Some(source));
        self.run_processor(&cli_args)?;
        Ok(())
    }

    /// Parse arguments and run processor
    fn run_processor(&mut self, args: &clap::ArgMatches) -> RadResult<()> {
        self.parse_options(args);

        // Build processor
        #[allow(unused_mut)]
        let mut processor = Processor::new()
            .set_comment_type(CommentType::from_str(
                if args.occurrences_of("comment") == 0 {
                    "none" // None when no runtime flag
                } else {
                    args.value_of("comment").unwrap() // default is start
                },
            )?)
            .purge(args.is_present("purge"))
            .lenient(args.is_present("lenient"))
            .silent(WarningType::from_str(
                args.value_of("silent").unwrap_or("none"),
            )?)
            .assert(args.is_present("assert"))
            .allow(&self.allow_auth)
            .allow_with_warning(&self.allow_auth_warn)
            .unix_new_line(args.is_present("newline"))
            .melt_files(&self.rules)?
            .discard(args.is_present("discard"));

        // Early return-able procedures
        // - Help
        // - Compile

        // Help command
        #[cfg(feature = "signature")]
        if let Some(name) = args.value_of("manual") {
            if name == "*" {
                // Get all values
                // Sort by name
                // and make it a single stirng, then print
                let sig_map = processor.get_signature_map(SignatureType::All)?;
                let mut manual = sig_map.content.values().collect::<Vec<_>>();
                manual.sort_unstable_by(|a, b| a.name.cmp(&b.name));
                let manual = manual.iter().fold(String::new(), |mut acc, sig| {
                    writeln!(acc, "{0}{1}{1}----", sig, LINE_ENDING).unwrap();
                    acc
                });
                write!(std::io::stdout(), "{}", manual)?
            } else {
                match processor.get_macro_manual(name) {
                    Some(text) => writeln!(std::io::stdout(), "{}", text)?,
                    None => writeln!(
                        std::io::stdout(),
                        "Given macro \"{}\" doesn't exist and cannot display a manual",
                        name
                    )?,
                }
            }
            return Ok(());
        }

        if args.is_present("package") {
            if let Some(sources) = args.values_of("INPUT") {
                let sources = sources.into_iter().map(Path::new).collect::<Vec<_>>();
                processor
                    .package_sources(&sources, self.write_to_file.as_ref().map(|p| p.as_ref()))?;
            } else {
                return Err(RadError::InvalidCommandOption(
                    "No sources were given for packaging.".to_string(),
                ));
            }
            return Ok(());
        }

        // Compile command

        if let Some(file) = self.write_to_file.as_ref() {
            processor = processor.write_to_file(file)?;
        }
        if let Some(file) = self.error_to_file.as_ref() {
            processor = processor.error_to_file(file)?;
        }

        #[cfg(feature = "template")]
        script::extend_processor(&mut processor)?;

        #[cfg(feature = "debug")]
        {
            processor = processor
                .debug(args.is_present("debug"))
                .log(args.is_present("log"))
                .interactive(args.is_present("interactive"))
                .diff(DiffOption::from_str(if args.occurrences_of("diff") == 0 {
                    "none" // None when no runtime flag
                } else {
                    args.value_of("diff").unwrap() // default is all
                })?)?;
        }

        // Update processor
        self.processor = processor;

        // Debug
        // Clear terminal cells
        #[cfg(feature = "debug")]
        if args.is_present("debug") {
            Utils::clear_terminal()?;
        }

        // ========
        // Main options

        // Process type related state changes
        if args.is_present("freeze") {
            if self.write_to_file.is_none() {
                return Err(RadError::InvalidCommandOption(
                    "Freeze options needs an out file to write into".to_string(),
                ));
            }
            self.processor.set_freeze_mode();
        } else if args.is_present("dryrun") {
            self.processor.set_dry_mode();
            self.processor.add_pass_through("anon");
        }

        // print permission
        self.processor.print_permission()?;

        // Process
        // Redirect stdin as argument
        if args.is_present("pipe") {
            let stdin = std::io::stdin();
            let mut input = String::new();
            stdin.lock().read_to_string(&mut input)?;
            self.processor.set_pipe(&input)
        }

        // -->> Read from files or process as raw text
        if let Some(sources) = args.values_of("INPUT") {
            // Also read from stdin if given combiation option
            if args.is_present("combination") {
                // Stdin doesn't work with debug flag
                if args.is_present("debug") {
                    return Err(RadError::InvalidCommandOption(String::from(
                        "Stdin cannot use debug option",
                    )));
                } else {
                    self.processor.process_stdin()?;
                }
            }

            // Interpret every input source as literal text
            let literal = args.is_present("literal");

            // Read from given sources and write with given options
            for src in sources {
                if literal {
                    self.processor.process_string(src)?;
                } else {
                    let src_as_file = Path::new(src);
                    if src_as_file.exists() {
                        // If file extension is .r4c extract from it
                        let result = if src_as_file.extension().unwrap_or_default() == "r4c"
                            || args.is_present("script")
                        {
                            self.processor.set_hygiene(Hygiene::Input);
                            let result = self.processor.process_static_script(src_as_file);
                            self.processor.toggle_hygiene(false);
                            result
                        } else {
                            self.processor.process_file(src_as_file)
                        };

                        match result {
                            // Exit is a sane behaviour and should not exit from whole process
                            Ok(_) => {
                                self.processor.reset_flow_control();
                                continue;
                            }
                            Err(err) => {
                                return Err(err);
                            }
                        }
                    } else {
                        return Err(RadError::InvalidFile(format!("{}", src_as_file.display())));
                    }
                }
            }
            #[cfg(feature = "signature")]
            self.print_signature(args)?;
        } else {
            // -->> Read from stdin

            // Print signature if such option is given
            // Signature option doesn't go with stdin option
            #[cfg(feature = "signature")]
            if self.print_signature(args)? {
                return Ok(());
            }

            // Stdin doesn't work with debug flag
            if args.is_present("debug") {
                return Err(RadError::InvalidCommandOption(String::from(
                    "Stdin cannot use debug option",
                )));
            } else {
                self.processor.process_stdin()?;
            }
        }

        // Print result
        self.processor.print_result()?;

        // Freeze to a rule file if such option was given
        if args.is_present("freeze") {
            match &self.write_to_file {
                Some(file) => self.processor.freeze_to_file(file)?,
                None => {
                    return Err(RadError::InvalidCommandOption(
                        "Freeze options needs an out file to write into".to_string(),
                    ))
                }
            }
        }

        // Clear pass through if it was dryrun
        if args.is_present("dryrun") {
            self.processor.clear_pass_through();
        }

        Ok(())
    }

    /// Print signature
    ///
    /// Returns whether signature operation was executed or not
    #[cfg(feature = "signature")]
    fn print_signature(&mut self, args: &clap::ArgMatches) -> RadResult<bool> {
        #[cfg(feature = "signature")]
        if args.occurrences_of("signature") != 0 {
            let sig_type = SignatureType::from_str(args.value_of("sigtype").unwrap_or("all"))?;
            let sig_map = self.processor.get_signature_map(sig_type)?;
            // TODO
            let sig_json =
                serde_json::to_string(&sig_map.content).expect("Failed to create sig map");

            // This is file name
            let file_name = args.value_of("signature").unwrap();

            // This is default empty value should not be "" because it is ignored by clap
            if file_name != " " {
                std::fs::write(Path::new(file_name), sig_json.as_bytes())?;
            } else {
                writeln!(std::io::stdout(), "{}", &sig_json)?;
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Parse processor options
    fn parse_options(&mut self, args: &clap::ArgMatches) {
        // ========
        // Sub options
        // custom rules
        self.rules = if let Some(files) = args.values_of("melt") {
            files
                .into_iter()
                .map(PathBuf::from)
                .collect::<Vec<PathBuf>>()
        } else {
            vec![]
        };

        // Write to file
        self.write_to_file = args.value_of("out").map(PathBuf::from);

        // Error to file
        self.error_to_file = args.value_of("err").map(PathBuf::from);

        // Permission
        if let Some(auths) = args.value_of("allow") {
            self.allow_auth = auths
                .split('+')
                .into_iter()
                .filter_map(AuthType::from)
                .collect()
        }

        // Permission with warning
        if let Some(auths) = args.value_of("allow_warn") {
            self.allow_auth_warn = auths
                .split('+')
                .into_iter()
                .filter_map(AuthType::from)
                .collect()
        }

        // Permission all
        if args.is_present("allow_all") {
            self.allow_auth = vec![AuthType::FIN, AuthType::FOUT, AuthType::ENV, AuthType::CMD];
        }

        // Permission all with warning
        if args.is_present("allow_all_warn") {
            self.allow_auth_warn =
                vec![AuthType::FIN, AuthType::FOUT, AuthType::ENV, AuthType::CMD];
        }
    }

    /// Create argument requirements
    fn args_builder(source: Option<&[&str]>) -> clap::ArgMatches {
        use clap::{App, Arg};
        let app = App::new("rad")
            .version("3.1")
            .author("Simon creek <simoncreek@tutanota.com>")
            .about( "R4d(rad) is a modern macro processor made with rust. Refer https://github.com/simhyeon/r4d for detailed usage.")
            .long_about("R4d is a text oriented macro processor which aims to be an modern alternative to m4 macro processor. R4d procedurally follows texts and substitue macro calls with defined macro body. R4d comes with varoius useful built in macros so that user don't have to define from scratch. R4d also supports multiple debugging flags for easier error detection. Refer https://github.com/simhyeon/r4d for detailed usage.")
            .override_usage("rad <FILE> -o <OUT_FILE> -e <ERR_FILE>
    echo <STDIN_TEXT | rad 
    echo <STDIN_TEXT> | rad --combination <FILE> --diff
    rad <FILE> --debug --log --interactive
    rad <FILE> --freeze -o <RULE_FILE> --discard -n --silent")
            .arg(Arg::new("INPUT")
                .multiple_values(true)
                .help("INPUT source to execute processing"))
            .arg(Arg::new("pipe")
                .long("pipe")
                .conflicts_with("combination")
                .help("Send stdin as a pipe value"))
            .arg(Arg::new("literal")
                .short('L')
                .long("literal")
                .help("Don't interpret input source as file"))
            .arg(Arg::new("script")
                .long("script")
                .help("Interpret source files as scripts"))
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
                .takes_value(true)
                .default_missing_value("any")
                .value_name("WARNING TYPE")
                .help("Supress warnings (security|sanity|any)"))
            .arg(Arg::new("purge")
                .short('p')
                .long("purge")
                .help("Purge unused macros without panicking. Doesn't work in strict mode"))
            .arg(Arg::new("lenient")
                .short('l')
                .long("lenient")
                .help("Lenient mode, disables strict mode"))
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
                .help("Freeze macros into a single file"))
            .arg(Arg::new("dryrun")
                .long("dryrun")
                .help("Dry run macros"))
            .arg(Arg::new("package")
                .long("package")
                .help("Package sources into a single static file"));

        #[cfg(feature = "signature")]
        let app = app
            .arg(
                Arg::new("manual")
                    .long("man")
                    .takes_value(true)
                    .default_missing_value("*")
                    .value_name("MACRO_NAME")
                    .help("Get manual of a macro"),
            )
            .arg(
                Arg::new("signature")
                    .long("signature")
                    .takes_value(true)
                    .value_name("FILE")
                    .default_missing_value(" ")
                    .help("Print signature to file."),
            )
            .arg(
                Arg::new("sigtype")
                    .long("sigtype")
                    .takes_value(true)
                    .value_name("SIG TYPE")
                    .default_value("all")
                    .help("Signature type to get"),
            );

        if let Some(src) = source {
            app.get_matches_from(src)
        } else {
            app.get_matches()
        }
    }
}
