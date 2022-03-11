### Macro extension

You can extend macros easily with processor interface.

**Basic demo**

```rust
use rad::ExtMacroBuilder;
use rad::ext_template::*;

// Extend function macro
processor.add_ext_macro(ExtMacroBuilder::new("macro_name")
    .args(&vec!["a1","b2"])
    .function(function_template!(
        let args = split_args!(2)?;
        let result = format!("{} + {}", args[0], args[1]);
        Ok(Some(result))
)));

// Extend deterred macro
processor.add_ext_macro(ExtMacroBuilder::new("macro_name")
    .args(&vec!["a1","b2"])
    .deterred(deterred_template!(
        let args = split_args!(2)?;
        let result = if expand!(&args[0])? == "doit" {
            Some(format!("I did it -> {}", expand!(&args[1])?))
        } else { None };
        Ok(result)
)));
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