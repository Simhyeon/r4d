use crate::argument::{MacroInput, ValueType};
use crate::common::MacroAttribute;
use crate::function_map::FMacroSign;
use crate::utils::{RadStr, Utils};
use crate::{ArgParser, Processor, RadResult};

fn yo(i: MacroInput, p: &mut Processor) -> RadResult<Option<String>> {
    Ok(None)
}
#[test]
fn test() {
    let test = FMacroSign::new("", [(ValueType::Text, "d")], yo, Some("".to_string()));
}
