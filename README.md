### R4d (Rad)

R4d is a text oriented macro prosessor aims to be an alternative to m4 macro
processor.

[Changes](./docs/change.md)

[3.0 Changes](./docs/3_0.md)

### Note

Because crates.io's readme is tied to version. There might be undocumented
readme changes. Please use [github](https://github.com/simhyeon/r4d) for
latest information.

### Demo

**Raw texts**
```text
$define(author=Simon Creek)
$define(title=R4d demo)
---
title  : $title()
author : $author()
---
My name is $author() and I made r4d to make macros can be used within various
forms of texts. This article was written in $date() $time().

$ifdef(test, This should be only printed when I'm testing not in release)$dnl()

This is some important table automatically formatted according to environment
variable.

$regcsv(addr,$include(addr.csv))$dnl()

$static(
    queried,
    $query(
        SELECT id,first_name,address 
        FROM addr where first_name = John
    )
)$dnl()

% Comments are disabled by default for better compatibility
% TABLE_FORM == github
$table($env(TABLE_FORM),$queried())

$wrap(40,$lipsum(15))

Evaluation : $prec($eval( $num(0.1second) + $num(0.2sekunde)),2)
Evaluation : $evalk( 1 + 2 )
```
**Processed texts**

Ran with ```TABLE_FORM=github rad file_name.md -a env+fin --comment```

```
---
title  : R4d demo
author : Simon Creek
---
My name is Simon Creek and I made r4d to make macros can be used within various
forms of texts. This article was written in 2022-01-18 16:38:07.

This is some important table automatically formatted according to environment
variable.

|id|first_name|address|
|-|-|-|
|1|John|111-2222|
|2|John|222-3333|

Lorem ipsum dolor sit amet, consectetur
adipiscing elit, sed do eiusmod tempor
incididunt ut labore.

Evaluation : 0.30
Evaluation : 1 + 2 = 3
```

### Install

[Install rust toolchain for build](https://www.rust-lang.org/tools/install)

I recommend using ```cargo install``` until I prepare a proper CD
pipeline.

Currently version 3.0 is preparing for release and can be downloaded with
specific version flag.

```bash
cargo install r4d --git https://github.com/simhyeon/r4d --features binary --locked --version 3.0.0-rc.4
```

Or use crates.io registry,

e.g.

```bash
# Binary with full macros support
cargo install r4d --features binary --locked

# If you need color prompt, then use features "color"
cargo install r4d --features binary,color --locked

# Only include macros that doesnt't require external crates
cargo install r4d --features basic --locked

# Refer docs.rs or usage section for detailed feature usage
```

### Simple usage

**Binary**

There are two binaries each **rad** and **rado**. Rad is a main processor and
rado is a open+edit binary.

```
# Read from a file and print to stdout 
rad input_file.txt
# Read from standard input and print to a file
printf '...text...' | rad -o out_file.txt
# Get a simple manual for a macro
rad --man ifdef

# Edit a source file
rado edit file_name.txt
# Read a processed file
rado read file_name.txt
# Print environment variables
rado env
```

**Library**
```rust
use r4d::RadError;
use r4d::Processor;

let processor = Processor::new()
    .write_to_file(Some(PathBuf::from("cache.txt")))?;

processor.process_file(Path::new("input.txt"))?;
processor.print_result()?;
```

### [Detailed usage](./docs/usage.md)

### [Macro syntax](./docs/macro_syntax.md)

### [Built-in macros](./docs/macro_indices.md)

### [About modes](./docs/modes.md)

### [Macro types](./docs/macro_types.md)

### [How to extend function macros](./docs/ext.md)

### [Extend processor with storage feature](./docs/storage.md)

### [How to debug](./docs/debug.md)

### Goal

R4d aims to be a modern alternative to m4 processor, which means

- No trivial m4 quotes
- An explicit rule for macro definition and invocation
- Enable combination of file stream and stdout
- As expressive as current m4 macro processor
