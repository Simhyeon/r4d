### R4d (Rad)

R4d is a text oriented macro prosessor that is modern, expressive, and easy to
use.

[Changes](./docs/change.md)

[3.0 Changes](./docs/3_0.md)

### NOTICE

Use rad version **3.0.1** instead of 3.0. Refer a [md](docs/3_0_hassle.md) for
a reason.

### Table of conents

* [Demo](#demo)
* [Install](#install)
* [Simple usage](#simple-usage)
* [Documentation](#documentation)

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

```bash
# Binary with full macros support
cargo install r4d --features binary --locked

# Only include macros that doesnt't require external crates
cargo install r4d --features basic --locked

# Refer docs.rs or usage section for detailed feature usage
```

### Simple usage

**Binary**

There are two binaries of each **rad** and **rado**. Rad is a main processor
and rado is a wrapper binary.

```
# rad
# Read from a file and print to stdout 
rad input_file.txt
# Read from standard input and print to a file
printf '...text...' | rad -o out_file.txt
# Get a simple manual for a macro
rad --man ifdef

# rado
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
    .write_to_file(PathBuf::from("cache.txt"))?;

processor.process_file(Path::new("input.txt"))?;
processor.print_result()?;
```

## Documentation

[index](./docs/index.md)

## Known issues

[issues](./docs/issues.md)
