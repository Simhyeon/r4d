### R4d (Rad)

R4d is a text oriented macro prosessor made with rust.

R4d is in very early stage, so there might be lots of undetected bugs.

### Usage

```bash
# Usage : rad [OPTIONS] [FILE]...

# Read from file and save to file
rad input_file.txt -o out_file.txt

# Read from file and print to stdout (without error print)
rad input_file.txt 2>/dev/null

# Read from standard input and print to file
printf '...text...' | rad -o out_file.txt

# Read from stdin and print to stdout (without error print)
printf '...text...' | rad 2>/dev/null
```

### Syntax 

#### Macro definition

Definition is also a macro. However it requires specific form not like other
macros.

```
$define(name,arg1 arg2, $arg1() $arg2())
```

- First argument is a macro name. Macro  should start with alphabets and
following characters should be either alphanumeric or underscore.
- Second argument is macro's arguments. Macro argument also follows same rule
of naming. Multiple arguments can be declared and should be separated by a
whitespace.
- Third argument is a macro body. Any text can be included in the macro body
while an unbalanced right parenthesis will end the definition. Enclose the body
with double quote or escape with backslash to type literal parenthesis. Any
character including newlines('\n', "\r\n") are all respected. (UTF-8)

#### Macro inovokation

Prefix is a dollar sign($)
```
$define(macro_name,a1 a2,$a1() $a2())
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

Special argument ```$_``` is used for iterated value.
```
$foreach("John,Simon,Jane",Name : $_
)
$forloop("5,10",$_th
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

An unbalanced right parenthesis ends macro invocation and non-double-quoted
comma will change the number or content of arguments. If desirable content
includes unbalanced right parentheses or commas, enclose the body with double
quotes. Use escaped form of double quote, ```\"``` to use literal comma inside
double quotes.

### Goal

R4d aims to be a modern m4 alternative, which means

- No trivial m4 quotes for macro definition
- Explicit rule for macro definition and usage so that de facto underscore rule
is not necessary
- Easier binding with other programming languages(Rust's c binding)
- Enable combination of file stream and stdout
- As expressive as current m4 macro processor

#### In-built macros(WIP)

**Implemented**

- define
- undef
- include
- repeat
- foreach
- forloop
- ifelse
- syscmd
- rsub, rdel
- eval
- trim, chomp, comp
- lipsum
- time, date
- ifdef

**Not implemented**

- csv sql query
- csv formatting(gfm table, wititext table, latex table)
- Macro automation 

**Possibly**

- Web request macro
There would many caveats implementing web request macro, because it is
inherently insecure + r4d targets gnu linux, windows and possibly musl
linux(alpine linux), thus configuing libcurl and libssl compatible global
interface might not be easy, I guess
