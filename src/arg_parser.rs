use crate::consts::{ESCAPE_CHAR, LIT_CHAR};
pub struct ArgParser;

#[derive(Debug)]
pub enum GreedyState {
    Reserve(usize),
    None,
    Never,
}

impl ArgParser {
    pub(crate) fn args_with_len<'a>(args: &'a str, length: usize, greedy: bool) -> Option<Vec<String>> {
        let greedy_state = if greedy { 
            if length > 1 {
                GreedyState::Reserve(length - 1)
            } else {
                GreedyState::None
            }
        } else { 
            GreedyState::Never
        };
        let args: Vec<_> = ArgParser::args_to_vec(args, ',', greedy_state);

        if args.len() < length {
            return None;
        } 

        Some(args)
    }

    pub(crate) fn args_to_vec(arg_values: &str, delimiter: char, mut greedy_state: GreedyState) -> Vec<String> {
        let mut values = vec![];
        let mut value = String::new();
        let mut previous : Option<char> = None;
        let mut lit_count : usize = 0;
        let mut no_previous = false;
        let mut arg_iter = arg_values.chars().peekable();

        while let Some(ch) = arg_iter.next() {
            // If greedy 
            if ch == delimiter {
                // Either literal or escaped
                if lit_count > 0 
                    || previous.unwrap_or('0') == ESCAPE_CHAR 
                { 
                    value.push(ch); 
                } else { // not literal
                    match greedy_state {
                        GreedyState::Reserve(count) => {
                            // move to next value
                            values.push(value);
                            value = String::new();
                            let count = count - 1;
                            if count > 0 {
                                greedy_state = GreedyState::Reserve(count);
                            } else {
                                greedy_state = GreedyState::None;
                            }
                            continue;
                        }
                        // Push everything to current item, index, value or you name it
                        GreedyState::None => {
                            value.push(ch);
                            continue;
                        }
                        GreedyState::Never => {
                            // move to next value
                            values.push(value);
                            value = String::new();
                        }
                    } // Match end
                } // else end
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
                    } else if let Some(&ch_next) = arg_iter.peek() {
                        // Next is escape chart and not inside lit_count
                        // *\
                        if ch_next == ESCAPE_CHAR && lit_count >= 1 {
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
