### R4d (Rad)

R4d is a text oriented macro prosessor made with rust.

### NOTE

**Is it stable?**

I'm currently dog fooding for bug detections, and there might be some
undetected bugs. However it doesn't mean that the syntax will change in the
future without breaking release. (At least until 2.0)

The reason r4d is not yet in 1.0 is that I want to make sure that followings
are solved.

- Absence of critical bugs
- No more basic macro behaviour(internal logic) changes

Therefore it is quite stable, in terms of non-breakable syntax, but there will
be bug fixes and non breaking changes.

### Demo

**Raw texts**
```text
$define(author=Simon Creek)
$define(title=R4d demo)
---
title : $title()
author : $author()
---
My name is $author() and I made r4d to make macros can be used within various
forms of texts. This article was written in $date() $time().

$ifdef(test, This should be only printed when I'm testing not in release)

This is some important table automatically formatted according to environment
variable.

$table($env(TABLE_FORM),\*H1,H2,H3
a,b,c
d,e,f*\)

I'm out of idea and I need some texts, $lipsum(15) 
```
**Processed texts**
```
---
title : R4d demo
author : Simon Creek
---
My name is Simon Creek and I made r4d to make macros can be used within various
forms of texts. This article was written in 2021-09-26 21:36:59.


This is some important table automatically formatted according to environment
variable.

|H1|H2|H3|
|-|-|-|
|a|b|c|
|d|e|f|

I'm out of idea and I need some texts, Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore.
```

### Install

You can download binaries in [release page](https://github.com/Simhyeon/r4d/releases)

If your platform is not listed, mostly MacOS, directly install from cargo registry.

```bash
# Binary features is mandatory or else it is not an executable
cargo install r4d --features binary
# If you need color prompt, then use features "color"
cargo install r4d --features binary,color
```

### Simple usage

**Binary**
```
# Read from file and print to stdout 
rad input_file.txt
# Read from standard input and print to file
printf '...text...' | rad -o out_file.txt
```

**Library**
```rust
use rad::RadError;
use rad::Processor;

let processor = Processor::new()
    .purge(true)
    .greedy(true)
    .write_to_file(Some(PathBuf::from("cache.txt")))?
    .build(); 

processor.from_file(Path::new("input.txt"))?;
processor.print_result()?;
```

### Usage

[Detailed usage](./docs/usage.md)

### Syntax 

[Macro syntax](./docs/macro_syntax.md)

### Basic macros (or macro-like functions)

[Macros](./docs/basic_macros.md)

### Macro types

[Types](./docs/macro_types.md)

### How to debug

[Debug](./docs/debug.md)

### Goal

R4d aims to be a modern alternative to m4 processor, which means

- No trivial m4 quotes for macro definition
- An explicit rule for macro definition and usage so that de facto underscore rule
is not necessary
- Easier binding with other programming languages(Rust's c binding)
- Enable combination of file stream and stdout
- As expressive as current m4 macro processor
