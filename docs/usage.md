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

# Some macros need permission to process
# use following options to grant permission
-a env      # Give environment permission
-a cmd      # Give syscmd permission
-a fin+fout # give both file read and file write permission
-A          # Give all permission. this is same with '-a env+cmd+fin+fout'
-w env      # Give permission but warn when macro is used
-W          # Same with '-A' but for warning

# Use following options to decide error behaviours
# default is stderr
-e <FILE>   # Log error to <FILE>
-s          # Suppress error and warnings
-S          # Strict mode makes every error panicking

# Use following options to decide deubbing behaviours
# default is not to debug
-d          # Start debug mode
-l          # Print all macro invocation logs
-i          # Start debug mode as interactive, this makes stdout unwrapped

# Other flags
-n          # Always use unix newline (default is '\r\n' in windows platform)
-p          # Purge mode, print nothing if a macro doesn't exist
-g          # Always enable greedy for every macro invocation
--discard   # Discard all output

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
# "evalexpr", "chrono", "lipsum", "csv", "debug", "color"

# evalexpr - "eval" macro
# chrono - "date", "time" macro
# lipsum - "lipsum" macro
# csv - "from", "table" macro

# debug - Enable debug method
# color - Enable color prompt
```
**rust file**
```rust
use rad::RadError;
use rad::Processor;
use rad::MacroType;
use rad::auth::AuthType;

// Builder pattern
let processor = Processor::new()
    .purge(true)
    .greedy(true)
    .silent(true)
    .strict(true)
    .custom_rules(Some(vec![Pathbuf::from("rule.r4f")])) // Read from frozen rule files
    .write_to_file(Some(PathBuf::from("out.txt")))?      // default is stdout
    .error_to_file(Some(PathBuf::from("err.txt")))?      // default is stderr
    .discard(true)?                                      // discard all output
    .unix_new_line(true)                                 // use unix new line for formatting
    // Permission
    .allow(Some(vec![AuthType::ENV]))
    .allow_with_warning(Some(vec![AuthType::CMD]))
    // Debugging options
    .debug(true)                                         // Turn on debug mode
    .log(true)                                           // Use logging to terminal
    .interactive(true)                                   // Use interactive mode
    // Create unreferenced instance
    .build(); 

// Use Processor::empty() instead of Processor::new()
// if you don't want any default macros

// Add basic rules(= register functions)
// test function is not included in this demo
processor.add_basic_rules(vec![("test", test as MacroType)]);

// Add custom rules(in order of "name, args, body") 
processor.add_custom_rules(vec![("test","a_src a_link","$a_src() -> $a_link()")]);

processor.from_string(r#"$define(test=Test)"#);
processor.from_stdin();
processor.from_file(Path::new("from.txt"));
processor.freeze_to_file(Path::new("out.r4f")); // Create frozen file
processor.print_result();                       // Print out warning and errors count
```
