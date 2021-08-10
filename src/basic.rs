use std::array::IntoIter;
use std::collections::HashMap;
use std::iter::FromIterator;
use crate::error::BasicError;
use regex::Regex;

type MacroType = fn(String) -> Result<String, BasicError>;

pub struct BasicMacro<'a> {
    macros : HashMap<&'a str, MacroType>,
}

impl<'a> BasicMacro<'a> {
    pub fn new() -> Self {
        // Create hashmap of functions
        let map = HashMap::from_iter(IntoIter::new([
            ("test", BasicMacro::test as MacroType)
            //("regex_sub", BasicMacros::regex_sub)
        ]));

        // Return struct
        Self {  macros : map }
    }

    pub fn call(&self, name : &str, args: &str) -> Result<bool, BasicError> {
        if let Some(func) = self.macros.get(name) {
            // Print out macro call result
            println!("{}", func(args.to_owned())?);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn test(args: String) -> Result<String, BasicError> {
        Ok(format!("Test call with args : {}", args))
    }

    pub fn regex_sub(source: &str, target: &str, object: &str) -> Result<String, BasicError> {
        // This is regex expression without any preceding and trailing commands
        let reg = Regex::new(&format!(r"{}", target))?;
        let result = reg.replace_all(source, object); // This is a cow, moo~
        Ok(result.to_string())
    }

    pub fn regex_del(source: &str, target: &str) -> Result<String, BasicError> {
        // This is regex expression without any preceding and trailing commands
        let reg = Regex::new(&format!(r"{}", target))?;
        let result = reg.replace_all(source, ""); // This is a cow, moo~, btw this replaces all match as empty character which technically deletes matches
        Ok(result.to_string())
    }

    pub fn eval(formula: &str) -> Result<String, BasicError> {
        let result = evalexpr::eval(formula)?;
        Ok(result.to_string())
    }

    // TODO 
    // IF
    // IfElse
    // IfDefine
}

