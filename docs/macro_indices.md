**NOTE**

Currently r4d is preparing 3.0 release and 3.0 has many hard breaking changes
such as argument order change, macro removal, macro renames. Use rad's man flag
if you want to see most updated information.

```bash
# This needs signature feature
# Signature feature is included in binary but not in basic

# Get all manuals
rad --man

# Get a specific manual
rad --man ifelse
```

## Format

If any permission is necessary, it is displayed as

AUTH : (ENV|FIN|FOUT|CMD)

Use ```-a env+fin``` syntax to allow multiple permission or ```-A``` to allow
all.

Macro expansion demonstration is displayed as

```
$macro_invocation(...)
% As some comments
===
Evaluated text goes here
Expanded text from macro // This is a demonstration comment and not a real comment
```

## Table of conents

For assertion macros refer [debug part](./debug.md)

### Macros 

* [define](#define)
* [declare](#declare)
* [static, staticr](#static)
* [let, letr](#let)
* [undef](#undef)
* [docu](#docu)
* [rename](#rename)
* [repl](#repl)
* [append](#append)
* [counter](#counter)

### Text

<!-- Formatting -->
* [upper](#upper)
* [lower](#lower)
* [wrap](#wrap)
* [repeat](#repeat)
* [surr](#surr)
* [tr](#tr)
* [align](#align)
* [table](#table)
<!-- Extraction -->
* [num](#num)
* [sub](#sub)
* [head, headl](#head)
* [tail, taill](#tail)
<!-- Text creation -->
* [lipsum](#lipsum)
* [unicode](#unicode)
<!-- Meta information -->
* [len](#len)
* [countw](#countw)

### Array

<!-- Array creation -->
* [spilt](#spilt)
* [chars](#chars)
<!-- Array indexing -->
* [min](#min)
* [max](#max)
* [index](#index)
<!-- Array modification -->
* [sort](#sort)
* [rev](#rev)
<!-- Array transformation -->
* [join](#join)
* [fold](#fold)
<!-- Array meta data -->
* [count](#count)

### Lines

<!-- Lines modification -->
* [sortl](#sortl)
* [indent](#indent)
<!-- Lines transformation -->
* [joinl](#joinl)
* [foldl](#foldl)
<!-- Lines meta data -->
* [countl](#countl)

### Number

<!-- Number modification  -->
* [floor](#floor)
* [ceil](#ceil)
* [prec](#prec)
* [notat](#notat)
<!-- Number combination -->
* [eval](#eval)
* [evalk](#evalk)

### Qualification

* [not](#not)
* [cmp](#cmp)
* [isempty](#isempty)
* [istype](#istype)

### Conditional

* [if](#if)
* [ifelse](#ifelse)
* [ifdef](#ifdef)
* [ifdefel](#ifdefel)
* [ifenv](#ifenv)
* [ifenvel](#ifenvel)

### Loop

* [forby](#forby)
* [foreach](#foreach)
* [forline](#forline)
* [forloop](#forloop)

### Files

* [listdir](#listdir)
* [fileout](#fileout)
* [include](#include)
* [source](#source)
* [import](#import)
* [readin](#readin)
* [readto](#readto)
* [tempin](#tempin)
* [tempout](#tempout)
* [tempto](#tempto)

### Path manipulation

* [name](#name)
* [parent](#parent)
* [path](#path)
* [abs](#abs)
* [input](#input)
* [temp](#temp)

### Space manipulation

* [trim](#trim)
* [triml](#triml)
* [trimla](#trimla)
* [chomp](#chomp)
* [comp](#comp)
* [squash](#squash)
* [EB](#EB)
* [dnl](#dnl)
* [enl](#enl)

### Piping

* [pipe](#pipe)
* [pipeto](#pipeto)
* [-](#-)

### Semantics

* [empty](#empty)
* [space](#space)
* [tab](#tab)
* [nl](#nl)
* [comma](#comma)

### Regex 

* [find](#find)
* [findm](#findm)
* [grep](#grep)
* [grepl](#grepl)
* [grepf](#grepf)
* [regex](#regex)
* [regexpr](#regexpr)

### Time

* [time](#time)
* [date](#date)
* [hms](#hms)
* [ftime](#ftime)

### Expansion

* [exec](#exec)
* [spread](#spread)

## Environment variable

* [env](#env)
* [envset](#envset)

### Querying

* [regcsv](#regcsv)
* [query](#query)
* [dropcsv](#dropcsv)

### Operational

* [panic](#panic)
* [relay](#relay)
* [halt](#halt)
* [exit](#exit)
* [pause](#pause)
* [escape](#escape)
* [clear](#clear)
* [hygiene](#hygiene)

### Storage

* [update](#update)
* [extract](#extract)

### Special macros

* [syscmd](#syscmd)
* [que](#que)
* [ifque](#ifque)
* [strip](#strip)

### Debugging

* [log](#log)
* [logm](#logm)

## Indices

### Macros

#### define

Define creates a runtime macro. Define cannot be renamed, undefined or
overriden.

Refer [Macro syntax](./macro_syntax.md) for detailed usage

```
$define(name,a1 a2=$a1() + $a2())
===
```

#### declare

You can simply declare macros without defining its body.

This is useful when you simply need macros to be defined so that other macros
can utilizie them.

```
$declare(n0)
$declare(n1,n2,n3)
$ifdef(n0,I'm defined)
$ifdef(n3,I'm also defined)
===
I'm defined
I'm also defined
```

#### static

Statically binds an expression that persists for the whole processing. Static
is useful when you don't need dynamic evaluation but statically bound value.
Because definition is evaluated on every call which might not be necessarily
efficient or not be an intended behaviour.

"Static" trims bound value by default. Use **staticr** variant if you don't want
such behaviour.

```
$pipe(I'm piped)
$define(dyn=$-())
$static(stt,$-())
$stt()
$dyn()
===
I'm piped
warning: Empty pipe
 --> demo:2~~:2
warning: found 1 warnings
```

#### let

Creates a new local macro. This macro is automatically clared after evalution
of the macro.

Local macro is scope specific, thus cannot be accessed from outside.

"Let" trims bound value by default. Use **letr** variant if you don't want such
behaviour.

```
$define(let_test=
    $let(lc,
        ^^^ Bound Value ^^^
    )
    $lc()
)
$let_test^()
$lc()
===
^^^ Bound Value ^^^
error: Invalid macro name
= No such macro name : "lc"
 --> demo:8:2
error: Strict error
= Every error is panicking in strict mode
 --> demo:8:2
=== Processor panicked ===
```

#### undef

Undefine a macro. Undefine removes all the macros that has a same name.

```
$undef(name)
===
```

#### docu

Docu adds documentation for a given macro. Appended description can be accessed
with rad binary's --man flag.

```
$define(test=)
$docu(test,This is a test macro)
===
```

#### rename

Rename a macro.

```
$rename(len,length)
$length(I'm long)
===
8
```

#### repl

Replace contents of a runtime macro. Changing a content. While redefining a
macro is prohibited in strict mode, but replacing a contents is allowed.

```
$define(before=BEFORE)
$repl(before,AFTER)
$before()
===
AFTER
```

#### append

Appends given string into a macro. Only a runtime or a local macro can be
appended.

```
$define(test=TEST)
$append(test, CASE)
$test()
===
TEST CASE
```

You can give an optional argument to set a trailer. If the macro does not end
with trailer, trailer is automatically added beforehand.

```
$define(arr=)
$append(arr,v1,$comma())
$append(arr,v2,$comma())
$append(arr,v3,$comma())
$arr()
===
v1,v2,v3
```

#### counter

Modify a macro content by 1. A macro should be number. If a macro doesn't exit,
it is automatically created.

```
$counter(ct)
$counter(ct)
$counter(ct)
$ct()
===
v1,v2,v3
```

### Text

#### upper

Get upper class english text.

```
$upper(abcde)
===
ABCDE
```

#### lower

Get lower class english text.

```
$lower(ABCDE)
===
abcde
```

#### wrap

wrap sets given text's width. This uses amazing library of
[textwrap](https://crates.io/crates/textwrap). Wrap supports UTF-8 characters.

```
$wrap(20,$lipsum(10))
===
Lorem ipsum dolor
sit amet consectetur
adipiscing elit. In
rhoncus
```

#### Repeat

Repeat given content for given times
```
$repeat^(3,Content to be repeated
)
===
Content to be repeated
Content to be repeated
Content to be repeated
```

#### surr

Surround text with given pair 

```
$surr()
```

### Array

### Lines

### Number

### Qualification

### Conditional

### Loop

### Files

### Path manipulation

### Space manipulation

### Piping

### Semantics

### Regex

### Time

### Expansion

### Environment variable

### Querying

### Operational

### Storage

### Special

### Debugging

#### pause

Pause literally pauses every macro execution except pause macro. Even define is
not evaluated

```
$pause(true)
$define(some,a=$a())
$eval(1 + 2)
$pause(false)
$define(some,a=$a())
$eval(1 + 2)
===

$define(some,a=$a())
$eval(1 + 2)
3
```

#### include

AUTH : FIN

Include macro reads given file and paste into the position. Included file's
contents are all expanded.

Include macro reads a whole file's contents into a single string. This is an
intended behaviour so that nested include macro inside definition can respect
order of expressions.

```
$include(src/content.rs)
===
% Content of src/content.rs is pasted in here
```

Include's argument path is relative to current input's position.

e.g.
If input script is /home/radman/input.sh, ```$include(src/content.rs)```
fetches file located in /home/radman/content.rs. On
```$include(../dir/test.rs)```, /home/dir/test.rs is fetched.

You can also include file without expansion with optional parameter.

```
% Second argument is whether to include as raw format.
$include(file_name.txt,true)
===
% Content is copy pasted without expansion
```

This internally invokes $escape(true) before include and $escape(false) after
invocation.

#### import

Import frozen file with given path.

```
$import(ext_lib.r4f)
===
```

#### source

Source env styled static definitions. Source files are expanded on read.

```
$source(dec.renv)
===
```

Demo of a source file

```
ctime=$time()
name=Simon Creek
```

#### tempin, tempout, tempto

AUTH : FIN or FOUT

Tempin gets content from temp file named ```rad.txt```. Macros within temp file
is also expanded. Tempout pushes content into current temp file. You can also
change the temp file with tempto.

Temp file is saved in ```%TEMP%``` in Windows and ```/tmp``` in *nix systmes.
```
$tempout(Hello world)             # needs FOUT
$tempin()                         # needs FIN
$tempto(out.json)                 # needs FOUT
$tempout({"name":"simon creek"})
$tempin()
===
Hello world
{"name":"simon creek"}
```

#### relay halt

Relay macro sends all following texts into relay target

Relay is implemented as stack. Thus nested relaying can be properly handled.

```
% Available relay targets are
% - temp
% - file
% - macro
$relay(temp)
$relay(file,out.txt)
$declare(relayed)
$relay(macro,relayed)
===
```

halt stops relaying

```
$halt()
```

#### fileout

AUTH : FOUT

Fileout saves contents to a file. If truncate is false, non existent file
argument is panic behaviour.

```
$fileout(file_name.txt,true,Hello World)
$fileout(file_name.txt,false,This is appended)
===
```

#### env

AUTH : ENV

Print environment variable. Non existent env varaible will yield warning on
strict mode.

```
$env(HOME)
===
/home/simoncreek
```

#### envset

AUTH : ENV

Set environment variable for current shell session. Overriding environment
variable will yield error in strict mode.

```
$envset(CUSTOM_VALUE,I'm new)
$env(CUSTOM_VALUE)
$envset(HOME,/etc/passwd)
$send_log_to_sound_website($env(HOME))
===
I'm new
error: Invalid argument
= You cannot override environment variable in strict mode. Failed to set "HOME"
 --> InnocentScript.sh:3:2
=== Processor panicked ===
```

#### ifenv (deterred macro)

AUTH: ENV

If environment variable is defined, execute expression

```
$ifenv(HOME,$env(HOME))
===
/home/username
```

#### ifenvel (deterred macro)

AUTH: ENV

If environment variable is defined, execute expression else execute another expression

```
$ifenvel(HOME,$env(HOME),No home is defined)
===
% Home is most likely always defined
/home/username
```

#### path

Join elements into a path.

```
$path($env(HOME),document)
$paths(a,b,c)
===
/home/simoncreek/document
a/b/c
```

#### abs

AUTH : FIN

Get absolute(canonicalized) path from argument. This yield panicking error
when there is no such file.

```
$abs(../../some_file.txt)
$abs(../../no_such_file.txt)
===
/home/radman/some_file.txt
error: Standard IO error
= No such file or directory (os error 2)
 --> clumsy_script.sh:1:2
=== Processor panicked ===
```

#### name

Get file name(last part) from input

```
$name(/home/test/Documents/info.txt)
===
info.txt
```

#### parent

Get parent path from input

```
$parent(/home/test/Documents/info.txt)
===
/home/test/Documents
```

#### listdir

AUTH : FIN

This lists file in directory directory.

- Firt argument is whether to print absolute path or not.
- Second optional argument is target path.
- Third optional argument is custom delimter other than comma

```
$listdir(false)
===
src,diff.out,.git,.gitignore,Cargo.lock,README.md,docs,test,auto.sh,wasm,diff.src,Cargo.toml,target,pkg
```

#### pipe

Pipe macro simply saves value to pipe. $-() returns piped value
$-*() returns piped value in literal form.

In addition to normal pipes. You can use named pipe with arguments.

```
$pipe(Value)
$pipeto(other,vallllue)
$-()
$-(other)
===
Value
vallllue
```

#### arr

Create comma separated array from given value. You can set custom delimiter as
second argument(default is single whitespace). You can also filter array with
regex expression in third argument.

```
% Pipe truncate is false
$syscmd|^(ls)
$arr($-(),$nl())
$arr($-(),$nl(),\.sh$) // File that ends with .sh
===
auto.sh,Cargo.lock,Cargo.toml,oush
auto.sh
```

#### sep

Separate an array with given separator.

```
$sep(|,1,2,3,4,5)
=
1|2|3|4|5
```

#### foreach (deterred macro)

Loop around given value. Value is separated with commas. Thus values should be
always escaped. Iterated values are references with ```$:()```.

```
$foreach(\*a,b,c*\,Value: $:()
)
===
Value: a
Value: b
Value: c

```

#### forline (deterred macro)

Loop around given lines. Value is separated with newline.Iterated values are
references with ```$:()```.

```
$forline(a
b
c,Value: $:()
)
===
Value: a
Value: b
Value: c

```

#### forloop (deterred macro)

Loop around given range. Value is separated with commas. Iterated values are
references with ```$:()```.


Range is inclusive e.g. 1 and 3 means from 1 to 3.

```
$forloop(3,5,Number: $:()
)
===
Number: 3
Number: 4
Number: 5

```

#### max

Get the biggest number from given array.

```
$max(1,2,3,4,5)
===
5
```

#### min

Get the smallest number from given array.

```
$min(1,2,3,4,5)
===
1
```

#### ceil

Get ceiling from given number

```
$ceil(1.56)
===
2
```

#### floor

Get floor from given number

```
$floor(1.56)
===
1
```

#### prec

Format number with precision

```
$prec($eval(0.1 + 0.2),2)
===
0.30
```

#### cap

Capitalize given text

```
$cap(abcde)
===
ABCDE
```

#### low

Lower given text

```
$low(ABCDE)
===
abcde
```

#### num

Extract number from given text

```
$num(1km/s)
$eval($num(1km/s) + $num(3km/s))
===
1
4
```

#### rev

Reverse an array

```
$rev(1,2,3,4,5)
===
5,4,3,2,1
```

#### eval

Eval evaluates expression. This macro(function) uses rust's evalexpr crate
[crate link](https://crates.io/crates/evalexpr). Therefore argument formula
follows evalexpr's syntax.

You can keep the origianl formaul with evalk variant.

```
$eval(1+2)
$eval(0.1+0.2)
$evalk( 1 + 2 )
===
3
0.30000000000000004
1 + 2 = 3
```

#### ieval (deterred macro)

Eval in place. This executes $eval( GIVEN EXPRESSION ) and substitute given
macro with the result.

```
$define(counter=1)
$ieval(count,+1)
$counter()
===
2
```

#### if (deterred macro)

If gets a condition and prints if given value is true

Text "true" and "false", non "0" integer and "0" are valid inputs. "true" and
"false" is case sensitive. 0 is false and any number other than 0 is true even
negative integer is valid input. Floating point number is not allowed.

```
$if(true,TRUE)
$if(false,False)
===
TRUE
```

#### ifelse (deterred macro)

Ifelse gets two branches and print out one according to given condition.

```
$ifelse( true ,I'm true,I'm false)
$ifelse( false ,I'm true,I'm false)
$ifelse( 1 ,I'm true,I'm false)
$ifelse( 0 ,I'm true,I'm false)
===
I'm true
I'm false
I'm true
I'm false
```

#### ifdef (deterred macro)

If macro is defined then execute given expression.

```
$define(some=value)
$ifdef(some,Defined)
$undef(some)
$ifdef(some,Not defined)
===
Defined
```

#### not

Not negates given boolean value.

```
$not(true)
$not(false)
$not(1)
$not(0)
===
false
true
false
true
```

#### syscmd

AUTH : CMD

Call system command, on unix system macro calls given command directly. While
windows call are mediated through ```cmd /C``` call.

```
$syscmd(uname -a)
$syscmd(ver)
===
Linux

Microsoft Windows [Version 10......]

```

#### sub

Sub gets substring from given input range. You can give empty value. This is technically same with rust's syntax ```[min..max]```. Also supports utf8 characters.

```
$sub(1,5,123456789)
$sub(2,,123456789)
$sub(,6,123456789)
===
2345
3456789
123456
```

#### head

Get head parts of given value.

- head
- headl

```
$head(4,Text To extract)
$headl(2,aaaaa
bbbbb
ccccc
eeeee)
===
Text
aaaaa
bbbbb
```

#### tail

Get tail parts of given value.

- tail
- taill

```
$tail(7,Text To extract)
$taill(2,aaaaa
bbbbb
ccccc
eeeee)
===
extract
ccccc
eeeee
```

#### strip

Get stripped remainder from given value.

- strip
- stripl

```
$static(TEXT,aaaaa
bbbbb
ccccc
eeeee)
$strip(5,head,Text To extract)
$strip(8,tail,Text To extract)
$stripl(2,head,$TEXT())
$stripl(2,tail,$TEXT())
===
To extract
Text To
ccccc
eeeee
aaaaa
bbbbb
```

#### grep

Grab matching lines from given value.

```
$grep((?i)hello world,hello world
AlOHa woRlD
HELLO WORLD
haLO WORld
HelLo woRlD
holo world
heLLO WOrld)
===
hello world
HELLO WORLD
HelLo woRlD
heLLO WOrld
```

#### index

Get indexed value from given array.

```
$static(idx,5)
$index($idx(),long,array,with,texts,separated,with,columns)
===
with
```

#### sort

sort given value

- sort
- sortl

```
$sort(asec,5,4,3,2,1)
$sortl(desc,abcde
edcba
bhcChicken)
===
1,2,3,4,5
edcba
bhcChicken
abcde
```

#### fold

Fold separated value into non-separated single value.

- fold
- foldl

```
$fold(Hello,World,Without,Space)
$foldl(Lines
Separated
By
Newline characters)
===
HelloWorldWithoutSpace
LinesSeparatedByNewline characters
```

#### count

Count given values.

- count
- countw
- countl

```
$count(a,b,c,d,e)
$countw(Hello world with many spaces)
$countl(a
b
c
d
e
f
g)
===
5
5
7
```

#### tr

Tr translate characters to other characters. Utf8 characters work.

```
$tr(Given String,iSg,aOs)
===
Gaven Otrans
```

#### len

Return the length of given string. This operation takes O(n) not like
traditional O(1) from rust' string data. This is because len returns length of
utf characters not ASCII characters.

```
$len(Lorem ipsum dolor)
$len(ሰማይ አይታረስ ንጉሥ አይከሰስ።)
$len(Зарегистрируйтесь)
$len(สิบสองกษัตริย์ก่อนหน้าแลถัดไป)
$len(⡍⠔⠙⠖ ⡊ ⠙⠕⠝⠰⠞ ⠍⠑⠁⠝)
$len(나는 안녕하지 못하다)
$len(我们刚才从图书馆来了)
===
17
20
17
29
17
11
10
```

#### regex

Regex substitution. This use [regex crate](https://crates.io/crates/regex).

Regex is saved in cache and has hard limit of 100. Until 100 regex creation,
recompilation of regex is prevented.

```
$regex(w.*?d,rust,Hello world)
$regex(//.*$,,Hello World// TODO)
===
Hello rust
Hello World
```

#### find

Find match from source. This return boolean.

```
% Use \\* because \* will trigger literal chunk
% But use \[ to include literal [ since \ doesn't do anything without following
% asterisk
$find(^\\* \[ \],* [ ] Todo)
===
true
```

#### findm

Find multiple occurrences from source. This return integer. If none found, this
will return 0.

```
$findm(a,I have many a's inside me ay.)
$findm(Oops,Hello world)
===
4
0
```

#### trim, chomp, comp, triml

```Trim``` removes preceding and trailing new lines, tabs and whitespaces from
given input. ```Chomp``` removes duplicate newlines from given input ( or say
squeezes multi newlines into a single newline ). ```Comp``` both trim and chomp
given input.

**Caveats**

Chomp converts all CRLF(\\r\\n) into a LF(\\n) for cross platform chomp
behaviour and reformats LF into a processors newline which is CRLF in windows
and LF in unix be default. (Which you can change with --newline flag).

```
$define(value="
UP


DOWN

")
--
$trim($value())
--
$chomp($value())
--
$comp($value())
--
===
--
UP


DOWN
--

UP

DOWN


--
UP

DOWN
--
```

**Triml** trims line by line.

```
$triml(
    1 2 3
  a b c
      가 나 다
)
===
1 2 3
a b c
가 나 다
```

#### nl

Simply print out "newline" characters. This newline respects formatter's
newline. Which is ```\r\n``` for windows and a ```\n``` in *nix systems by
default.

```
$nl()
===

% This is useful when you want to construct an output in one-liner
```

#### enl

Escapes right next newline

```
Before $enl()
After
===
Before After
```

#### dnl

Deny newline after macro execution. This have no effect if next following line
is not empty line.

```
$ifdef(undefined,Print me)$dnl()

Yatti yatta
===
Yatti yatta
```

#### lipsum

Lipsum creates a placehoder with given word counts.

```
$lipsum(5)
===
Lorem ipsum dolor sit amet.
```

#### time, date

Time and date prints current local time and date. This requires features
"chrono".

```
$time()
$date()
===
03:17:11
2021-08-20
```

#### hms

Format second into hh:mm:ss

```
$hms(10500)
===
02:55:00
```

#### from (deterred macro)

From creates formatted macro invocations with given csv values. The given macro
name doesn't need dollar sign prefix.

```
$define(three,a1 a2 a3=1-$a1(), 2-$a2(), 3-$a3())
$from+(three,
a,b,c
d,e,f
)
===
1-a, 2-b, 3-c
1-d, 2-e, 3-f
```

NOTE

Former syntax required data as first parameter, however it was such an pain to
always quote values, thus I found second value as csv was much more ergonomic.

#### table

Table creates a formatted table from given csv values. Currently supported
formats are ```github```, ```wikitext``` and ```html```. This macro doesn't
pretty print but just make it readable from other programs.

```
$table(github,\*a,b,c
1,2,3
4,5,6*\)
$table(wikitext,\*a,b,c
1,2,3
4,5,6*\)
$table(html,\*a,b,c
1,2,3
4,5,6*\)
===
|a|b|c|
|-|-|-|
|1|2|3|
|4|5|6|
{| class="wikitable"
!a
!b
!c
|-
|1
|2
|3
|-
|4
|5
|6
|-
|}
<table><thead><tr><td>a</td><td>b</td><td>c</td></tr></thead><tbody><tr><td>1</td><td>2</td><td>3</td></tr><tr><td>4</td><td>5</td><td>6</td></tr></tbody></table>
```

#### update

Update storage with given arguments

```
$update(arg1, arg2)
===
```

#### extract

Extract storage content.

```
$extract()
===
```

#### regcsv

feature: cindex

Register a csv as a table. Registered table can be queries with query macro.

```
$regcsv+(table_name,a,b,c
1,2,3)
===
```

#### dropcsv

feature: cindex

Drop a csv table

```
$dropcsv(table_name)
===
```

#### query

feature: cindex

Qeury a registered csv table with a statment.

Query implmentation uses [cindex](https://github.com/Simhyeon/cindex). Which
supports sql-like queries that is kind of a subset of SQL.

```
$query(SELECT * FROM table_name WHERE a = 1)
===
a
1
```

#### flowcontrol

```exit``` and ```escape``` changes flow of the processor behaviour. However
these flow control doesn't mean direct exit. Rather a signal to processor so
that processor can stop the work gracefully. Thus, ```exit``` inside macro
definition will stop a processor only after the processing is finished.

**exit**

```
---Before---
$exit()
---After---
===
---Before---
```

Exit stops processing at the given point of macro invocation.

**escape**

```
---Before---
$escape()
$exit()
---After---
===
---Before---
$exit()
---After---
```

Escape simply escapes all texts after macro call. Which is similar to pause but
you cannot revert the escape. Simply said, escape is one way around macro.

#### panic

```
Before
$panic(This was panicked because...)
After
===
error: Panic triggered with message
= This was panicked because...
 --> stdin:2:2
=== Processor panicked ===
```

#### Clear

Clear volatile macros. Volatile macros are macros that defined in hygiene mode.

```r4d
$clear()
===
```

#### Hygiene

Toggles hygiene's macro mode. Within hygiene mode runtime macros are volatile and
newly defined macros is removed when macro invocation ends.

Refer [modes](./modes.md) for more hygiene modes which can be toggled in
library.

```r4d
$hygiene(true)
$hygiene(false)
===
```
