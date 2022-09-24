use crate::RadResult;

#[test]
fn function_name_test() -> RadResult<()> {
    use crate::Processor;
    let processor = Processor::new();
    Ok(())
}
