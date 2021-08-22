use crate::error::RadError;
use regex::Regex;
use std::io::BufRead;
use lazy_static::lazy_static;

lazy_static!{
   pub static ref COMMA: Regex = Regex::new(r",").unwrap();
   pub static ref DQUOTE: Regex = Regex::new(r#"""#).unwrap();
   pub static ref LEFT_PAREN: Regex = Regex::new(r"\(").unwrap();
   pub static ref RIGHT_PAREN: Regex = Regex::new(r"\)").unwrap();
}

pub(crate) struct Utils;

impl Utils {
    pub(crate) fn local_name(level: usize, caller: &str, name : &str) -> String {
        format!("{}.{}.{}", level,caller, name)
    }

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

    pub(crate) fn is_blank_char(ch: char) -> bool {
        ch == ' ' || ch == '\t' || ch == '\n' || ch == '\r'
    }

    pub(crate) fn escape_all(args: &str) ->  Result<String, RadError> {
        let cow1 = COMMA.replace_all(args, "\\,");
        let cow2 = DQUOTE.replace_all(cow1.as_ref(), "\\\"");
        let cow3 = LEFT_PAREN.replace_all(cow2.as_ref(), "\\(");
        let cow4 = RIGHT_PAREN.replace_all(cow3.as_ref(), "\\)");
        Ok(cow4.to_string())
    }

    pub(crate) fn is_arg_true(arg: &str) -> Result<bool, RadError> {
        let arg = Utils::trim(arg)?;
        if let Ok(value) = arg.parse::<usize>() {
            if value == 0 {
                return Ok(false);
            } else {
                return Ok(true);
            }
        } else {
            if arg == "true" {
                return Ok(true);
            } else if arg == "false" {
                return Ok(false);
            }
        }
        return Err(RadError::InvalidArgument("Neither true nor false"));
    }
}
