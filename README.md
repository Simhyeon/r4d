### R4d (Rad)

R4d is a text oriented macro prosessor aims to be an alternative to m4 macro
processor.

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

$static(queried,$query(SELECT id,first_name,address FROM addr where first_name = John))$dnl()

% TABLE_FORM == github
$table+($env(TABLE_FORM),$queried())

$wrap(40,$lipsum(15))
```
**Processed texts**
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
```

### Install

I recommend using ```cargo install``` until I prepare a proper CD
pipeline.

e.g.

```bash
# Binary features is mandatory or else it is not an executable
cargo install r4d --features binary --locked

# If you need color prompt, then use features "color"
cargo install r4d --features binary,color --locked

# Refer lib.rs or usage section for detailed feature usage
```

But you can download some binaries in [release
page](https://github.com/Simhyeon/r4d/releases)

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
    .greedy(true)
    .write_to_file(Some(PathBuf::from("cache.txt")))?;

processor.from_file(Path::new("input.txt"))?;
processor.print_result()?;
```

### Usage

[Detailed usage](./docs/usage.md)

### Syntax 

[Macro syntax](./docs/macro_syntax.md)

### Built-in macros

[Macros](./docs/macro_indices.md)

### Macro types

[Types](./docs/macro_types.md)

### Extend processor with storage feature

[Storage](./docs/storage.md)

### How to debug

[Debug](./docs/debug.md)

### Goal

R4d aims to be a modern alternative to m4 processor, which means

- No trivial m4 quotes for macro definition
- An explicit rule for macro definition and usage so that de facto underscore
rule is not necessary
- Easier binding with other programming languages(Rust's c binding)
- Enable combination of file stream and stdout
- As expressive as current m4 macro processor

### CVE related issue

[Setvar data races
vulnerability](https://nvd.nist.gov/vuln/detail/CVE-2020-26235) is known issue
at the time but for current state, it doesn't affect r4d since r4d doesn't
utillize multi threading in processing. Every time,date call and env_set calls
are sequential and doensn't overlap.

If you're using an operating system that doesn't utilize libc, then its fine.
