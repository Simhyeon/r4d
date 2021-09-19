use crate::error::RadError;
use regex::Regex;
use std::io::BufRead;

pub(crate) struct Utils;

impl Utils {
    /// Create local name
    pub(crate) fn local_name(level: usize, name : &str) -> String {
        format!("{}.{}", level, name)
    }

    /// Trim preceding and trailing whitespaces for given input
    ///
    /// # Arguments
    ///
    /// * `args` - Text to trim
    pub(crate) fn trim(args: &str) -> Result<String, RadError> {
        let reg = Regex::new(r"^[ \t\r\n]+|[ \t\r\n]+$")?;
        let result = reg.replace_all(args, "");

        Ok(result.to_string())
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
    pub(crate) fn is_arg_true(arg: &str) -> Result<bool, RadError> {
        let arg = Utils::trim(arg)?;
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
        return Err(RadError::InvalidArgument("Neither true nor false".to_owned()));
    }

    /// Get a substring of utf8 encoded text.
    pub(crate) fn utf8_substring(source: &str, min: Option<usize>, max: Option<usize>) -> String {
        let mut result = String::new();
        if let Some(min) = min {
            if let Some(max) = max { // Both
                for (idx,ch) in source.chars().enumerate() {
                    if idx >= min && idx <= max {
                        result.push(ch);
                    }
                }
            } else { // no max
                for (idx,ch) in source.chars().enumerate() {
                    if idx >= min {
                        result.push(ch);
                    }
                }
            }
        } else { // No min
            if let Some(max) = max { // at least max 
                for (idx,ch) in source.chars().enumerate() {
                    if idx <= max {
                        result.push(ch);
                    }
                }
            } else { // Nothing 
                return source.to_owned();
            }
        }
        return result;
    }
    
    // Copied from
    // https://llogiq.github.io/2016/09/24/newline.html
    // Actually the source talks about how to make following function faster
    // yet I don't want to use simd because r4d's logic is currently very synchronous
    // and making it a asynchornous would take much more effort and time
    // NOTE : Trailing single is necessary because this only checks newline chracter
    // thus line without trailing newline doesn't count as 1
    /// Count new lines
    pub(crate) fn count_sentences(s: &str) -> usize {
        s.as_bytes().iter().filter(|&&c| c == b'\n').count() + 1
    }

    #[cfg(feature = "debug")]
    /// Clear terminal cells
    pub fn clear_terminal() -> Result<(), RadError> {
        use crossterm::{ExecutableCommand, terminal::ClearType};

        std::io::stdout()
            .execute(crossterm::terminal::Clear(ClearType::All))?
            .execute(crossterm::cursor::MoveTo(0,0))?;

        Ok(())
    }
}
