use std::array::IntoIter;
use std::collections::HashMap;
use std::iter::FromIterator;
use crate::error::RadError;
use crate::consts::MAIN_CALLER;
use regex::Regex;
use crate::utils::Utils;
use crate::processor::Processor;
use lipsum::lipsum;

type MacroType = fn(&str) -> Result<String, RadError>;

pub struct BasicMacro<'a> {
    macros : HashMap<&'a str, MacroType>,
}

impl<'a> BasicMacro<'a> {
    pub fn new() -> Self {
        // Create hashmap of functions
        let map = HashMap::from_iter(IntoIter::new([
            ("regex_sub", BasicMacro::regex_sub as MacroType),
            ("regex_del", BasicMacro::regex_del as MacroType),
            ("eval", BasicMacro::eval as MacroType),
            ("trim", BasicMacro::trim as MacroType),
            ("chomp", BasicMacro::chomp as MacroType),
            ("compress", BasicMacro::compress as MacroType),
            ("placeholder", BasicMacro::placeholder as MacroType),
            ("time", BasicMacro::time as MacroType),
            ("date", BasicMacro::date as MacroType),
            ("include", BasicMacro::include as MacroType),
            ("repeat", BasicMacro::repeat as MacroType)
        ]));

        // Return struct
        Self {  macros : map }
    }

    pub fn contains(&self, name: &str) -> bool {
        self.macros.contains_key(name)
    }

    pub fn call(&self, name : &str, args: &str) -> Result<String, RadError> {
        if let Some(func) = self.macros.get(name) {
            // Print out macro call result
            let result = func(args)?;
            Ok(result)
        } else {
            Ok(String::new())
        }
    }

    fn time(_: &str) -> Result<String, RadError> {
        Ok(format!("{}", chrono::offset::Local::now().format("%H:%M:%S")))
    }

    fn date(_: &str) -> Result<String, RadError> {
        Ok(format!("{}", chrono::offset::Local::now().format("%Y-%m-%d")))
    }

    fn regex_sub(args: &str) -> Result<String, RadError> {
        if let Some(args) = Utils::args_with_len(args, 3) {
            let source: &str = args[0];
            let target: &str = args[1];
            let object: &str = args[2];

            // This is regex expression without any preceding and trailing commands
            let reg = Regex::new(&format!(r"{}", target))?;
            let result = reg.replace_all(source, object); // This is a cow, moo~
            Ok(result.to_string())
        } else {
            Err(RadError::InvalidArgument("Regex sub requires three arguments"))
        }
    }

    fn regex_del(args: &str) -> Result<String, RadError> {
        if let Some(args) = Utils::args_with_len(args, 2) {
            let source = args[0];
            let target = args[1];

            // This is regex expression without any preceding and trailing commands
            let reg = Regex::new(&format!(r"{}", target))?;
            let result = reg.replace_all(source, ""); // This is a cow, moo~, btw this replaces all match as empty character which technically deletes matches
            Ok(result.to_string())
        } else {
            Err(RadError::InvalidArgument("Regex del requires two arguments"))
        }
    }

    fn eval(args: &str) -> Result<String, RadError> {
        if let Some(args) = Utils::args_with_len(args, 1) {
            let formula = args[0];
            let result = evalexpr::eval(formula)?;
            Ok(result.to_string())
        } else {
            Err(RadError::InvalidArgument("Regex del requires an argument"))
        }
    }

    // Trim preceding and trailing whitespaces
    fn trim(args: &str) -> Result<String, RadError> {
        Utils::trim(args)
    }

    // Remove duplicate newlines
    fn chomp(args: &str) -> Result<String, RadError> {
        if let Some(args) = Utils::args_with_len(args, 1) {
            let source = args[0];
            let reg = Regex::new(r"\n\s*\n")?;
            let result = reg.replace_all(source, "\n\n");

            Ok(result.to_string())
        } else {
            Err(RadError::InvalidArgument("Chomp requires an argument"))
        }
    }

    fn compress(args: &str) -> Result<String, RadError> {
        if let Some(args) = Utils::args_with_len(args, 1) {
            let source = args[0];
            // Chomp and then compress
            let result = BasicMacro::trim(&BasicMacro::chomp(source)?)?;

            Ok(result.to_string())
        } else {
            Err(RadError::InvalidArgument("Compress requires an argument"))
        }
    }

    fn placeholder(args: &str) -> Result<String, RadError> {
        if let Some(args) = Utils::args_with_len(args, 1) {
            let word_count = args[0];
            Ok(lipsum(word_count.parse::<usize>()?))
        } else {
            Err(RadError::InvalidArgument("Placeholder requires an argument"))
        }
    }

    fn include(args: &str) -> Result<String, RadError> {
        if let Some(args) = Utils::args_with_len(args, 1) {
            let file_path = args[0];
            let file_path = std::path::Path::new(file_path);
            Ok(Processor::new().from_file(file_path)?)
        } else {
            Err(RadError::InvalidArgument("Include requires an argument"))
        }
    }

    // TODO
    fn repeat(args: &str) -> Result<String, RadError> {
        if let Some(args) = Utils::args_with_len(args, 2) {
            let repeat_count =  args[0].parse::<usize>()?;
            let repeat_object = args[1];
            let mut repeated = String::new();
            let item = Processor::new().parse_chunk(0, &MAIN_CALLER.to_owned(), &mut repeat_object.lines())?;
            for _ in 0..repeat_count {
                repeated.push_str(&item);
            }
            Ok(repeated)
        } else {
            Err(RadError::InvalidArgument("Include requires an argument"))
        }
    }

    // TODO 
    // IF
    // IfElse
    // IfDefine
    // Repeat
    // Syscmd
}

