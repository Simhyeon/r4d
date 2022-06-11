//! # Cli module
//! Cli module takes care of command line argument parsing and executing branches accordingly
//!
//! Cli module is only included in binary feature flag.

use crate::auth::AuthType;
use crate::consts::{DEFAULT_RADIT_EDITOR, RADIT_READ_TEMP_DIR};
use crate::logger::WarningType;
use crate::models::{CommentType, DiffOption};
use crate::processor::Processor;
use crate::utils::Utils;
use crate::RadResult;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

#[cfg(feature = "signature")]
use crate::models::SignatureType;
#[cfg(feature = "signature")]
use std::io::Write;

/// Struct to parse command line arguments and execute proper operations
pub struct Cli {
    rules: Vec<PathBuf>,
    write_to_file: Option<PathBuf>,
    error_to_file: Option<PathBuf>,
    allow_auth: Option<Vec<AuthType>>,
    allow_auth_warn: Option<Vec<AuthType>>,
}

impl Cli {
    pub fn new() -> Self {
        Self {
            rules: vec![],
            write_to_file: None,
            error_to_file: None,
            allow_auth: None,
            allow_auth_warn: None,
        }
    }

    /// User method to call cli workflow
    ///
    /// This sequentially parse command line arguments and execute necessary operations
    pub fn parse(&mut self) -> RadResult<()> {
        let cli_args = Cli::args_builder(None);
        self.run_processor(&cli_args)?;
        Ok(())
    }

    fn parse_from(&mut self, source: &Vec<&str>) -> RadResult<()> {
        let cli_args = Cli::args_builder(Some(source));
        self.run_processor(&cli_args)?;
        Ok(())
    }

    /// Parse arguments and run processor
    fn run_processor(&mut self, args: &clap::ArgMatches) -> RadResult<()> {
        self.parse_options(args);
        // Build processor
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
            ))
            .nopanic(args.is_present("nopanic"))
            .assert(args.is_present("assert"))
            .allow(std::mem::replace(&mut self.allow_auth, None))
            .allow_with_warning(std::mem::replace(&mut self.allow_auth_warn, None))
            .unix_new_line(args.is_present("newline"))
            .melt_files(std::mem::replace(&mut self.rules, vec![]))?
            .write_to_file(std::mem::replace(&mut self.write_to_file, None))?
            .discard(args.is_present("discard"))
            .error_to_file(std::mem::replace(&mut self.error_to_file, None))?;

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
        } else {
            // -->> Read from stdin

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
    fn print_signature(
        &mut self,
        args: &clap::ArgMatches,
        processor: &mut Processor,
    ) -> RadResult<bool> {
        #[cfg(feature = "signature")]
        if args.occurrences_of("signature") != 0 {
            let sig_type = SignatureType::from_str(args.value_of("sigtype").unwrap_or("all"))?;
            let sig_map = processor.get_signature_map(sig_type)?;
            // TODO
            let sig_json =
                serde_json::to_string(&sig_map.object).expect("Failed to create sig map");

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
            let files = files
                .into_iter()
                .map(|value| PathBuf::from(value))
                .collect::<Vec<PathBuf>>();
            files
        } else {
            vec![]
        };

        // Write to file
        self.write_to_file = if let Some(output_file) = args.value_of("out") {
            Some(PathBuf::from(output_file))
        } else {
            None
        };

        // Error to file
        self.error_to_file = if let Some(error_file) = args.value_of("err") {
            Some(PathBuf::from(error_file))
        } else {
            None
        };

        // Permission
        self.allow_auth = if let Some(auths) = args.value_of("allow") {
            auths.split("+").map(|s| AuthType::from(s)).collect()
        } else {
            None
        };

        // Permission with warning
        self.allow_auth_warn = if let Some(auths) = args.value_of("allow_warn") {
            auths.split("+").map(|s| AuthType::from(s)).collect()
        } else {
            None
        };

        // Permission all
        if args.is_present("allow_all") {
            self.allow_auth = Some(vec![
                AuthType::FIN,
                AuthType::FOUT,
                AuthType::ENV,
                AuthType::CMD,
            ]);
        }

        // Permission all with warning
        if args.is_present("allow_all_warn") {
            self.allow_auth_warn = Some(vec![
                AuthType::FIN,
                AuthType::FOUT,
                AuthType::ENV,
                AuthType::CMD,
            ]);
        }
    }

    fn args_builder(source: Option<&Vec<&str>>) -> clap::ArgMatches {
        use clap::{App, Arg};
        let app = App::new("rad")
            .version("2.1.2")
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
                .takes_value(true)
                .default_missing_value("none")
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
        let app = app
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

/// Cli arguments parser for rado binary
pub struct RadoCli {
    flag_arguments: Vec<String>,
    flag_out: Option<PathBuf>,
}

impl RadoCli {
    pub fn new() -> Self {
        Self {
            flag_arguments: vec![],
            flag_out: None,
        }
    }

    /// Parse rado arguments
    pub fn parse(&mut self) -> RadResult<()> {
        let cli_args = self.args_builder();
        self.parse_flags(&cli_args);
        self.run_rado(&cli_args)
    }

    fn parse_flags(&mut self, args: &clap::ArgMatches) {
        match args.subcommand() {
            Some((_, args)) => {
                if let Some(path) = args.value_of("out") {
                    self.flag_out.replace(PathBuf::from(path));
                }

                if let Some(values) = args.value_of("argument") {
                    self.flag_arguments =
                        values.split(' ').map(|s| s.to_string()).collect::<Vec<_>>();
                }
            }
            _ => (),
        }
    }

    fn run_rado(&mut self, args: &clap::ArgMatches) -> RadResult<()> {
        match args.subcommand() {
            Some(("edit", sub_m)) => {
                if let Some(input) = sub_m.value_of("INPUT") {
                    self.view_file(Path::new(input))?;
                }
            }
            Some(("read", sub_m)) => {
                if let Some(input) = sub_m.value_of("INPUT") {
                    let file = self.update_file(input, false)?;
                    self.view_file(&file)?;
                }
            }
            #[cfg(feature = "debug")]
            Some(("diff", sub_m)) => {
                if let Some(input) = sub_m.value_of("INPUT") {
                    self.show_diff(input, sub_m.is_present("force"))?;
                }
            }
            Some(("clear", _)) => {
                let temp_dir = &*RADIT_READ_TEMP_DIR;
                // Remake a directory
                std::fs::remove_dir_all(temp_dir)?;
                std::fs::create_dir(temp_dir)?;
            }
            Some(("force", sub_m)) => {
                if let Some(input) = sub_m.value_of("INPUT") {
                    let path = self.update_file(input, true)?;
                    if sub_m.is_present("read") {
                        self.view_file(&path)?;
                    }
                }
            }
            Some(("sync", sub_m)) => {
                if let Some(input) = sub_m.value_of("INPUT") {
                    use filetime::{set_file_mtime, FileTime};
                    let temp_file = self.get_temp_path(input)?;
                    set_file_mtime(temp_file, FileTime::now())?;
                }
            }
            _ => (),
        }
        // Read file

        Ok(())
    }

    fn args_builder(&self) -> clap::ArgMatches {
        use clap::{App, Arg};
        let mut app = App::new("rado")
            .version("0.1.0")
            .author("Simon creek <simoncreek@tutanota.com>")
            .about( "Rado is a high level wrapper around rad binary")
            .long_about("Rado is a high level wrapper around rad binary. You can either edit a source macro file or read final compiled file with rado wrapper. Rado only compiles when the final product has different timestamp with source file.")
            .override_usage("rado <FILE>
    rado read <FILE>
    rado -s <FILE>
    rado -f <FILE>")
            .arg(Arg::new("argument")
                .last(true)
                .takes_value(true)
                .global(true)
                .help("Send arguments to rad binary"))
            .arg(Arg::new("out")
                .short('o')
                .long("out")
                .takes_value(true)
                .global(true)
                .help("Write to a file"))
            .subcommand(App::new("edit")
                .about("Edit a file as raw")
                .arg(Arg::new("INPUT")
                    .required(true)
                    .help("INPUT source to execute processing")
                )
            )
            .subcommand(App::new("clear")
                .about("Clear temp directory")
            )
            .subcommand(App::new("read")
                .about("Read a file")
                .arg(Arg::new("INPUT")
                    .required(true)
                    .help("INPUT name to read from temp directory")
                )
            )
            .subcommand(App::new("force")
                .about("Force update a file")
                .arg(Arg::new("INPUT")
                    .required(true)
                    .help("INPUT name to force update")
                )
                .arg(Arg::new("read")
                    .short('r')
                    .long("read")
                    .help("Also open a file")
                )
            )
            .subcommand(App::new("sync")
                .about("Sync file's timestamp")
                .arg(Arg::new("INPUT")
                    .required(true)
                    .help("INPUT name to sync")
                )
            );

        #[cfg(feature = "debug")]
        {
            app = app.subcommand(
                App::new("diff")
                    .about("Show difference between files")
                    .arg(
                        Arg::new("INPUT")
                            .required(true)
                            .help("INPUT name to read from temp directory"),
                    )
                    .arg(
                        Arg::new("force")
                            .short('f')
                            .long("force")
                            .help("Force update before showding diff"),
                    ),
            );
        }
        app.get_matches()
    }

    fn view_file(&self, file: &Path) -> RadResult<()> {
        // View file
        let editor = if let Ok(editor) = std::env::var("RADIT_EDITOR") {
            editor
        } else {
            DEFAULT_RADIT_EDITOR.to_string()
        };

        let proc_args = vec![editor.as_str(), file.to_str().as_ref().unwrap()];
        Utils::subprocess(&proc_args)?;
        Ok(())
    }

    #[cfg(feature = "debug")]
    fn show_diff(&self, file: &str, force: bool) -> RadResult<()> {
        use similar::ChangeTag;

        let temp_file = self.get_temp_path(file)?;

        // Prevent panicking
        if force | !temp_file.exists() {
            let mut out = std::io::stdout();
            write!(&out,"Try expanding source file without any arguments because diff target is not present.\n")?;
            out.flush()?;
            self.update_file(file, true)?;
        }

        let source_content = std::fs::read_to_string(file)?;
        let target_content = std::fs::read_to_string(&temp_file)?;

        let result = similar::TextDiff::from_lines(&source_content, &target_content);
        let mut log: String;
        // Color function reference
        let mut colorfunc: Option<fn(string: &str) -> Box<dyn std::fmt::Display>>;

        // Print changes with color support
        for change in result.iter_all_changes() {
            colorfunc = None;
            match change.tag() {
                ChangeTag::Delete => {
                    log = format!("- {}", change);
                    colorfunc.replace(Utils::red);
                }
                ChangeTag::Insert => {
                    log = format!("+ {}", change);
                    colorfunc.replace(Utils::green);
                }
                ChangeTag::Equal => {
                    log = format!("  {}", change);
                }
            }

            if let Some(func) = colorfunc {
                log = func(&log).to_string(); // Apply color
            }
            write!(std::io::stdout(), "{}", log)?;
        }
        Ok(())
    }

    fn get_temp_path(&self, path: &str) -> RadResult<PathBuf> {
        // Create temp directory if not present
        if !RADIT_READ_TEMP_DIR.exists() {
            std::fs::create_dir(&*RADIT_READ_TEMP_DIR)?;
        }

        let temp_dir = if let Ok(dir) = std::env::var("RADIT_DIR") {
            PathBuf::from(dir)
        } else {
            RADIT_READ_TEMP_DIR.to_path_buf()
        };
        let temp_file = temp_dir.join(path);
        Ok(temp_file)
    }

    fn update_file(&self, path: &str, force: bool) -> RadResult<PathBuf> {
        let source_file = path;
        let mut target_file = self.get_temp_path(path)?;
        if target_file.exists() {
            // TODO
            // Compare file_stamp
            let source_modified = std::fs::metadata(&source_file)?.modified()?;
            let temp_modified = std::fs::metadata(&target_file)?.modified()?;

            // Source is freshier or force
            if force | (source_modified > temp_modified) {
                let mut rad_args = vec!["rad", source_file];
                if self.flag_arguments.len() != 0 {
                    rad_args.extend(self.flag_arguments.iter().map(|s| s.as_str()));
                }

                // Respect user configured out option
                if let Some(path) = self.flag_out.as_ref() {
                    target_file = path.to_path_buf();
                }

                rad_args.extend(vec!["-o", target_file.to_str().as_ref().unwrap()].iter());
                Cli::new().parse_from(&rad_args)?;
            }
        } else {
            // File doesn't exist, run it anyway
            let mut rad_args = vec!["rad", source_file];
            if self.flag_arguments.len() != 0 {
                rad_args.extend(self.flag_arguments.iter().map(|s| s.as_str()));
            }
            rad_args.extend(vec!["-o", target_file.to_str().as_ref().unwrap()].iter());
            Cli::new().parse_from(&rad_args)?;
        }

        Ok(target_file)
    }
}
