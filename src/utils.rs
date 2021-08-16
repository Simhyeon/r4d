use crate::error::RadError;
use regex::Regex;

pub(crate) struct Utils;

impl Utils {
    pub(crate) fn args_to_vec<'a>(args : &'a str) -> Vec<&'a str> {
        args.split(",").collect()
    }

    pub(crate) fn args_with_len<'a>(args: &'a str, length: usize) -> Option<Vec<&'a str>> {
        let args: Vec<_> = args.split(",").collect();

        if args.len() != length {
            return None;
        } 

        Some(args)
    }

    pub(crate) fn local_name(caller: &str, name : &str) -> String {
        format!("{}.{}", caller, name)
    }

    pub(crate) fn trim(args: &str) -> Result<String, RadError> {
        if let Some(args) = Utils::args_with_len(args, 1) {
            let source = args[0];
            // let reg = Regex::new(r"^[ \t]+")?;
            let reg = Regex::new(r"^[ \t\n]+|[ \t\n]+$")?;
            let result = reg.replace_all(source, "");

            Ok(result.to_string())
        } else {
            Err(RadError::InvalidArgument("Trim requires an argument"))
        }
    }
}
