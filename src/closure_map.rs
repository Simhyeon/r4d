use std::collections::HashMap;
use crate::error::RadError;
use crate::RadResult;
use crate::arg_parser::ArgParser;

pub(crate) struct ClosureMap {
    map: HashMap<String, Box<dyn FnMut(&str,bool) -> RadResult<Option<String>>>>,
}

impl ClosureMap {
    /// Create new closure map
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }
    /// Check if map contains the name
    pub fn contains(&self, name: &str) -> bool {
        self.map.contains_key(name)
    }

    /// Add new closure
    pub fn add_new(&mut self, name: &'static str, arg_count: usize, mut closure : Box<dyn FnMut(Vec<String>) -> Option<String>>) 
    {
        self.map.insert(
            name.to_owned(), 
            Box::new(move |args: &str, greedy:bool| -> RadResult<Option<String>> {  
                if let Some(args) = ArgParser::new().args_with_len(args, arg_count, greedy) {
                    Ok(closure(args))
                } else {
                    Err(RadError::InvalidArgument(format!("Argument of \"{}\" is not sufficient.",name)))
                }
            })
        );
    }

    /// Execute closure by name
    pub fn call(&mut self, name: &str, args: &str, greedy: bool) -> RadResult<Option<String>> {
        if let Some(closure) = self.map.get_mut(name) {
            Ok(closure(args,greedy)?)
        } else {
            Ok(None)
        }
    }
}
