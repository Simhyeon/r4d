## Usage

### Rad binary

```bash
# Usage : rad [OPTIONS] [FILE]...

# Read from file and save to file
rad input_file.txt -o out_file.txt

# Read from file and print to stdout 
rad input_file.txt

# Read from standard input and print to file
printf 'text' | rad -o out_file.txt

# Read from stdin and print to stdout 
printf 'text' | rad 

# Print simple manual
rad --man 
rad --man ifdef

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
-s, --silent <OPTION> # Suppress warnings default is all. All|Security|sanity|None
-l, --lenient         # Disable strict mode. Print original if macro doesn't exist.
-p, --purge           # Purge mode, print nothing if a macro doesn't exist.
    --assert          # Enable assertion mode

# Use following options to decide deubbing behaviours
# default is not to debug
# you need to enable debug mode first to use other debug flags
-d, --debug           # Start debug mode
    --log             # Print all macro invocation logs
-i                    # Start debug mode as interactive, this makes stdout unwrapped
    --diff            # Show diff result between source and processed result
    --dryrun          # Dry run macros

# Other flags
-n                    # Always use unix newline (default is '\r\n' in windows platform)
-D, --discard         # Discard all output

# Freeze(zip to binary) rules to a single file
# Frozen file is a bincode file thus, theoretically faster in reading
rad test --freeze --out frozen.r4f
# Melt a file and use in processing
rad test --melt frozen.r4f

# Package into a single file
rad test --package --out bin.r4c

# Print signature information into file
rad --signature sig.json
# Print signature to stdout but only runtime macros
rad --signature --sigtype runtime

# Available signature types
# all (This is the default value for signature type)
# default
# runtime
```

Type ```-h``` or ```--help``` to see full options.

### Rado binary

```bash
# Edit source file
# It invokes default editor "RADO\_EDITOR"
rado edit file_name.txt

# Read processed file.
# Processed file is saved in temporary directory "RADO\_DIR".
# This command processes source file only when source file's timestamp is newer
# than processed file's timestamp.
# You can set processed file ith "out" flag.
# You can send rad arguments with last arguments in "--" forms
rado read file_name.txt
rado read file_name.txt -o other.txt
rado read file_name.txt -o other.txt -- -a fin+env

# Process in-place
# Backup is saved in std's temporary directory not rado's temporary directory
rado replace file_name.txt

# Clear temporary files
rado clear

# Sync file's timestamp
rado sync file_name.txt

# Force process regardless of file's timestamep with optional read after flag
rado force file_name.txt
rado force file_name.txt --read

# Package file into a packaged script
rado package file_name.txt

# Execute a file wihtout using path variation
rado execute file_name.r4d
rado execute bin.r4c

# Print out all environment variables
rado env
```

### Library 

**Cargo.toml**

```toml
[dependencies]
r4d = { version = "*", features = ["full"] }

# Other available features are... 

# evalexpr  - "eval", "evalk" macro
# chrono    - "date", "time" macro
# textwrap  - Enable "wrap" macro
# cindex    - Query related macros
# full      - Enable all features

# debug     - Enable debug methods
# color     - Enable color prompt
# signature - Enable signature map
# hook      - Enable hook macro
# template  - Enbale multiple template macros
```

**rust file**

```rust
use r4d::RadResult;
use r4d::Processor;
use r4d::AuthType;
use r4d::CommentType;
use r4d::DiffOption;
use r4d::MacroType;
use r4d::WarningType;
use r4d::Hygiene;
use r4d::HookType; // This is behind hook feature
use std::path::Path;

// Assume following codes return "Result" at the end
// Builder
let mut processor = Processor::new()
    .set_comment_type(CommentType::Start)                // Use comment
    .custom_macro_char('~')?                             // use custom macro character
    .custom_comment_char('#')?                           // use custom comment character
    .purge(true)                                         // Purge undefined macro
    .silent(WarningType::Security)                       // Silents all warnings
    .assert(true)                                        // Enable assertion mode
    .lenient(true)                                       // Disable strict mode
    .aseptic(true)                                       // Enable aseptic mode
    .hygiene(Hygiene::Macro)                             // Enable hygiene mode
    .pipe_truncate(false)                                // Disable pipe truncate
    .write_to_file(Path::new("out.txt"))?                // default is stdout
    .error_to_file(Path::new("err.txt"))?                // default is stderr
    .unix_new_line(true)                                 // use unix new line for formatting
    .discard(true)                                       // discard all output
    .melt_files(&[Path::new("source.r4d")])?             // Read runtime macros from frozen
    // Permission
    .allow(&[AuthType::ENV])                             // Grant permission of authtypes
    .allow_with_warning(&[AuthType::CMD])                // Grant permission of authypes with warning enabled
    // Debugging options
    .debug(true)                                         // Turn on debug mode
    .log(true)                                           // Use logging to terminal
    .diff(DiffOption::All)?                              // Print diff in final result
    .interactive(true);                                  // Use interactive mode

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
    HookType::Macro,            // Macro type
    "trigger_macro",            // Macro that triggers
    "hook_div",                 // Macro to be executed
    1,                          // target count
    false                       // Resetable
)?;

// Add runtime rules(in order of "name, args, body")
processor.add_runtime_rules(&[("test","a_src a_link","$a_src() -> $a_link()")])?;

// Add custom rules without any arguments
processor.add_static_rules(&[("test","TEST"),("lul","kekw")])?;

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
