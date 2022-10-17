### Macro extension

You can extend macros easily with processor interface which is enabled by
**template** feature.

Refer last part of this documentation if you want to extend macros as binary
target.

**Basic demo**

```toml
[dependencies]
r4d = {version="*", features = ["template"]}
```

```rust
use r4d::ExtMacroBuilder;
use r4d::ext_template::*;
use r4d::AuthType;
use r4d::Processor;
use r4d::RadResult;

// ---
// Processor creation precedes...
// ---

// DEMOS
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
    ExtMacroBuilder::new("mac2")
        .args(&["a_first", "a_second"])
        .desc("Description comes here")
        .deterred(deterred_template!(
                // NOTE
                // Audit authentication
                // Macro name, and required auth type
                audit_auth!("mac2",AuthType::CMD);

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
```

**More about codes**

These macros are actually a convenience for trivial typings. Actual code under
the hood are expanded as closure.

```rust
// Original function macro's type
pub(crate) type FunctionMacroType = fn(&str, &mut Processor) -> RadResult<Option<String>>;

// function_template!( Your code ) expands to
|args: &str, processor: &mut Processor| -> RadResult<Option<String>> {
    Your code
}

// Original deterred macro's type
pub(crate) type DFunctionMacroType = fn(&str, usize, &mut Processor) -> RadResult<Option<String>>;

// deterred_template!( Your code ) expands to
|args: &str, level: usize, processor: &mut Processor| -> RadResult<Option<String>> {
    Your code
}

// split_args is  equivalent to
processor.split_arguments("text,to,parse", args, false)

// expand_args macro is equivalent to
processor.expand(level,"text_to_parse", true)

// expand_expr macro is equivalent to
processor.expand(level,"text_to_parse", false)
```

You can also simply send your function as argument instead of using template macros.

### Extend macros as binary with help of script.rs file

You can also extend rad macros by manually editing script.rs file inside src
directory.

[Script file](../src/script.rs) doesn't do anything by default and actually
doesn't get included without template feature.

You can modify the file and make it included by compiling with ```template```
feature.

```bash
# You need to work in a local copy of a project e.g) git clone
git clone https://github.com/simhyeon/r4d

# Build within r4d procject
cargo build --release --features binary,template

# Built binary is located in "target/release" directory
```

### Some general examples for deterred macro extension

By default, deterred macro's argument is not expanded at all. Therefore the
user has to manually set all expansion rules. This is cumbersome but useful
when you need a contextual information about macro expansion. R4d's default
"if" macro variants really represent such cases well.

You, as an end user can also profit by using hand written deterred macro when
you need to capture a context but don't need early expansion. For example,
```radroff``` which converts macro codes into a intermediate manual struct
simply passes raw text arguments into a data structure. While an expansion
occurs when the user decides a final format to print. With help of deterred
macro, r4d knows a context of a macro call and able to redirect raw information
to internal struct.
