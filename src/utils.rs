use crate::error::RadError;
use regex::Regex;
use crate::consts::ESCAPE_CHAR;
use std::io::BufRead;

pub(crate) struct Utils;

impl Utils {
    pub(crate) fn args_with_len<'a>(args: &'a str, length: usize) -> Option<Vec<String>> {
        let args: Vec<_> = Utils::args_to_vec(args, ',', ('"', '"'));

        if args.len() < length {
            return None;
        } 

        Some(args)
    }

    pub(crate) fn local_name(level: usize, caller: &str, name : &str) -> String {
        format!("{}.{}.{}", level,caller, name)
    }

    pub(crate) fn trim(args: &str) -> Result<String, RadError> {
        let reg = Regex::new(r"^[ \t\r\n]+|[ \t\r\n]+$")?;
        let result = reg.replace_all(args, "");

        Ok(result.to_string())
    }

    pub(crate) fn args_to_vec(arg_values: &str, delimiter: char, literal_rules: (char, char)) -> Vec<String> {
        let mut values = vec![];
        let mut value = String::new();
        let mut previous : Option<char> = None;
        let (lit_start, lit_end) = literal_rules;
        let mut literal = false;
        let mut no_previous = false;
        for ch in arg_values.chars() {
            if ch == delimiter  {
                if literal || previous.unwrap_or('0') == ESCAPE_CHAR { value.push(ch); } 
                else {
                    println!("DELMITED");
                    values.push(value);
                    value = String::new();
                }
            } else if ch == lit_start {
                // Not escaped
                if previous.unwrap_or('0') != ESCAPE_CHAR {
                    if lit_start == lit_end { literal = !literal; } 
                    else { literal = true; }
                } 
                // Escaped
                else { value.push(ch); }
            } else if ch == lit_end {
                // Not escaped
                if previous.unwrap_or('0') != ESCAPE_CHAR {
                    literal = false;
                } 
                // Escaped
                else { value.push(ch); }
            } else if ch == ESCAPE_CHAR {
                // Previous was escape, then add
                if previous.unwrap_or('0') == ESCAPE_CHAR {
                    value.push(ch);
                    // Current escape is consumed and doesn't affect next character
                    no_previous = true;
                } 
            } else { value.push(ch) }
            if no_previous {
                previous.replace('0');
                no_previous = false;
            } else {
                previous.replace(ch);
            }
        }
        // Add last arg
        values.push(value);

        values
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
}
