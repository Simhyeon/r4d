### R4d (Rad)

R4d is a text oriented macro prosessor made with rust.

### NOTE

R4d is in very early stage, so there might be lots of undetected bugs. Fast
implementaiton was my priorites, thus optimization has had a least
consideration for the time.

**When it gets 1.0?**

R4d's will reach 1.0 only when followings are resolved.

- Debugger
- Consistent parenthesis rules
- Absence of critical bugs
- No possiblity of basic macro changes
- (maybe) Library feature to extend basic macros

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
rad -e FileToWriteError.txt # Log error to file
rad -s # Suppress error and warnings
rad -S # Strict mode makes every error panicking
rad -n # Always use unix newline (default is '\r\n' in windows platform)
rad -p # Purge mode, print nothing if a macro doesn't exist
rad -g # Always enable greedy for every macro invocation

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

# Individually available features are 
# "evalexpr", "chrono", "lipsum", "csv"

# evalexpr - "eval" macro
# chrono - "date", "time" macro
# lipsum - "lipsum" macro
# csv - "from", "table" macro
```
**rust file**
```rust
use rad::error::RadError;
use rad::processor::Processor

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
	.build(); // Create unreferenced instance

processor.from_string(r#"$define(test=Test)"#);
processor.from_stdin();
processor.from_file(&path);
processor.freeze_to_file(&path); // Create frozen file
processor.print_result(); // Print out warning and errors count
```

### Syntax 

[Macro syntax](./docs/macro_syntax.md)

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
