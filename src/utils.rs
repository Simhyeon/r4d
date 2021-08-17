use crate::error::RadError;
use regex::Regex;

pub(crate) struct Utils;

impl Utils {
    pub(crate) fn args_to_vec<'a>(args : &'a str) -> Vec<&'a str> {
        args.split(",").collect()
    }

    pub(crate) fn args_with_len<'a>(args: &'a str, length: usize) -> Option<Vec<&'a str>> {
        let args: Vec<_> = args.split(",").collect();

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
    // TODO
    pub(crate) fn command_str(args: &str) -> Result<String, RadError> {
        // TODO 
        // parse string
        // Execute 
        unimplemented!();
        //std::process::Command(name)
            //.args(["", ""])
            //.output()
    }
}
