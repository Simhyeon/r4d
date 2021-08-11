use std::array::IntoIter;
use std::collections::HashMap;
use std::iter::FromIterator;
use crate::error::BasicError;
use regex::Regex;
use crate::utils;
use lipsum::lipsum;

type MacroType = fn(&str) -> Result<String, BasicError>;

pub struct BasicMacro<'a> {
    macros : HashMap<&'a str, MacroType>,
}

impl<'a> BasicMacro<'a> {
    pub fn new() -> Self {
        // Create hashmap of functions
        let map = HashMap::from_iter(IntoIter::new([
            ("test", BasicMacro::test as MacroType),
            ("regex_sub", BasicMacro::regex_sub as MacroType),
            ("regex_del", BasicMacro::regex_del as MacroType),
            ("eval", BasicMacro::eval as MacroType),
            ("trim", BasicMacro::trim as MacroType),
            ("chomp", BasicMacro::chomp as MacroType),
            ("compress", BasicMacro::compress as MacroType),
            ("placeholder", BasicMacro::placeholder as MacroType),
        ]));

        // Return struct
        Self {  macros : map }
    }

    pub fn call(&self, name : &str, args: &str) -> Result<bool, BasicError> {
        if let Some(func) = self.macros.get(name) {
            // Print out macro call result
            println!("{}", func(args)?);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn test(args: &str) -> Result<String, BasicError> {
        Ok(format!("Test call with args : {}", args))
    }

    pub fn regex_sub(args: &str) -> Result<String, BasicError> {
        if let Some(args) = utils::args_with_len(args, 3) {
            let source: &str = args[0];
            let target: &str = args[1];
            let object: &str = args[2];

            // This is regex expression without any preceding and trailing commands
            let reg = Regex::new(&format!(r"{}", target))?;
            let result = reg.replace_all(source, object); // This is a cow, moo~
            Ok(result.to_string())
        } else {
            Err(BasicError::InvalidArgument("Regex sub requires three arguments"))
        }
    }

    pub fn regex_del(args: &str) -> Result<String, BasicError> {
        if let Some(args) = utils::args_with_len(args, 2) {
            let source = args[0];
            let target = args[1];

            // This is regex expression without any preceding and trailing commands
            let reg = Regex::new(&format!(r"{}", target))?;
            let result = reg.replace_all(source, ""); // This is a cow, moo~, btw this replaces all match as empty character which technically deletes matches
            Ok(result.to_string())
        } else {
            Err(BasicError::InvalidArgument("Regex del requires two arguments"))
        }
    }

    pub fn eval(args: &str) -> Result<String, BasicError> {
        if let Some(args) = utils::args_with_len(args, 1) {
            let formula = args[0];
            let result = evalexpr::eval(formula)?;
            Ok(result.to_string())
        } else {
            Err(BasicError::InvalidArgument("Regex del requires an argument"))
        }
    }

    // Trim preceding and trailing whitespaces
    pub fn trim(args: &str) -> Result<String, BasicError> {
        if let Some(args) = utils::args_with_len(args, 1) {
            let source = args[0];
            // let reg = Regex::new(r"^[ \t]+")?;
            let reg = Regex::new(r"^[ \t\n]+|[ \t\n]+$")?;
            let result = reg.replace_all(source, "");

            Ok(result.to_string())
        } else {
            Err(BasicError::InvalidArgument("Trim requires an argument"))
        }
    }

    // Remove duplicate newlines
    pub fn chomp(args: &str) -> Result<String, BasicError> {
        if let Some(args) = utils::args_with_len(args, 1) {
            let source = args[0];
            let reg = Regex::new(r"\n\s*\n")?;
            let result = reg.replace_all(source, "\n\n");

            Ok(result.to_string())
        } else {
            Err(BasicError::InvalidArgument("Chomp requires an argument"))
        }
    }

    pub fn compress(args: &str) -> Result<String, BasicError> {
        if let Some(args) = utils::args_with_len(args, 1) {
            let source = args[0];
            // Chomp and then compress
            let result = BasicMacro::trim(&BasicMacro::chomp(source)?)?;

            Ok(result.to_string())
        } else {
            Err(BasicError::InvalidArgument("Compress requires an argument"))
        }
    }

    pub fn placeholder(args: &str) -> Result<String, BasicError> {
        if let Some(args) = utils::args_with_len(args, 1) {
            let word_count = args[0];
            Ok(lipsum(word_count.parse::<usize>()?))
        } else {
            Err(BasicError::InvalidArgument("Placeholder requires an argument"))
        }
    }

    // TODO
    pub fn include(args: &str) -> Result<String, BasicError> {
        if let Some(args) = utils::args_with_len(args, 1) {
            let file_path = args[0];

            unimplemented!();
            // TODO
            // Include should invoke macro in its internal content first
        } else {
            Err(BasicError::InvalidArgument("Include requires an argument"))
        }
    }

    // TODO 
    // IF
    // IfElse
    // IfDefine
}

