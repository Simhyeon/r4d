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

#### Macro definition

Definition syntax is similar to macro invocation but requires a specific form
to sucessfully register the macro.

```
$define(name,arg1 arg2=$arg1() $arg2())
```

- First argument is a macro name. Macro should start with an alphabet character
and following characters should be either alphanumeric or underscore.
- Second argument is macro's arguments. Macro argument also follows same rule
of naming. Multiple arguments can be declared and should be separated by a
whitespace.
- Third argument is a macro body. Any text can be included in the macro body
while an unbalanced parenthesis will prevent processor's work. Currently there
is no way to include unbalanced parenthesis inside definition body. 

You can simply bind the value without giving arguments.

```
$define(v_name=Simon creek)
```

##### Caveats

Definition's body can include any macro invocation in itself, thus wrong
argument declaration cannot be detected at the time of definition. To make
matters worse, arguments doesn't have any types either.

```
$define(panik,kalm=$calm())
$panik()
$define(calm=KALM)
$panik()
===
error: Failed to invoke a macro : "calm"
 --> stdin:2:2
$calm()
KALM // After defnition it prints out without error
```

Thus, idiomatic way to check if definition has sane invocations is to invoke a
dummy after declartion if possible. Although this might not be avilable all the
time.

```
$define(
	test,
	a_expr a_path
	=
	$bind+(cond,$eval($a_expr()))
	$bind+(true_path,$path(cache,$a_path()))
	$bind+(false_path,$path(cache,index.md))
	$ifelse(
		$cond(),
		$include($true_path()),
		$include($false_path())
	)
)
$test|(1==1,out.md)

// Real usage
$test($ifdef(mod_mw),out.md)
===
// Some outputs...
```

#### Macro inovokation

Prefix is a dollar sign($)
```
$define(macro_name,a1 a2=$a1() $a2())
$macro_name(arg1, arg2)
```
Macro can be invoked anywhere after the definition.
```
My name is $macro_name(Simon, Creek).
```
converts to
```
My name is Simon Creek.
```

Special argument ```$:``` is used for iterated value.
```
$foreach(\*John,Simon,Jane*\,Name : $:
)
$forloop(5,10,$:th
)
```
converts to
```
Name : John
Name : Simon
Name : Jane

5th
6th
7th
8th
9th
10th

```

**NOTE**

An unbalanced parenthesis changes the behaviour of macro invocation and a
non-double-quoted comma will change the number or content of arguments. If
desirable content includes unbalanced parentheses or commas, enclose the body
with string literal with the syntax of ```\* TEXT GOES HERE *\```, yet literal
inside macro body will printed as is. Or put escacpe character before ending
parenthesis, though this won't work in macro definition.

```
$repeat(2,I'm,comma,separated
)
===
I'mI'm
```
To include commas you need to enclose with string literal
```
$repeat(2,\*I'm,comma,separated*\
)
===
I'm,comma,separated
I'm,comma,separated

```

### Macro attributes

**Trim**

Trim attribute trims preceding and following newlines, whitespaces from output.
To make a definition look much more easier to read, many blank spaces are often
included.

for e.g.

```
$define(
	test,
	a_expr a_path
	= // This new line
	$bind+(cond,$eval($a_expr())) // Whitespaces before "$bind"
	$bind+(true_path,$path(cache,$a_path())) // Whitespaces before "$bind"
	$bind+(false_path,$path(cache,index.md)) // Whitespaces before "$bind"
	$ifelse( // Whitespaces before "$ifelse"
		$cond(),
		$include($true_path()),
		$include($false_path())
	) // This new line
)
$test(1==1,out.md)
===


                1
2
3
4
5



```

Such definition is easier to read, but makes formatting unpredictable. So trim
attribute comes handy, although you can always manually call trim macro.

```
...
$test^(1==1,out.md)
// This is same with 
// $trim($test(1==1,out.md))
===
1
2
3
4
5
```

**Greedy**

Greedy attribute ```+``` parses arguments as greedy as possible so that last
argument will include or following characters in it. This is useful when
argument is a big chunk of data and includes multiple commas.

```
$define(test,a b c=$a() $b() $c())
$test(first, second, third, fourth, fifth, sixth)
$test+(first, second, third, fourth, fifth, sixth)
===
first  second  third
first  second  third, fourth, fifth, sixth
```

**Piping**

Pipe attribute ```|``` saves macro output into temporary value. This is useful
when you have to use mutlple macros for desired output which makes it hard to
grasp the code and maintain them.

```
$define(test,a=$a())
$test|(I'm going to be used by a pipe macro)
$trim($repeat(2,$-()
))
$test|(\*I'll be requoted with literal*\)
$-*()
===
I'm going to be used by a pipe macro
I'm going to be used by a pipe macro
\*I'll be requoted with literal*\
```

**Yield literal**

Yield literal attribute ```*``` makes output printed as literal form. This is
useful when you have to give an argument literal value but you need to pre
process the data which means it cannot be marked as literal.

```
$define(array,content=$regex($content(), ,\*,*\))
$foreach($array*(a b c),Iterated: $:
)
$foreach(\*$regex(a b c, ,\*,*\)*\,Iterated: $:
)
===
Iterated: a
Iterated: b
Iterated: c

warning: Unbalanced parenthesis detected.
 --> stdin:2:13
Iterated: $regex(a b c
Iterated:
Iterated: \*
Iterated: *\)

warning: found 1 warnings
// This is because foreach evaluate expression once more and "*\)" doesn't have
matching "(" character.
```

```
$ifelse?(false,TRUE,$path(a,b))
===
$path(a
// Though greedy attribute can be used in this case
// $ifelse?+(false,TRUE,$path(a,b)) will be evaluated to a/b
```

### Errors

There are two types of errors. One is failed invocation error and second is
invalid argument error. Failed invocation means that macro definition was not
present at the time. On the other hand, invalid argument error is panic error
since ignoring such error is very undesirable for end user experience..

```
$define(test=Test)
$tesT()
$include($path(typo_in_name, index.md))
===
error: Failed to invoke a macro : "tesT"
 --> test:2:2

$tesT()
error: Invalid argument
= File path : "typo_in_name/index.md" doesn't exist
 --> test:3:2
Processor panicked, exiting...
```

### Goal

R4d aims to be a modern alternative to m4 processor, which means

- No trivial m4 quotes for macro definition
- Explicit rule for macro definition and usage so that de facto underscore rule
is not necessary
- Easier binding with other programming languages(Rust's c binding)
- Enable combination of file stream and stdout
- As expressive as current m4 macro processor

#### Built-in macros (or macro-like functions)

[Usages](./docs/macros.md)
