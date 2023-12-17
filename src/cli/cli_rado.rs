//! Cli processor for rado binary

use crate::utils::Utils;
use crate::RadResult;
use crate::{RadCli, RadError};
use clap::ArgAction;
use once_cell::sync::Lazy;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Temporary rado directory
pub static RADO_DIR: Lazy<std::path::PathBuf> = Lazy::new(|| std::env::temp_dir().join("rado"));

#[cfg(windows)]
/// Default rado editor
pub(crate) const RADO_EDITOR: &str = "notepad.exe";
#[cfg(not(windows))]
/// Default rado editor
pub(crate) const RADO_EDITOR: &str = "vim";

/// Cli arguments parser for rado binary
pub struct RadoCli {
    flag_arguments: Vec<String>,
    flag_out: Option<PathBuf>,
}

impl Default for RadoCli {
    fn default() -> Self {
        Self::new()
    }
}

impl RadoCli {
    /// Create a new instance
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

    /// Parse flag occurrences from matches
    fn parse_flags(&mut self, args: &clap::ArgMatches) {
        if let Some((_, args)) = args.subcommand() {
            if let Some(path) = args.get_one::<&str>("out") {
                self.flag_out.replace(PathBuf::from(path));
            }

            if let Some(values) = args.get_many::<&str>("argument") {
                self.flag_arguments = values.map(|s| s.to_string()).collect();
            }
        }
    }

    /// Run rado binary according to flag matches
    fn run_rado(&mut self, args: &clap::ArgMatches) -> RadResult<()> {
        match args.subcommand() {
            Some(("env", _)) => {
                write!(
                    std::io::stdout(),
                    "RADO_EDITOR\t: {}\nRADO_DIR\t: {}\n",
                    &*RADO_EDITOR,
                    (&*RADO_DIR).display()
                )?;
            }
            Some(("package", sub_m)) => {
                if let Some(input) = sub_m.get_one::<&str>("INPUT") {
                    self.package_file(Path::new(input))?;
                }
            }
            Some(("execute", sub_m)) => {
                if let Some(input) = sub_m.get_one::<&str>("INPUT") {
                    self.execute(Path::new(input))?;
                }
            }
            Some(("replace", sub_m)) => {
                if let Some(input) = sub_m.get_one::<&str>("INPUT") {
                    self.replace_file(Path::new(input))?;
                }
            }
            Some(("edit", sub_m)) => {
                if let Some(input) = sub_m.get_one::<&str>("INPUT") {
                    self.view_file(Path::new(input))?;
                }
            }
            Some(("read", sub_m)) => {
                if let Some(input) = sub_m.get_one::<&str>("INPUT") {
                    let file = self.update_file(input, false)?;
                    self.view_file(&file)?;
                }
            }
            #[cfg(feature = "debug")]
            Some(("diff", sub_m)) => {
                if let Some(input) = sub_m.get_one::<&str>("INPUT") {
                    self.show_diff(input, sub_m.get_flag("force"))?;
                }
            }
            Some(("clear", _)) => {
                let temp_dir = &*RADO_DIR;
                // Remake a directory
                std::fs::remove_dir_all(temp_dir)?;
                std::fs::create_dir(temp_dir)?;
            }
            Some(("force", sub_m)) => {
                if let Some(input) = sub_m.get_one::<&str>("INPUT") {
                    let path = self.update_file(input, true)?;
                    if sub_m.get_flag("read") {
                        self.view_file(&path)?;
                    }
                }
            }
            Some(("sync", sub_m)) => {
                if let Some(input) = sub_m.get_one::<&str>("INPUT") {
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

    /// Create argument requirements
    fn args_builder(&self) -> clap::ArgMatches {
        use clap::{Arg, Command};
        #[allow(unused_mut)]
        let mut app = Command::new("rado")
            .version("0.1.0")
            .author("Simon creek <simoncreek@tutanota.com>")
            .about( "Rado is a high level wrapper around rad binary")
            .long_about("Rado is a high level wrapper around rad binary. You can either edit a source macro file or read final compiled file with rado wrapper. Rado only compiles when the final product has different timestamp with source file.")
            .override_usage("rado <FILE>
    rado read <FILE> -- arguments
    rado edit <FILE>
    rado force <FILE>
    rado execute <FILE>
    rado package <FILE>
    rado diff <FILE>
    rado sync <FILE>")
            .arg(Arg::new("argument")
                .last(true)
                .action(ArgAction::Append)
                .num_args(0..)
                .global(true)
                .help("Send arguments to rad binary"))
            .arg(Arg::new("out")
                .short('o')
                .long("out")
                .action(ArgAction::Set)
                .global(true)
                .help("Write to a file"))
            .subcommand(Command::new("replace")
                .about("In-replace a file's content")
                .after_help("This creates cache file in temp directory's rado_repl.txt")
                .arg(Arg::new("INPUT")
                    .required(true)
                    .action(ArgAction::Set)
                    .help("INPUT source to execute processing")
                )
            )
            .subcommand(Command::new("edit")
                .about("Edit a file as raw")
                .arg(Arg::new("INPUT")
                    .required(true)
                    .action(ArgAction::Set)
                    .help("INPUT source to execute processing")
                )
            )
            .subcommand(Command::new("clear")
                .about("Clear temp directory")
            )
            .subcommand(Command::new("env")
                .about("Print env information")
            )
            .subcommand(Command::new("package")
                .about("Package into a file")
                .arg(Arg::new("INPUT")
                    .required(true)
                    .action(ArgAction::Set)
                    .help("File name to package")
                )
            )
            .subcommand(Command::new("execute")
                .about("Execute a file")
                .arg(Arg::new("INPUT")
                    .required(true)
                    .action(ArgAction::Set)
                    .help("File name to execute")
                )
            )
            .subcommand(Command::new("read")
                .about("Read a file")
                .arg(Arg::new("INPUT")
                    .required(true)
                    .action(ArgAction::Set)
                    .help("File name to read from temp directory")
                )
            )
            .subcommand(Command::new("force")
                .about("Force update a file")
                .arg(Arg::new("INPUT")
                    .required(true)
                    .action(ArgAction::Set)
                    .help("File name to force update")
                )
                .arg(Arg::new("read")
                    .short('r')
                    .long("read")
                    .action(ArgAction::SetTrue)
                    .help("Also open a file")
                )
            )
            .subcommand(Command::new("sync")
                .about("Sync file's timestamp")
                .arg(Arg::new("INPUT")
                    .required(true)
                    .action(ArgAction::Set)
                    .help("File name to sync")
                )
            );

        #[cfg(feature = "debug")]
        {
            app = app.subcommand(
                Command::new("diff")
                    .about("Show difference between files")
                    .arg(
                        Arg::new("INPUT")
                            .required(true)
                            .action(ArgAction::Set)
                            .help("File name to read from temp directory"),
                    )
                    .arg(
                        Arg::new("force")
                            .short('f')
                            .long("force")
                            .action(ArgAction::SetTrue)
                            .help("Force update before showding diff"),
                    ),
            );
        }
        app.get_matches()
    }

    /// Execute rad processing
    fn execute(&self, file: &Path) -> RadResult<()> {
        if file.exists() {
            let file_string = file.display().to_string();
            let mut rad_args = vec!["rad", file_string.as_ref()];
            if !self.flag_arguments.is_empty() {
                rad_args.extend(self.flag_arguments.iter().map(|s| s.as_str()));
            }
            RadCli::new().parse_from(&rad_args)?;
        }
        Ok(())
    }

    /// Package a static script
    fn package_file(&self, file: &Path) -> RadResult<()> {
        if file.exists() {
            let file_string = file.display().to_string();
            let out_string = file.with_extension("r4c").display().to_string();
            let mut rad_args = vec!["rad", file_string.as_ref(), "--package", "-o", &out_string];
            if !self.flag_arguments.is_empty() {
                rad_args.extend(self.flag_arguments.iter().map(|s| s.as_str()));
            }
            RadCli::new().parse_from(&rad_args)?;
        }
        Ok(())
    }

    /// Replace file's content
    fn replace_file(&self, file: &Path) -> RadResult<()> {
        if file.exists() {
            let file_string = file.display().to_string();
            let mut rad_args = vec!["rad", file_string.as_ref()];
            if !self.flag_arguments.is_empty() {
                rad_args.extend(self.flag_arguments.iter().map(|s| s.as_str()));
            }
            let cached_path = std::env::temp_dir().join("rado_repl.txt");
            let cached_string = cached_path.display().to_string();

            rad_args.extend(vec!["-o", &cached_string].iter());
            {
                RadCli::new().parse_from(&rad_args)?;
            }

            std::fs::copy(cached_path, file)?;
        }
        Ok(())
    }

    /// View a file
    fn view_file(&self, file: &Path) -> RadResult<()> {
        // View file
        let editor = if let Ok(editor) = std::env::var("RADO_EDITOR") {
            editor
        } else {
            RADO_EDITOR.to_string()
        };

        let proc_args = vec![editor.as_str(), file.to_str().as_ref().unwrap()];
        Utils::subprocess(&proc_args)?;
        Ok(())
    }

    #[cfg(feature = "debug")]
    /// Show difference of processing
    fn show_diff(&self, file: &str, force: bool) -> RadResult<()> {
        #[cfg(feature = "color")]
        use crate::consts::ColorDisplayFunc;
        use similar::ChangeTag;

        let temp_file = self.get_temp_path(file)?;

        // Prevent panicking
        if force | !temp_file.exists() {
            let mut out = std::io::stdout();
            writeln!(&out,"Try expanding source file without any arguments because diff target is not present.")?;
            out.flush()?;
            self.update_file(file, true)?;
        }

        let source_content = std::fs::read_to_string(file)?;
        let target_content = std::fs::read_to_string(&temp_file)?;

        let result = similar::TextDiff::from_lines(&source_content, &target_content);
        let mut log: String;
        // Color function reference
        #[cfg(feature = "color")]
        let mut colorfunc: Option<ColorDisplayFunc>;
        #[cfg(feature = "color")]
        let is_stdout: bool;
        #[cfg(feature = "color")]
        {
            colorfunc = None;
            is_stdout = atty::is(atty::Stream::Stdout);
            if is_stdout {
                colorfunc.replace(|string: &str, _| -> Box<dyn std::fmt::Display> {
                    Box::new(string.to_owned())
                });
            }
        }

        // Print changes with color support
        for change in result.iter_all_changes() {
            #[cfg(feature = "color")]
            {
                colorfunc = None;
            }
            match change.tag() {
                ChangeTag::Delete => {
                    log = format!("- {}", change);
                    #[cfg(feature = "color")]
                    {
                        if is_stdout {
                            colorfunc.replace(Utils::red);
                        }
                    }
                }
                ChangeTag::Insert => {
                    log = format!("+ {}", change);
                    #[cfg(feature = "color")]
                    {
                        if is_stdout {
                            colorfunc.replace(Utils::green);
                        }
                    }
                }
                ChangeTag::Equal => {
                    log = format!("  {}", change);
                }
            }

            #[cfg(feature = "color")]
            {
                if let Some(func) = colorfunc {
                    log = func(&log, !is_stdout).to_string(); // Commandly color
                }
            }
            write!(std::io::stdout(), "{}", log)?;
        }
        Ok(())
    }

    /// Get temporary file path
    fn get_temp_path(&self, path: &str) -> RadResult<PathBuf> {
        // Create temp directory if not present
        if !RADO_DIR.exists() {
            std::fs::create_dir(&*RADO_DIR)?;
        }

        let temp_dir = if let Ok(dir) = std::env::var("RADO_DIR") {
            PathBuf::from(dir)
        } else {
            RADO_DIR.to_path_buf()
        };
        let temp_file = temp_dir.join(path);
        Ok(temp_file)
    }

    /// Update file's content
    fn update_file(&self, path: &str, force: bool) -> RadResult<PathBuf> {
        let source_file = path;
        let mut target_file = self.get_temp_path(path)?;
        if target_file.exists() {
            // Compare file_stamp
            let source_modified = std::fs::metadata(source_file)?.modified()?;
            let temp_modified = std::fs::metadata(&target_file)?.modified()?;

            // Source is freshier or force
            if force | (source_modified > temp_modified) {
                let mut rad_args = vec!["rad", source_file];
                if !self.flag_arguments.is_empty() {
                    rad_args.extend(self.flag_arguments.iter().map(|s| s.as_str()));
                }

                // Respect user configured out option
                if let Some(path) = self.flag_out.as_ref() {
                    target_file = path.to_path_buf();
                }

                rad_args.extend(vec!["-o", target_file.to_str().as_ref().unwrap()].iter());
                RadCli::new().parse_from(&rad_args)?;
            }
        } else {
            if !Path::new(source_file).exists() {
                return Err(RadError::InvalidCommandOption(format!(
                    "Cannot read from non-existent file : \"{}\"",
                    source_file
                )));
            }

            // File doesn't exist, run it anyway
            let mut rad_args = vec!["rad", source_file];
            if !self.flag_arguments.is_empty() {
                rad_args.extend(self.flag_arguments.iter().map(|s| s.as_str()));
            }
            rad_args.extend(vec!["-o", target_file.to_str().as_ref().unwrap()].iter());
            RadCli::new().parse_from(&rad_args)?;
        }

        Ok(target_file)
    }
}
