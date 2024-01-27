//! Utility struct, methods for various operations

use crate::auth::{AuthState, AuthType};
use crate::common::{ProcessInput, RadResult};
use crate::error::RadError;
use crate::logger::WarningType;
use crate::{NewArgParser, Processor, WriteOption};
use once_cell::sync::Lazy;
use regex::Regex;
use std::borrow::Cow;
use std::ffi::OsStr;
use std::io::BufRead;
use std::path::Path;

use crate::common::RelayTarget;

// Thanks stack overflow! SRC : https://stackoverflow.com/questions/12643009/regular-expression-for-floating-point-numbers
/// Number matches
pub static NUM_MATCH: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"[+-]?([\d]*[.])?\d+"#).expect("Failed to create number regex"));

// ----------
// MACRO RULES
// ----------

/// string std::mem::take
#[macro_export]
macro_rules! stake {
    ($e:expr) => {
        std::mem::take(&mut $e)
    };
}

// Include function macro manaul
#[macro_export]
macro_rules! man_fun {
    ($fname:expr) => {
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/manual_src/fun/",
            $fname
        ))
        .to_string() // assumes Linux ('/')!
    };
}

// Include deterred macro manaul
#[macro_export]
macro_rules! man_det {
    ($fname:expr) => {
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/manual_src/det/",
            $fname
        ))
        .to_string() // assumes Linux ('/')!
    };
}
// ----------

#[cfg(feature = "color")]
use colored::*;

/// Utility relates struct
pub(crate) struct Utils;

impl Utils {
    pub fn get_split_arguments_or_error<'a>(
        macro_name: &'a str,
        arguments: &'a &str,
        length: usize,
        mut parser: NewArgParser<'a>,
    ) -> RadResult<Vec<Cow<'a, str>>> {
        if let Some(args) = parser.args_with_len(arguments, length) {
            Ok(args)
        } else {
            let error_message = match length {
                0 => format!("{} does not require any arguments", macro_name),
                1 => format!("{} requires an argument", macro_name),
                _ => format!("{} requires {} arguments", macro_name, length),
            };
            Err(RadError::InvalidArgument(error_message))
        }
    }

    /// Split string by whitespaces but respect spaces within commas and strip commas
    ///
    /// # Example
    ///
    /// "a b c"           -> [a,b,c]
    /// "a b ' ' d"       -> [a,b, ,d]
    /// "a b c' ' d"      -> [a,b,c ,d]
    /// "a b ' c f g ' d" -> ["a","b"," c f g ","d",]
    pub fn get_whitespace_split_retain_quote_rule<'a>(args: &'a str) -> Vec<Cow<'a, str>> {
        let mut split: Vec<Cow<'a, str>> = vec![];
        let args = args.trim();
        if args.is_empty() {
            return split;
        }
        let mut index = 0usize;
        let mut quoted = false;
        let mut strip_quote = false;
        let mut prev = '0';
        for (idx, ch) in args.chars().enumerate() {
            // Decide whether split needs or not
            if ch == ' ' {
                if quoted {
                    // Ignore space
                    // continue;
                } else {
                    // Not quoted

                    // if previous was also space
                    // don't split and continue
                    if prev == ' ' {
                    } else {
                        // Previous was concrete character
                        if strip_quote {
                            split.push(args[index..idx].trim_start().replace('\'', "").into());
                            strip_quote = false;
                        } else {
                            split.push(args[index..idx].trim_start().into());
                        }
                    }
                    index = idx; // Update index
                }
            }
            if ch == '\'' {
                quoted = !quoted;
                strip_quote = true;
            }

            // GLobal process
            prev = ch;
        }

        // Collect last
        if strip_quote {
            split.push(args[index..].trim_start().replace('\'', "").into());
        } else {
            split.push(args[index..].trim_start().into());
        }
        split
    }

    /// Split macro name and arguments from given source
    pub(crate) fn get_name_n_arguments(
        src: &str,
        with_trailing_comma: bool,
    ) -> RadResult<(&str, String)> {
        let macro_src_joined = src.split_whitespace().collect::<Vec<_>>();
        let macro_name = macro_src_joined.first().ok_or(RadError::InvalidArgument(
            "Macro name cannot be empty".to_string(),
        ))?;
        let mut macro_arguments = macro_src_joined[1..].join(",");
        if with_trailing_comma && !macro_arguments.is_empty() {
            macro_arguments.push(',');
        }
        Ok((macro_name, macro_arguments))
    }

    /// Generic levenshtein distance function
    ///
    /// SOURCE : https://en.wikibooks.org/wiki/Algorithm_Implementation/Strings/Levenshtein_distance#Rust
    pub(crate) fn levenshtein<T1: AsRef<[u8]>, T2: AsRef<[u8]>>(s1: T1, s2: T2) -> usize {
        let v1 = s1.as_ref();
        let v2 = s2.as_ref();

        // Early exit if one of the strings is empty
        let v1len = v1.len();
        let v2len = v2.len();
        if v1len == 0 {
            return v2len;
        }
        if v2len == 0 {
            return v1len;
        }

        #[inline]
        fn min3<T: Ord>(v1: T, v2: T, v3: T) -> T {
            std::cmp::min(v1, std::cmp::min(v2, v3))
        }

        #[inline]
        fn delta(x: u8, y: u8) -> usize {
            if x == y {
                0
            } else {
                1
            }
        }

        let mut column: Vec<usize> = (0..v1len + 1).collect();
        for x in 1..v2len + 1 {
            column[0] = x;
            let mut lastdiag = x - 1;

            for y in 1..v1len + 1 {
                let olddiag = column[y];
                column[y] = min3(
                    column[y] + 1,
                    column[y - 1] + 1,
                    lastdiag + delta(v1[y - 1], v2[x - 1]),
                );
                lastdiag = olddiag;
            }
        }

        column[v1len]
    }

    /// Create a local name from level and name
    pub(crate) fn local_name(level: usize, name: &str) -> String {
        format!("{}.{}", level, name)
    }

    // Shamelessly copied from
    // https://stackoverflow.com/questions/64517785/read-full-lines-from-stdin-including-n-until-end-of-file
    /// Read full lines of bufread iterator which doesn't chop new lines
    pub fn full_lines(mut input: impl BufRead) -> impl Iterator<Item = std::io::Result<String>> {
        std::iter::from_fn(move || {
            let mut vec = String::new();
            match input.read_line(&mut vec) {
                Ok(0) => None,
                Ok(_) => Some(Ok(vec)),
                Err(e) => Some(Err(e)),
            }
        })
    }

    /// Check if a character is a blank chracter
    pub(crate) fn is_blank_char(ch: char) -> bool {
        ch == ' ' || ch == '\t' || ch == '\n' || ch == '\r'
    }

    /// Check if a character is true
    ///
    /// In this contenxt, true and non zero number is 'true' while false and zero number is false
    pub(crate) fn is_arg_true(arg: &str) -> RadResult<bool> {
        let arg = arg.trim();
        if let Ok(value) = arg.parse::<usize>() {
            if value == 0 {
                return Ok(false);
            } else {
                return Ok(true);
            }
        } else if arg.to_lowercase() == "true" {
            return Ok(true);
        } else if arg.to_lowercase() == "false" {
            return Ok(false);
        }

        Err(RadError::InvalidArgument(
            "Neither true nor false".to_owned(),
        ))
    }

    /// Get a substring of utf8 encoded text.
    pub(crate) fn utf8_substring(source: &str, min: Option<usize>, max: Option<usize>) -> String {
        let mut result = String::new();
        if let Some(min) = min {
            if let Some(max) = max {
                // Both
                for (idx, ch) in source.chars().enumerate() {
                    if idx >= min && idx <= max {
                        result.push(ch);
                    }
                }
            } else {
                // no max
                for (idx, ch) in source.chars().enumerate() {
                    if idx >= min {
                        result.push(ch);
                    }
                }
            }
        } else {
            // No min
            if let Some(max) = max {
                // at least max
                for (idx, ch) in source.chars().enumerate() {
                    if idx <= max {
                        result.push(ch);
                    }
                }
            } else {
                // Nothing
                return source.to_owned();
            }
        }

        result
    }

    /// Print text as green if possible
    #[allow(unused_variables)]
    pub fn green(string: &str, to_file: bool) -> Box<dyn std::fmt::Display> {
        if cfg!(feature = "color") {
            #[cfg(feature = "color")]
            if !to_file {
                return Box::new(string.green());
            }
        }
        Box::new(string.to_owned())
    }

    /// Print text as red if possible
    #[allow(unused_variables)]
    pub fn red(string: &str, to_file: bool) -> Box<dyn std::fmt::Display> {
        if cfg!(feature = "color") {
            #[cfg(feature = "color")]
            if !to_file {
                return Box::new(string.red());
            }
        }
        Box::new(string.to_owned())
    }

    /// Print text as yellow if possible
    #[allow(unused_variables)]
    pub fn yellow(string: &str, to_file: bool) -> Box<dyn std::fmt::Display> {
        if cfg!(feature = "color") {
            #[cfg(feature = "color")]
            if !to_file {
                return Box::new(string.yellow());
            }
        }
        Box::new(string.to_owned())
    }

    // Copied from
    // https://llogiq.github.io/2016/09/24/newline.html
    // Actually the source talks about how to make following function faster
    // yet I don't want to use simd because r4d's logic is currently very synchronous
    // and making it a asynchornous would take much more effort and time
    // NOTE : Trailing single is necessary because this only checks newline chracter
    // thus line without trailing newline doesn't count as 1
    /// Count new lines
    #[allow(dead_code)]
    pub(crate) fn count_sentences(s: &str) -> usize {
        s.as_bytes().iter().filter(|&&c| c == b'\n').count() + 1
    }

    #[cfg(feature = "debug")]
    /// Clear terminal cells
    pub fn clear_terminal() -> RadResult<()> {
        use crossterm::{terminal::ClearType, ExecutableCommand};

        std::io::stdout()
            .execute(crossterm::terminal::Clear(ClearType::All))?
            .execute(crossterm::cursor::MoveTo(0, 0))?;

        Ok(())
    }

    /// Check if path is really in file system or not
    pub fn is_real_path(path: &std::path::Path) -> RadResult<()> {
        if !path.exists() {
            return Err(RadError::InvalidFile(path.display().to_string()));
        }
        Ok(())
    }

    /// Pop only a single newline from a source
    pub fn pop_newline(s: &mut String) {
        if s.ends_with('\n') {
            s.pop();
            if s.ends_with('\r') {
                s.pop();
            }
        }
    }

    /// Check file authority
    pub(crate) fn is_granted(
        name: &str,
        auth_type: AuthType,
        processor: &mut Processor,
    ) -> RadResult<bool> {
        match processor.get_auth_state(&auth_type) {
            AuthState::Restricted => Err(RadError::PermissionDenied(name.to_owned(), auth_type)),
            AuthState::Warn => {
                processor.log_warning(
                    &format!(
                        "\"{}\" was called with \"{:?}\" permission",
                        name, auth_type
                    ),
                    WarningType::Security,
                )?;
                Ok(true)
            }
            AuthState::Open => Ok(true),
        }
    }

    /// Execute a subprocess with given arguments
    #[cfg(feature = "basic")]
    pub(crate) fn subprocess(args: &[&str]) -> RadResult<()> {
        use std::io::Write;
        use std::process::Stdio;
        #[cfg(target_os = "windows")]
        let process = std::process::Command::new("cmd")
            .arg("/C")
            .args(&args[0..])
            .stdin(Stdio::piped())
            .spawn()
            .map_err(|_| {
                RadError::InvalidArgument(format!("Failed to execute command : \"{:?}\"", &args[0]))
            })?;

        #[cfg(not(target_os = "windows"))]
        let process = std::process::Command::new("sh")
            .arg("-c")
            .arg(&args[0..].join(" "))
            .stdin(Stdio::piped())
            .spawn()
            .map_err(|_| {
                RadError::InvalidArgument(format!("Failed to execute command : \"{:?}\"", &args[0]))
            })?;

        let output = process.wait_with_output()?;
        let out_content = String::from_utf8_lossy(&output.stdout);
        let err_content = String::from_utf8_lossy(&output.stderr);

        if out_content.len() != 0 {
            write!(std::io::stdout(), "{}", &out_content)?;
        }
        if err_content.len() != 0 {
            write!(std::io::stderr(), "{}", &err_content)?;
        }
        Ok(())
    }

    /// This checks if a file is safely modifiable
    ///
    /// File operation can be nested and somtimes logically implausible. Such as referencing self,
    /// or referencing parent file is would cause infinite loop
    ///
    /// Alos, opening processor's out option and err option would be impossible from the start,
    /// while creating a hard to read error message.
    ///
    /// Unallowed files are followed
    ///
    /// - Current input
    /// - Input that is saved in stack
    /// - File that is being relayed to
    /// - Processor's out option
    /// - Processor's err option
    ///
    /// # Argument
    ///
    /// - processor : Processor to get multiple files from
    /// - canoic    : Real absolute path to evaluate ( If not this possibly panicks )
    pub(crate) fn check_file_sanity(processor: &Processor, canonic: &Path) -> RadResult<()> {
        // Rule 1
        // You cannot include self
        if let ProcessInput::File(path) = &processor.state.current_input {
            if path.canonicalize()? == canonic {
                return Err(RadError::UnallowedMacroExecution(format!(
                    "Processing self is not allowed : \"{}\"",
                    &path.display()
                )));
            }
        }

        // Rule 2
        // Field is in input stack
        // This unwraps is mostly ok ( I guess )
        if processor.state.input_stack.contains(canonic) {
            return Err(RadError::UnallowedMacroExecution(format!(
                "Processing self is not allowed : \"{}\"",
                &canonic
                    .file_name()
                    .unwrap_or_else(|| OsStr::new("input_file"))
                    .to_string_lossy()
            )));
        }

        // Rule 3
        // You cannot include file that is being relayed
        if let Some(RelayTarget::File(file)) = &processor.state.relay.last() {
            if file.path() == canonic {
                return Err(RadError::UnallowedMacroExecution(format!(
                    "Processing relay target while relaying to the file is not allowed : \"{}\"",
                    &file.name().display()
                )));
            }
        }

        // Rule 4
        // You cannot include processor's out file
        if let WriteOption::File(target) = &processor.write_option {
            if target.path() == canonic {
                return Err(RadError::UnallowedMacroExecution(format!(
                    "Cannot process an out file : \"{}\"",
                    &target.name().display()
                )));
            }
        }

        // Rule 5
        // You cannot include processor's error file
        if let Some(WriteOption::File(target)) = &processor.get_logger_write_option() {
            if target.path() == canonic {
                return Err(RadError::UnallowedMacroExecution(format!(
                    "Cannot process an error file : \"{}\"",
                    &target.name().display()
                )));
            }
        }
        Ok(())
    }
}
