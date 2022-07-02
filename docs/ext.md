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

// ---
// Processor creation precedes...
// ---

// Extend function macro
processor.add_ext_macro(ExtMacroBuilder::new("macro_name")
    .args(&["a1","b2"])
    .function(function_template!(
        let args = split_args!(2)?;
        let result = format!("{} + {}", args[0], args[1]);
        Ok(Some(result))
)));

// Extend deterred macro
processor.add_ext_macro(ExtMacroBuilder::new("macro_name")
    .args(&["a1","b2"])
    .deterred(deterred_template!(
        let args = split_args!(2)?;
        let result = if expand!(&args[0])? == "doit" {
            Some(format!("I did it -> {}", expand!(&args[1])?))
        } else { None };
        Ok(result)
)));

/// Optionally use audit_auth inside template macro for auth requirment.
/// This will return error if given auth is not allowed
audit_auth!("macro_name",AuthType::CMD);
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

// split_args are equivalent to
processor.get_split_arguments("text,to,parse", args)

// expand are equivalent to
processor.expand(level,"text_to_parse")
```

You can also simply send your function as argument instead of using template macros.

### Extend macros as binary with help of script.rs file

You can also extend rad macros by manually editing script.rs file and compiling
with ```template``` feature.

[Script file](../src/script.rs) doesn't do anything by default and actually
doesn't get included.

You can modify the file and make it included by compiling with ```template``` feature.

```bash
# You need to work in a local copy of a project e.g) git clone
git clone https://github.com/simhyeon/r4d

# Build within r4d procject
cargo build --release --features binary,color,template

# Built binary is located in "target/release" directory
```
