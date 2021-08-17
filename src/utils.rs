use crate::error::RadError;
use regex::Regex;
use crate::consts::ESCAPE_CHAR;

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
        let reg = Regex::new(r"^[ \t\n]+|[ \t\n]+$")?;
        let result = reg.replace_all(args, "");

        Ok(result.to_string())
    }

    pub(crate) fn args_to_vec(arg_values: &str, delimiter: char, literal_rules: (char, char)) -> Vec<String> {
        let mut values = vec![];
        let mut value = String::new();
        let mut previous : Option<char> = None;
        let (lit_start, lit_end) = literal_rules;
        let mut literal = false;
        for ch in arg_values.chars() {
            if ch == delimiter && !literal {
                values.push(value);
                value = String::new();
            } 
            // Literal start
            else if ch == lit_start && previous.unwrap_or(' ') != ESCAPE_CHAR {
                literal = true;
            } 
            // Literal end
            else if ch == lit_end && previous.unwrap_or(' ') != ESCAPE_CHAR {
                literal = false;
            } 
            else {
                value.push(ch);
            }

            previous.replace(ch);
        }
        // Add last arg
        values.push(value);

        values
    }
}
