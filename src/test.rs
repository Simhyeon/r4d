use crate::{RadError, RadResult, RadStorage, StorageOutput, StorageResult};
use std::io::Write;

pub struct TestStorage;

impl RadStorage for TestStorage {
    fn update(&mut self, args: &[String]) -> crate::StorageResult<()> {
        match args[0].as_str() {
            "err" => return StorageResult::Err(Box::new(RadError::Interrupt)),
            _ => return StorageResult::Ok(()),
        }
    }

    fn extract(&mut self, serialize: bool) -> crate::StorageResult<Option<crate::StorageOutput>> {
        StorageResult::Ok(None)
    }
}

#[test]
fn function_name_test() -> RadResult<()> {
    // use crate::Processor;
    // let mut processor = Processor::new();
    // processor.add_static_rules(&[("test", "")])?;
    // writeln!(std::io::stdout(), "{}", processor.get_static("test")?);
    // processor.replace_macro("test", "WOWZER");
    let chs = ['안', 'a', 'ي', '𝅘𝅥𝅰']; // let chs = ['a', 'b', 'c', 'd'];
    for ch in chs {
        eprintln!("CH : \"{ch}\" -- bytes {}", ch.len_utf8())
    }
    Ok(())
}
