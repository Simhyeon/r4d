use crate::error::BasicError;
use regex::Regex;
use evalexpr::*;

pub fn regex(source: &str, target: &str, object: &str) -> Result<String, BasicError> {
    let reg = Regex::new(&format!(r"{}", target))?;
    let result = reg.replace_all(source, object); // This is a cow, moo~
    Ok(result.to_string())
}

pub fn cal(formula: &str) -> Result<String, BasicError> {
    let result = eval(formula)?;
    Ok(result.to_string())
}
