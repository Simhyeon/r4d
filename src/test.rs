#[test]
fn function_name_test() {
    println!(
        "{:?}",
        std::fs::canonicalize("../ced/docs/meta.md").unwrap()
    );
}

//#[test]
fn lul() -> RadResult<()> {
    use crate::AuthType;
    use crate::CommentType;
    use crate::DiffOption;
    use crate::HookType; // This is behind hook feature
    use crate::Hygiene;
    use crate::MacroType;
    use crate::Processor;
    use crate::RadResult;
    use crate::WarningType;
    use std::path::Path;

    // Builder
    let mut processor = Processor::new()
        .set_comment_type(CommentType::Start) // Use comment
        .custom_macro_char('~')? // use custom macro character
        .custom_comment_char('#')? // use custom comment character
        .purge(true) // Purge undefined macro
        .silent(WarningType::Security) // Silents all warnings
        .assert(true) // Enable assertion mode
        .lenient(true) // Disable strict mode
        .hygiene(Hygiene::Macro) // Enable hygiene mode
        .pipe_truncate(false) // Disable pipe truncate
        .write_to_file(Some(Path::new("out.txt")))? // default is stdout
        .error_to_file(Some(Path::new("err.txt")))? // default is stderr
        .unix_new_line(true) // use unix new line for formatting
        .discard(true) // discard all output
        .melt_files(vec![Path::new("source.r4d")])? // Read runtime macros from frozen
        // Permission
        .allow(Some(vec![AuthType::ENV])) // Grant permission of authtypes
        .allow_with_warning(Some(vec![AuthType::CMD])) // Grant permission of authypes with warning enabled
        // Debugging options
        .debug(true) // Turn on debug mode
        .log(true) // Use logging to terminal
        .diff(DiffOption::All)? // Print diff in final result
        .interactive(true); // Use interactive mode

    // Comment char and macro char cannot be same
    // Unallowed pattern for the characters are [a-zA-Z1-9\\_\*\^\|\(\)=,]

    // Use Processor::empty() instead of Processor::new()
    // if you don't want any default macros

    // Print information about current processor permissions
    // This is an warning and can be suppressed with silent option
    processor.print_permission()?;

    // Register a hook macro
    // Trigger and execution macro should be defined elsewhere
    processor.register_hook(
        HookType::Macro, // Macro type
        "trigger_macro", // Macro that triggers
        "hook_div",      // Macro to be executed
        1,               // target count
        false,           // Resetable
    )?;

    // Add runtime rules(in order of "name, args, body")
    processor.add_runtime_rules(vec![("test", "a_src a_link", "$a_src() -> $a_link()")])?;

    // Add custom rules without any arguments
    processor.add_static_rules(vec![("test", "TEST"), ("lul", "kekw")])?;

    // Undefine only macro
    processor.undefine_macro("name1", MacroType::Any);

    // Process with inputs
    // This prints to desginated write destinations
    processor.process_string(r#"$define(test=Test)"#)?;
    processor.process_stdin()?;
    processor.process_file(Path::new("from.txt"))?;

    processor.freeze_to_file(Path::new("out.r4f"))?; // Create frozen file

    // Print out result
    // This will print counts of warning and errors.
    // It will also print diff between source and processed if diff option was
    // given as builder pattern.
    processor.print_result()?;
    Ok(())
}
