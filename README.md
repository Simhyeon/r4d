### R4d (Rad)

R4d is a text oriented macro prosessor made with rust.

### NOTE

R4d is in very early stage, so there might be lots of undetected bugs. Fast
implementaiton was my priorites, thus optimization has had a least
consideration for the time.

**When it gets 1.0?**

R4d's will reach 1.0 only when followings are resolved.

- Consistent parenthesis rules
- Absence of critical bugs
- No possiblity of basic macro changes

### Usage

**As a binary**

```bash
# Usage : rad [OPTIONS] [FILE]...

# Read from file and save to file
rad input_file.txt -o out_file.txt

# Read from file and print to stdout 
rad input_file.txt

# Read from standard input and print to file
printf '...text...' | rad -o out_file.txt

# Read from stdin and print to stdout 
printf '...text...' | rad 

# Use following options to decide error behaviours
# default is stderr
-e <FILE> # Log error to <FILE>
-s # Suppress error and warnings
-S # Strict mode makes every error panicking

# Use following options to decide deubbing behaviours
# default is not to debug
-d # Start debug mode
-l # Print all macro invocation logs
-i # Start debug mode as interactive, this makes stdout unwrapped

# Other flags
-n # Always use unix newline (default is '\r\n' in windows platform)
-p # Purge mode, print nothing if a macro doesn't exist
-g # Always enable greedy for every macro invocation

# Freeze(zip to binary) rules to a single file
rad test -f frozen.r4f
# Melt a file and use in processing
rad test -m frozen.r4f
```

Type ```-h``` or ```--help``` to see full options.

**As a library**

**Cargo.toml**
```toml
[dependencies]
rad = { version = "0.5", features = ["full"] }

# Other available features are 
# "evalexpr", "chrono", "lipsum", "csv", "debug"

# evalexpr - "eval" macro
# chrono - "date", "time" macro
# lipsum - "lipsum" macro
# csv - "from", "table" macro

# debug - Enable debug method
```
**rust file**
```rust
use rad::RadError;
use rad::Processor;
use rad::MacroType;

// Every option is not mendatory
let processor = Processor::new()
    .purge(true)
    .greedy(true)
    .silent(true)
    .strict(true)
    .custom_rules(Some(vec![pathbuf])) // Read from frozen rule files
    .write_to_file(Some(pathbuf))? // default is stdout
    .error_to_file(Some(pathbuf))? // default is stderr
    .unix_new_line(true) // use unix new line for formatting
    // Debugging options
    .debug(true) // Turn on debug mode
    .log(true) // Use logging to terminal
    .interactive(true) // Use interactive mode
    // Create unreferenced instance
    .build(); 

// Use Processor::empty() instead of Processor::new()
// if you don't want any default macros

// Add basic rules(= register functions)
processor.add_basic_rules(vec![("test", test as MacroType)]);

// Add custom rules(in order of "name, args, body") 
processor.add_custom_rules(vec![("test","a_src a_link","$a_src() -> $a_link()")]);

processor.from_string(r#"$define(test=Test)"#);
processor.from_stdin();
processor.from_file(&path);
processor.freeze_to_file(&path); // Create frozen file
processor.print_result(); // Print out warning and errors count
```

### Syntax 

[Macro syntax](./docs/macro_syntax.md)

### How to debug

[Debug](./docs/debug.md)

### Goal

R4d aims to be a modern alternative to m4 processor, which means

- No trivial m4 quotes for macro definition
- Explicit rule for macro definition and usage so that de facto underscore rule
is not necessary
- Easier binding with other programming languages(Rust's c binding)
- Enable combination of file stream and stdout
- As expressive as current m4 macro processor

#### Built-in macros (or macro-like functions)

[Usages](./docs/basic_macros.md)
