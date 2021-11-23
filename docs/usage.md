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

# Use comment in input texts
# Comment character is '%'
# Refer macro_syntax for further information
rad --comment
rad --comment any

# Some macros need permission to process
# use following options to grant permission.
# Permission argument is case insensitive
-a env                # Give environment permission
-a cmd                # Give syscmd permission
-a fin+fout           # give both file read and file write permission
-A                    # Give all permission. this is same with '-a env+cmd+fin+fout'
-w env                # Give permission but warn when macro is used
-W                    # Same with '-A' but for warning

# Use following options to decide error behaviours
# default is stderr
-e, --err <FILE>      # Log error to <FILE>
-s, --silent          # Suppress warnings
-l, --lenient         # Disable strict mode
    --nopanic         # Don't panic in any circumstances
	--assert          # Enable assertion mode

# Use following options to decide deubbing behaviours
# default is not to debug
-d, --debug           # Start debug mode
    --log             # Print all macro invocation logs
-i                    # Start debug mode as interactive, this makes stdout unwrapped
    --diff            # Show diff result between source and processed result

# Other flags
-n                    # Always use unix newline (default is '\r\n' in windows platform)
-p, --purge           # Purge mode, print nothing if a macro doesn't exist doesn't work in strict mode
-g, --greedy          # Always enable greedy for every macro invocation
-D, --discard         # Discard all output

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
rad = { version = "0.1.0", features = ["full"] }

# Other available features are 
# "evalexpr", "chrono", "lipsum", "csv", "debug", "color", "full"

# evalexpr - "eval" macro
# chrono   - "date", "time" macro
# lipsum   - "lipsum" macro
# csv      - "from", "table" macro
# full     - Enable evalexpr, chrono, lipsum and csv

# debug    - Enable debug methods
# color    - Enable color prompt
```
**rust file**
```rust
use rad::RadError;
use rad::Processor;
use rad::MacroType;
use rad::AuthType;
use rad::CommentType;
use std::path::Path;

// Builder
let mut processor = Processor::new()
    .custom_macro_char('~')                              // use custom macro character
    .custom_comment_char('#')                            // use custom comment character
    .comment(CommentType::Start)                         // Use comment
    .purge(true)                                         // Purge undefined macro
    .greedy(true)                                        // Makes all macro greedy
    .silent(true)                                        // Silents all warnings
    .nopanic(true)                                       // No panic in any circumstances
    .assert(true)                                        // Enable assertion mode
    .lenient(true)                                       // Disable strict mode
    .custom_rules(Some(vec![Path::new("rule.r4f")]))?    // Read from frozen rule files
    .write_to_file(Some(Path::new("out.txt")))?          // default is stdout
    .error_to_file(Some(Path::new("err.txt")))?          // default is stderr
    .unix_new_line(true)                                 // use unix new line for formatting
    .discard(true)                                       // discard all output
    // Permission
    .allow(Some(vec![AuthType::ENV]))                    // Grant permission of authtypes
    .allow_with_warning(Some(vec![AuthType::CMD]))       // Grant permission of authypes with warning enabled
    // Debugging options
    .debug(true)                                         // Turn on debug mode
    .log(true)                                           // Use logging to terminal
    .interactive(true)                                   // Use interactive mode
    .diff(true)                                          // Eanble diff variant
    // Create unreferenced instance
    .build(); 

// Comment char and macro char cannot be same 
// Unallowed pattern for the characters are [a-zA-Z1-9\\_\*\^\|\+\(\)=,]

// Use Processor::empty() instead of Processor::new()
// if you don't want any default macros

// Print information about current processor permissions
// This is an warning and can be suppressed with silent option
processor.print_permission()?;

// Add basic rules(= register functions)
// test function is not included in this demo
processor.add_basic_rules(vec![("test", test as MacroType)]);

// You can add basic rule in form of closure too
processor.add_closure_rule(
    "test",                                                       // Name of macro
    2,                                                            // Count of arguments
    Box::new(|args: Vec<String>| -> Option<String> {              // Closure as an internal logic
        Some(format!("First : {}\nSecond: {}", args[0], args[1]))
    })
);

// Add custom rules(in order of "name, args, body") 
processor.add_custom_rules(vec![("test","a_src a_link","$a_src() -> $a_link()")]);

// Process with inputs
// This prints to desginated write destinations
processor.from_string(r#"$define(test=Test)"#)?;
processor.from_stdin()?;
processor.from_file(Path::new("from.txt"))?;

processor.freeze_to_file(Path::new("out.r4f"))?; // Create frozen file

// Print out result
// This will print counts of warning and errors.
// It will also print diff between source and processed if diff option was
// given as builder pattern.
processor.print_result()?;                       
```
