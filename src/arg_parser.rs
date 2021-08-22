use crate::consts::{ESCAPE_CHAR, LIT_CHAR};
pub struct ArgParser;

impl ArgParser {
    pub(crate) fn args_with_len<'a>(args: &'a str, length: usize) -> Option<Vec<String>> {
        let args: Vec<_> = ArgParser::args_to_vec(args, ',');

        if args.len() < length {
            return None;
        } 

        Some(args)
    }

    pub(crate) fn args_to_vec(arg_values: &str, delimiter: char) -> Vec<String> {
        let mut values = vec![];
        let mut value = String::new();
        let mut previous : Option<char> = None;
        let mut lit_count : usize = 0;
        let mut no_previous = false;
        let mut arg_iter = arg_values.chars().peekable();

        while let Some(ch) = arg_iter.next() {
            if ch == delimiter {
                // Either literal or escaped
                if lit_count > 0 
                    || previous.unwrap_or('0') == ESCAPE_CHAR 
                { 
                    value.push(ch); 
                } 
                // else move to next value
                else {
                    values.push(value);
                    value = String::new();
                }
            }
            // Default behaviour of escape_char is not adding
            else if ch == ESCAPE_CHAR { 
                // If literal print everything without escaping
                if lit_count > 0 {
                    value.push(ch);
                }
                // Previous was escape, then add
                else if previous.unwrap_or('0') == ESCAPE_CHAR  {
                    value.push(ch);
                    // Current escape is consumed and doesn't affect next character
                    no_previous = true;
                } 
            }
            else { // This pushes value in the end, so use continue not push the value
                if ch == LIT_CHAR { // '*'
                    if previous.unwrap_or('0') == ESCAPE_CHAR {
                        lit_count = lit_count + 1;
                        // If lit character was given inside literal
                        // e.g. \* '\*' *\ -> the one inside quotes
                        if lit_count > 1 {
                            value.push(ch);
                        } 
                        // First lit character in given args
                        // Simply ignore character and don't set previous
                        else { 
                            previous.replace('0');
                            continue; 
                        }
                    } else if let Some(&ch) = arg_iter.peek() {
                        // Next is escape chart and not inside lit_count
                        // *\
                        if ch == ESCAPE_CHAR && lit_count >= 1 {
                            lit_count = lit_count - 1; 
                            arg_iter.next(); // Conume next escape_char
                            // Lit end was outter most one
                            if lit_count == 0 {
                                previous.replace('0');
                                continue;
                            } 
                            // Inside other literal rules
                            else {
                                value.push(LIT_CHAR);
                                value.push(ESCAPE_CHAR);
                                no_previous = true;
                            }
                        } 
                        // When *\ Comes first without matching pair
                        // This is just a string without any meaning
                        else {
                            value.push(ch);
                        }
                    } 
                    // Meaningless literal charcter are just pushed
                    else {
                        value.push(ch);
                    }
                } 
                // Non literal character are just pushed
                else {
                    value.push(ch);
                }
            }

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
}
