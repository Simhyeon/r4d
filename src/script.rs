#[allow(unused_imports)]
use crate::rad_ext_template::*;
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

    /*
    // DEMOS
    // Extend function macro
    processor.add_ext_macro(
        ExtMacroBuilder::new("macro_name")
            .args(&["a1", "b2"])
            .function(function_template!(
                    let args = split_args!(2)?;
                    let result = format!("{} + {}", args[0], args[1]);
                    Ok(Some(result))
            )),
    );
    // Extend deterred macro
    processor.add_ext_macro(
        ExtMacroBuilder::new("macro_name")
            .args(&["a1", "b2"])
            .deterred(deterred_template!(
                    audit_auth!("macro_name",crate::AuthType::CMD);
                    let args = split_args!(2)?;
                    let result = if expand!(&args[0])? == "doit" {
                        Some(format!("I did it -> {}", expand!(&args[1])?))
                    } else { None };
                    Ok(result)
            )),
    );
    */

    // Should return Ok(()) at the end
    // ---
    Ok(())
}
