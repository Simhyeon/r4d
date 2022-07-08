// NOTE
// Leave imports as it is if you are not sure about it

#[allow(unused_imports)]
use crate::rad_ext_template::*;
#[allow(unused_imports)]
use crate::AuthType;
#[allow(unused_imports)]
use crate::ExtMacroBuilder;
use crate::Processor;
use crate::RadResult;

/// Extend a processor with user configured extension macros
///
/// Refer https://github.com/Simhyeon/r4d/blob/master/docs/ext.md for detailed explanation about
/// macro extensions
#[allow(unused_variables)]
pub fn extend_processor(processor: &mut Processor) -> RadResult<()> {
    // ---
    // Write your custom extension macros from here

    // NOTE
    // Remove surrounding quote /* */ to uncomment

    /*
    // Extend function macro
    processor.add_ext_macro(
        ExtMacroBuilder::new("mac1")
            .args(&["a_first", "a_second"])
            .desc("Description comes here")
            .function(function_template!(
                    // NOTE
                    // Function macro does not need to expand arguments
                    // Because it is already expanded

                    // NOTE
                    // On split_args macro,
                    // Function macro should give second argument as "false"
                    // If true, then it will not strip literal quotes
                    let args = split_args!(2,false)?;
                    let result = format!("{} + {}", args[0], args[1]);
                    Ok(Some(result))
            )),
    );
    // Extend deterred macro
    processor.add_ext_macro(
        ExtMacroBuilder::new("wow")
            .args(&["a_first", "a_second"])
            .desc("Description comes here")
            .deterred(deterred_template!(
                    // NOTE
                    // Audit authentication
                    // Macro name, and required auth type
                    audit_auth!("macro_name",AuthType::CMD);

                    // NOTE
                    // On split_args macro,
                    // Deterred macro should have second argument as "true"
                    // because deterred should not strip before expansion
                    let args = split_args!(2,true)?;

                    // NOTE
                    // expand_args macro should be only called on deterred macros
                    // because it expands arguments and also strip
                    let result = if expand_args!(&args[0])? == "doit" {

                        // NOTE
                        // You can expand normal expression with expand_args macro
                        Some(format!("I did it -> {}", expand_args!(&args[1])?))
                    } else { None };
                    Ok(result)
            )),
    );
    */

    // NOTE
    // Should return Ok(()) at the end
    // ---
    Ok(())
}
