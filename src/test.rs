use crate::utils::Utils;
use crate::{ArgParser, RadError, RadResult, RadStorage, StorageOutput, StorageResult};
use once_cell::sync::Lazy;
use regex::Regex;
use std::borrow::Cow;
use std::io::Write;

pub struct TestStorage;

/// Regex for leading and following spaces
static LF_SPACES: Lazy<Regex> = Lazy::new(|| Regex::new(r"(^[ \t\r\n]+)\S*([ \t\r\n]+$)").unwrap());
static LSPA: Lazy<Regex> = Lazy::new(|| Regex::new(r"(^[ \t\r\n]+)").unwrap());
static FSPA: Lazy<Regex> = Lazy::new(|| Regex::new(r"([ \t\r\n]+$)").unwrap());

// impl RadStorage for TestStorage {
//     fn update(&mut self, args: &[String]) -> crate::StorageResult<()> {
//         match args[0].as_str() {
//             "err" => return StorageResult::Err(Box::new(RadError::Interrupt)),
//             _ => return StorageResult::Ok(()),
//         }
//     }
//
//     fn extract(&mut self, serialize: bool) -> crate::StorageResult<Option<crate::StorageOutput>> {
//         StorageResult::Ok(None)
//     }
// }

#[test]
fn arg_test() {
    eprintln!(
        "{:#?}",
        Utils::get_whitespace_split_retain_quote_rule("a b ' c f g ' d")
    );
    let mut arg_parser = ArgParser::new();
    let result = arg_parser.args_to_vec("\\(,a", ',', crate::SplitVariant::Always);
    eprintln!("{result:#?}");
}
