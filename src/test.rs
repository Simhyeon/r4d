use crate::{RadError, RadResult, RadStorage, StorageOutput, StorageResult};

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
    use crate::Processor;
    let mut processor = Processor::new();
    processor.set_storage(Box::new(TestStorage {}));
    let result = processor.update_storage(&["err".to_string()]);
    if let Err(err) = result {
        eprintln!("{}", err);
    }
    Ok(())
}
