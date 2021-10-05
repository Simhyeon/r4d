use std::array::IntoIter;
use std::iter::FromIterator;
use std::collections::HashMap;
use crate::RadError;
use crate::utils::Utils;
use crate::arg_parser::ArgParser;
use crate::Processor;

type KeywordMacType = fn(&str, usize,&mut Processor) -> Result<Option<String>, RadError>;

#[derive(Clone)]
pub struct KeywordMacro {
    macros : HashMap<String, KeywordMacType>,
}

impl KeywordMacro {
    pub fn new() -> Self {
        let map = HashMap::from_iter(IntoIter::new([
            ("pause".to_owned(),   KeywordMacro::pause            as KeywordMacType),
            ("foreach".to_owned(), KeywordMacro::foreach          as KeywordMacType),
            ("forloop".to_owned(), KeywordMacro::forloop          as KeywordMacType),
            ("if".to_owned(),      KeywordMacro::if_cond          as KeywordMacType),
            ("ifelse".to_owned(),  KeywordMacro::ifelse           as KeywordMacType),
        ]));
        Self {
            macros: map,
        }
    }

    /// Get Function pointer from map
    pub fn get(&self, name: &str) -> Option<&KeywordMacType> {
        if let Some(mac) = self.macros.get(name) {
            Some(mac)
        } else {
            None
        }
    }

    /// Check if map contains the name
    pub fn contains(&self, name: &str) -> bool {
        self.macros.contains_key(name)
    }

    // ----------
    // Keyword Macros start

    /// Pause every macro expansion
    ///
    /// Only other pause call is evaluated
    ///
    /// # Usage
    /// 
    /// $pause(true)
    /// $pause(false)
    fn pause(args: &str, level: usize,processor : &mut Processor) -> Result<Option<String>, RadError> {
        if let Some(args) = ArgParser::new().args_with_len(args, 1, true) {
            let arg = &processor.parse_chunk_args(level, "", &args[0])?;

            if let Ok(value) =Utils::is_arg_true(arg) {
                if value {
                    processor.paused = true;
                } else {
                    processor.paused = false;
                }
                Ok(None)
            } 
            // Failed to evaluate
            else {
                Err(RadError::InvalidArgument(format!("Pause requires either true/false or zero/nonzero integer, but given \"{}\"", arg)))
            }
        } else {
            Err(RadError::InvalidArgument("Pause requires an argument".to_owned()))
        }
    }

    /// Loop around given values and substitute iterators  with the value
    ///
    /// Foreach's second macro is evaluated twice.
    ///
    /// # Usage 
    ///
    /// $foreach(\*a,b,c*\,\*$:*\)
    fn foreach(args: &str, level: usize,processor: &mut Processor) -> Result<Option<String>, RadError> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2, true) {
            let mut sums = String::new();
            let loopable = &processor.parse_chunk_args(level, "", &args[0])?;

            for value in loopable.split(',') {
                let result = processor.parse_chunk_args(level, "", &args[1].replace("$:", value))?;
                sums.push_str(&result);
            }
            Ok(Some(sums))
        } else {
            Err(RadError::InvalidArgument("Foreach requires two argument".to_owned()))
        }
    }

    /// For loop around given min, max value and finally substitue iterators with value
    ///
    /// Forloop's second macro is evaluated twice.
    ///
    /// # Usage
    ///
    /// $forloop(1,5,\*$:*\)
    fn forloop(args: &str, level: usize, processor: &mut Processor) -> Result<Option<String>, RadError> {
        if let Some(args) = ArgParser::new().args_with_len(args, 3, true) {
            let mut sums = String::new();

            let min: usize; 
            let max: usize; 
            if let Ok(num) = Utils::trim(&args[0])?.parse::<usize>() {
                min = num;
            } else { return Err(RadError::InvalidArgument(format!("Forloop's min value should be non zero positive integer but given {}", &args[0]))); }
            if let Ok(num) = Utils::trim(&args[1])?.parse::<usize>() {
                max = num
            } else { return Err(RadError::InvalidArgument(format!("Forloop's min value should be non zero positive integer gut given \"{}\"", &args[1]))); }
            
            for value in min..=max {
                let result = processor.parse_chunk_args(level, "", &args[2].replace("$:", &value.to_string()))?;
                sums.push_str(&result);
            }

            Ok(Some(sums))
        } else {
            Err(RadError::InvalidArgument("Forloop requires two argument".to_owned()))
        }
    }

    /// Print content according to given condition
    /// 
    /// If macro's second argument is evaluated twice.
    ///
    /// # Usage 
    ///
    /// $if(evaluation, ifstate)
    fn if_cond(args: &str,level:usize,processor: &mut Processor) -> Result<Option<String>, RadError> {
        if let Some(args) = ArgParser::new().args_with_len(args, 2, true) {
            let boolean = &processor.parse_chunk_args(level,"",&args[0])?;

            // Given condition is true
            let cond = Utils::is_arg_true(&Utils::trim(boolean)?);
            if let Ok(cond) = cond {
                if cond { 
                    let if_expr = processor.parse_chunk_args(level,"",&args[1])?;
                    return Ok(Some(if_expr)); 
                }
            } else {
                return Err(RadError::InvalidArgument(format!("If requires either true/false or zero/nonzero integer but given \"{}\"", boolean)))
            }

            Ok(None)
        } else {
            Err(RadError::InvalidArgument("if requires two arguments".to_owned()))
        }
    }

    /// Print content according to given condition
    /// 
    /// Ifelse second and third arguemtns are evaluated twice.
    ///
    /// # Usage 
    ///
    /// $ifelse(evaluation, \*ifstate*\, \*elsestate*\)
    fn ifelse(args: &str, level: usize,processor: &mut Processor) -> Result<Option<String>, RadError> {
        if let Some(args) = ArgParser::new().args_with_len(args, 3, true) {
            let boolean = &processor.parse_chunk_args(level,"",&args[0])?;

            // Given condition is true
            let cond = Utils::is_arg_true(&Utils::trim(boolean)?);
            if let Ok(cond) = cond {
                if cond { 
                    let if_expr = processor.parse_chunk_args(level,"",&args[1])?;
                    return Ok(Some(if_expr)); 
                }
            } else {
                return Err(RadError::InvalidArgument(format!("Ifelse requires either true/false or zero/nonzero integer but given \"{}\"", boolean)))
            }

            // Else state
            let else_expr = processor.parse_chunk_args(level, "", &args[2])?;
            return Ok(Some(else_expr));
        } else {
            Err(RadError::InvalidArgument("ifelse requires three argument".to_owned()))
        }
    }

    // Keyword macros end
    // ----------
}
