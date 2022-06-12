use crate::auth::{AuthState, AuthType};
use crate::error::RadError;
use crate::logger::WarningType;
use crate::models::RadResult;
use crate::Processor;
use lazy_static::lazy_static;
use regex::Regex;
use std::io::BufRead;
use std::io::Write;
use std::process::Stdio;

lazy_static! {
    pub static ref TRIM: Regex = Regex::new(r"^[ \t\r\n]+|[ \t\r\n]+$").unwrap();
}

#[cfg(feature = "color")]
use colored::*;

pub(crate) struct Utils;

impl Utils {
    /// Create local name
    pub(crate) fn local_name(level: usize, name: &str) -> String {
        format!("{}.{}", level, name)
    }

    /// Trim preceding and trailing whitespaces for given input
    ///
    /// # Arguments
    ///
    /// * `args` - Text to trim
    pub(crate) fn trim(args: &str) -> String {
        let result = TRIM.replace_all(args, "");

        result.to_string()
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

    /// Check if a character is blank
    pub(crate) fn is_blank_char(ch: char) -> bool {
        ch == ' ' || ch == '\t' || ch == '\n' || ch == '\r'
    }

    /// Check if a character is true
    ///
    /// In this contenxt, true and non zero number is 'true' while false and zero number is false
    pub(crate) fn is_arg_true(arg: &str) -> RadResult<bool> {
        let arg = Utils::trim(arg);
        if let Ok(value) = arg.parse::<usize>() {
            if value == 0 {
                return Ok(false);
            } else {
                return Ok(true);
            }
        } else {
            if arg.to_lowercase() == "true" {
                return Ok(true);
            } else if arg.to_lowercase() == "false" {
                return Ok(false);
            }
        }
        return Err(RadError::InvalidArgument(
            "Neither true nor false".to_owned(),
        ));
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
        return result;
    }

    pub fn green(string: &str) -> Box<dyn std::fmt::Display> {
        if cfg!(feature = "color") {
            #[cfg(feature = "color")]
            return Box::new(string.green().to_owned());
        }
        Box::new(string.to_owned())
    }

    pub fn red(string: &str) -> Box<dyn std::fmt::Display> {
        if cfg!(feature = "color") {
            #[cfg(feature = "color")]
            return Box::new(string.red().to_owned());
        }
        Box::new(string.to_owned())
    }

    pub fn yellow(string: &str) -> Box<dyn std::fmt::Display> {
        if cfg!(feature = "color") {
            #[cfg(feature = "color")]
            return Box::new(string.yellow().to_owned());
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

    pub(crate) fn subprocess(args: &Vec<&str>) -> RadResult<()> {
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
            .arg(&args[0..].join(" ")) // TODO, is this correct?
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
}
