//! Cli processor for rad binary

use clap::ArgAction;

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
            .set_comment_type(CommentType::from_str(if !args.contains_id("comment") {
                "none" // None when no runtime flag
            } else {
                args.get_one::<String>("comment").unwrap() // default is start
            })?)
            .purge(args.get_flag("purge"))
            .lenient(args.get_flag("lenient"))
            // TODO
            // ReallY? this is outrageous
            .silent(WarningType::from_str(
                args.get_one::<String>("silent")
                    .unwrap_or(&"none".to_string()),
            )?)
            .assert(args.get_flag("assert"))
            .allow(&self.allow_auth)
            .allow_with_warning(&self.allow_auth_warn)
            .unix_new_line(args.get_flag("newline"))
            .melt_files(&self.rules)?
            .discard(args.get_flag("discard"));

        // Early return-able procedures
        // - Help
        // - Compile

        // Help command
        #[cfg(feature = "signature")]
        if let Some(name) = args.get_one::<String>("manual") {
            if name == &"*" {
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

        // Search a macro
        #[cfg(feature = "signature")]
        if let Some(name) = args.get_one::<String>("search") {
            match processor.get_macro_manual(name) {
                Some(text) => writeln!(std::io::stdout(), "{}", text)?,
                None => writeln!(
                    std::io::stdout(),
                    "Given macro \"{}\" doesn't exist and cannot display a manual",
                    name
                )?,
            }
            return Ok(());
        }

        if args.get_flag("package") {
            if let Some(sources) = args.get_many::<String>("INPUT") {
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
                .debug(args.get_flag("debug"))
                .log(args.get_flag("log"))
                .interactive(args.get_flag("interactive"))
                .diff(DiffOption::from_str(if !args.contains_id("diff") {
                    "none" // None when no runtime flag
                } else {
                    args.get_one::<String>("diff").unwrap() // default is all
                })?)?;
        }

        // Update processor
        self.processor = processor;

        // Debug
        // Clear terminal cells
        #[cfg(feature = "debug")]
        if args.get_flag("debug") {
            Utils::clear_terminal()?;
        }

        // ========
        // Main options

        // Process type related state changes
        if args.get_flag("freeze") {
            if self.write_to_file.is_none() {
                return Err(RadError::InvalidCommandOption(
                    "Freeze options needs an out file to write into".to_string(),
                ));
            }
            self.processor.set_freeze_mode();
        } else if args.get_flag("dryrun") {
            self.processor.set_dry_mode();
            self.processor.add_pass_through("anon");
        }

        // print permission
        self.processor.print_permission()?;

        // Process
        // Redirect stdin as argument
        // Pipe doesn't evaluate contents but simply save contents into a pipe
        if args.get_flag("pipe") {
            let stdin = std::io::stdin();
            let mut input = String::new();
            stdin.lock().read_to_string(&mut input)?;
            self.processor.set_pipe(&input)
        }

        // -->> Read from files or process as raw text
        if let Some(sources) = args.get_many::<String>("INPUT") {
            // Also read from stdin if given combiation option
            if args.get_flag("combination") {
                // Stdin doesn't work with debug flag
                if args.get_flag("debug") {
                    return Err(RadError::InvalidCommandOption(String::from(
                        "Stdin cannot use debug option",
                    )));
                } else {
                    self.processor.process_stdin()?;
                }
            }

            // Interpret every input source as literal text
            let literal = args.get_flag("literal");

            // Read from given sources and write with given options
            for src in sources {
                if literal {
                    self.processor.process_string(src)?;
                } else {
                    let src_as_file = Path::new(src);
                    if src_as_file.exists() {
                        // If file extension is .r4c extract from it
                        let result = if src_as_file.extension().unwrap_or_default() == "r4c"
                            || args.get_flag("script")
                        {
                            self.processor.set_hygiene(Hygiene::Input);
                            let result = self.processor.process_static_script(src_as_file);
                            self.processor.toggle_hygiene(false);
                            result
                        } else if let Some(mac) = args.get_one::<String>("stream-chunk") {
                            // Execute macros on each file
                            self.processor.stream_by_chunk(
                                &std::fs::read_to_string(src)?,
                                Some("src"),
                                mac,
                            )
                        } else if let Some(mac) = args.get_one::<String>("stream-lines") {
                            let file_stream = std::fs::File::open(src)?;
                            let mut reader = std::io::BufReader::new(file_stream);
                            self.processor
                                .stream_by_lines(&mut reader, Some("src"), mac)
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
            if args.get_flag("debug") {
                return Err(RadError::InvalidCommandOption(String::from(
                    "Stdin cannot use debug option",
                )));
            } else if let Some(mac) = args.get_one::<String>("stream-chunk") {
                // Read stdin as chunk and stream to a macro
                let mut input = String::new();
                std::io::stdin().read_to_string(&mut input)?;
                self.processor.stream_by_chunk(&input, None, mac)?;
            } else if let Some(mac) = args.get_one::<String>("stream-lines") {
                // Read stdin as line buffer and streams to a macro
                #[allow(unused_imports)]
                // TODO This was copy pasted check if this is necessary
                use std::io::Read;
                let stdin = std::io::stdin();
                let mut reader = stdin.lock();
                self.processor.stream_by_lines(&mut reader, None, mac)?;
            } else {
                self.processor.process_stdin()?;
            }
        }

        // Print result
        self.processor.print_result()?;

        // Freeze to a rule file if such option was given
        if args.get_flag("freeze") {
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
        if args.get_flag("dryrun") {
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
        if args.contains_id("signature") {
            // TODO
            // Outragoues
            let sig_type = SignatureType::from_str(
                args.get_one::<String>("sigtype")
                    .unwrap_or(&"all".to_string()),
            )?;
            let sig_map = self.processor.get_signature_map(sig_type)?;

            let sig_json =
                serde_json::to_string(&sig_map.content).expect("Failed to create sig map");

            // This is file name
            let file_name = args.get_one::<String>("signature").unwrap();

            // TODO Check if this behaviour persists on clap version 4.0
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
        self.rules = if let Some(files) = args.get_many::<String>("melt") {
            files
                .into_iter()
                .map(PathBuf::from)
                .collect::<Vec<PathBuf>>()
        } else {
            vec![]
        };

        // Write to file
        self.write_to_file = args.get_one::<String>("out").map(PathBuf::from);

        // Error to file
        self.error_to_file = args.get_one::<String>("err").map(PathBuf::from);

        // Permission
        if let Some(auths) = args.get_one::<String>("allow") {
            self.allow_auth = auths.split('+').filter_map(AuthType::from).collect()
        }

        // Permission with warning
        if let Some(auths) = args.get_one::<String>("allow_warn") {
            self.allow_auth_warn = auths.split('+').filter_map(AuthType::from).collect()
        }

        // Permission all
        if args.get_flag("allow_all") {
            self.allow_auth = vec![AuthType::FIN, AuthType::FOUT, AuthType::ENV, AuthType::CMD];
        }

        // Permission all with warning
        if args.get_flag("allow_all_warn") {
            self.allow_auth_warn =
                vec![AuthType::FIN, AuthType::FOUT, AuthType::ENV, AuthType::CMD];
        }
    }

    /// Create argument requirements
    fn args_builder(source: Option<&[&str]>) -> clap::ArgMatches {
        use clap::{Arg, Command};
        let app = Command::new("rad")
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
                .num_args(0..)
                .help("INPUT source to execute processing"))
            .arg(Arg::new("pipe")
                .long("pipe")
                .action(ArgAction::SetTrue)
                .conflicts_with("combination")
                .help("Send stdin as a pipe value without evaluation"))
            .arg(Arg::new("literal")
                .short('L')
                .action(ArgAction::SetTrue)
                .long("literal")
                .help("Don't interpret input source as file"))
            .arg(Arg::new("stream-chunk")
                .action(ArgAction::Set)
                .long("stream-chunk")
                .conflicts_with_all(["pipe", "combination", "literal", "stream-lines"])
                .help("Stream contents to a macro execution"))
            .arg(Arg::new("stream-lines")
                .action(ArgAction::Set)
                .long("stream-lines")
                .conflicts_with_all(["pipe", "combination", "literal", "stream-chunk"])
                .help("Stream contents to a macro execution but by lines"))
            .arg(Arg::new("script")
                .long("script")
                .action(ArgAction::SetTrue)
                .help("Interpret source files as scripts"))
            .arg(Arg::new("out")
                .short('o')
                .long("out")
                .action(ArgAction::Set)
                .conflicts_with("discard")
                .value_name("FILE")
                .help("Save processed output to the file"))
            .arg(Arg::new("err")
                .short('e')
                .long("err")
                .action(ArgAction::Set)
                .value_name("FILE")
                .help("Save error logs to the file"))
            .arg(Arg::new("combination")
                .short('c')
                .long("combination")
                .action(ArgAction::SetTrue)
                .help("Read from both stdin and file inputs. Stdin is evaluated first"))
            .arg(Arg::new("discard")
                .short('D')
                .long("discard")
                .action(ArgAction::SetTrue)
                .help("Discard output"))
            .arg(Arg::new("silent")
                .short('s')
                .long("silent")
                .action(ArgAction::Set)
                .default_missing_value("any")
                .value_name("WARNING TYPE")
                .help("Supress warnings (security|sanity|any)"))
            .arg(Arg::new("purge")
                .short('p')
                .long("purge")
                .action(ArgAction::SetTrue)
                .help("Purge unused macros without panicking. Doesn't work in strict mode"))
            .arg(Arg::new("lenient")
                .short('l')
                .long("lenient")
                .action(ArgAction::SetTrue)
                .help("Lenient mode, disables strict mode"))
            .arg(Arg::new("debug")
                .short('d')
                .long("debug")
                .action(ArgAction::SetTrue)
                .help("Debug mode"))
            .arg(Arg::new("log")
                .long("log")
                .action(ArgAction::SetTrue)
                .help("Print log for every macro invocation. Only works on debug mode"))
            .arg(Arg::new("diff")
                .long("diff")
                .action(ArgAction::Set)
                .value_name("DIFF TYPE")
                .default_missing_value("all")
                .help("Show diff result (none|change|all)"))
            .arg(Arg::new("interactive")
                .short('i')
                .long("interactive")
                .action(ArgAction::SetTrue)
                .help("Use interactive debug mode. This enables line wrapping."))
            .arg(Arg::new("assert")
                .long("assert")
                .action(ArgAction::SetTrue)
                .help("Enable assert mode"))
            .arg(Arg::new("comment")
                .long("comment")
                .action(ArgAction::Set)
                .default_missing_value("start")
                .value_name("COMMENT TYPE")
                .help("Use comment option (none|start|any)"))
            .arg(Arg::new("allow")
                .short('a')
                .action(ArgAction::Set)
                .value_name("AUTH TYPE")
                .help("Allow permission (fin|fout|cmd|env)"))
            .arg(Arg::new("allow_warn")
                .short('w')
                .action(ArgAction::Set)
                .value_name("AUTH TYPE")
                .help("Allow permission with warnings (fin|fout|cmd|env)"))
            .arg(Arg::new("allow_all")
                .short('A')
                .action(ArgAction::SetTrue)
                .conflicts_with("allow_all_warn")
                .help("Allow all permission"))
            .arg(Arg::new("allow_all_warn")
                .short('W')
                .action(ArgAction::SetTrue)
                .conflicts_with("allow_all")
                .help("Allow all permission with warning"))
            .arg(Arg::new("newline")
                .short('n')
                .long("newline")
                .action(ArgAction::SetTrue)
                .help("Use unix newline for formatting"))
            .arg(Arg::new("melt")
                .short('m')
                .long("melt")
                .action(ArgAction::Append)
                .value_name("FILE")
                .help("Read macros from frozen file"))
            .arg(Arg::new("freeze") // TODO, should this arg accept value?
                .short('f')
                .long("freeze")
                .action(ArgAction::SetTrue)
                .help("Freeze macros into a single file"))
            .arg(Arg::new("dryrun")
                .long("dryrun")
                .action(ArgAction::SetTrue)
                .help("Dry run macros"))
            .arg(Arg::new("package")
                .long("package")
                .action(ArgAction::SetTrue)
                .help("Package sources into a single static file"));

        #[cfg(feature = "signature")]
        let app = app
            .arg(
                Arg::new("manual")
                    .long("man")
                    .action(ArgAction::Set)
                    .default_missing_value("*")
                    .value_name("MACRO_NAME")
                    .help("Get manual of a macro"),
            )
            .arg(
                Arg::new("search")
                    .long("search")
                    .action(ArgAction::Set)
                    .value_name("MACRO_NAME")
                    .help("Search for a macro"),
            )
            .arg(
                Arg::new("signature")
                    .long("signature")
                    .action(ArgAction::Set)
                    .value_name("FILE")
                    .default_missing_value(" ")
                    .help("Print signature to file."),
            )
            .arg(
                Arg::new("sigtype")
                    .long("sigtype")
                    .action(ArgAction::Set)
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
