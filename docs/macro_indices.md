## Format

If any permission is necessary, it is displayed as

AUTH : (ENV|FIN|FOUT|CMD)

Macro expansion demonstration is displayed as

```
$macro_invocation(...)
===
Expanded text from macro // This is a demonstration comment and not a real comment
% As some comments
```

## Basic(default) macros

All macros are case sensitive and should come with dollar sign prefix.

For assertion macros refer [debug part](./debug.md) 

* [define](#define)
* [declare](#declare)
* [undef](#undef)
* [rename](#rename)
* [repl](#repl)
* [append](#append)
* [pause](#pause-keyword-macro)
* [include](#include)
* [read](#read)
* [temp](#tempin-tempout-tempto)
* [redir](#redir)
* [fileout](#fileout)
* [env](#env)
* [envset](#envset)
* [ifenv](#ifenv-keyword-macro)
* [ifenvel](#ifenvel-keyword-macro)
* [path](#path)
* [abs](#abs)
* [name](#name)
* [parent](#parent)
* [bind](#let-keyword-macro)
* [let](#let-keyword-macro)
* [Global](#static-keyword-macro)
* [Static](#static-keyword-macro)
* [pipe](#pipe)
* [Repeat](#repeat)
* [array](#array)
* [foreach](#foreach-keyword-macro)
* [forloop](#forloop-keyword-macro)
* [eval](#eval)
* [if](#if--keyword-macro)
* [ifelse](#ifelse--keyword-macro)
* [ifdef](#ifdef--keyword-macro)
* [ifdefel](#ifdefel--keyword-macro)
* [not](#not)
* [syscmd](#syscmd)
* [sub](#sub)
* [tr](#tr)
* [len](#len)
* [regex](#regex)
* [trim, chomp, comp](#trim-chomp-comp)
* [nl](#nl)
* [lipsum](#lipsum)
* [time, date](#time-date)
* [from](#from)
* [table](#table)

### define

Define creates an custom macro. This macro is actually not a macro but special
function. Define cannot be renamed or undefined. Define macro cannot be
overriden too.

```
$define(name,a1 a2="$a1(),$a2()")
===
% Define doesn't print new line if it is a single input in the line
```

### declare

You can simply declare a macro or macros without defining its body.

```
$declare(name)
$declare(n1 n2 n3)
$ifdef(name,I'm defined)
$ifdef(n3,I'm also defined)
===
I'm defined
I'm also defined
```

### undef

Undef can undefine every macros including basic(default) macros. However
```define``` cannot be undefined.

```
$undef(name)
===
% Undef doesn't print new line if it is a single input in the line
```

### rename

Rename can change the name of the macro. This applies both to basic and custom
macro. You cannot rename define.

```
$rename(len,length)
$length(I'm long)
===
8
```

### repl

Replace contents of the custom macro.

```
$define(before=BEFORE)
$repl(before,AFTER)
$before()
```

### append

Append append given string into the macro. Only
custom macro can be appended.

```
$define(test=TEST)
$append(test, CASE)
$test()
===
TEST CASE
```

### pause (keyword macro)

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

### include

AUTH : FIN

Include macro reads given file and paste into the position. Included file's
contents are all expanded.

Include macro read a whole file's contents into a single string. This is an
intended behaviour so that nested include macro inside definition can respect
order of expressions. If you are using big chunk of data and you don't use
macros inside other declared macro, try use read macro which read files and
write on the way.

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

### read

"Read" include file's content but in a streamlined way. It include files
without saving to any memory. Use this macro when you read from huge file which
might affect memory usage but make sure macro is directly invoked or use some
detour.

```
$define(read_from,a_src=This go before
$read($a_src())
This go after)
$read_from(source.txt)
===
{{Read contents comes here}}
This go before
This go after
```

### tempin, tempout, tempto

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

### redir

AUTH : FOUT

Redirect all input into a temp file.

```
$redir(true)
$foreach(\*1,2,3*\,Value: $:
)
1,2,3,4,5
$redir(false)
===
% Yield nothing regardless of -o option
% Content is saved to current temp file.
```

### fileout

AUTH : FOUT

Fileout saves contents to a file. If truncate is false, non existent file
argument is panic behaviour.

```
$fileout(true,file_name.txt,Hello World)
$fileout(false,file_name.txt,This is appended)
===
```

### env

AUTH : ENV

Print environment variable. Non existent env varaible will yield warning on
strict mode.

```
$env(HOME)
===
/home/simoncreek
```

### envset

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
Processor panicked, exiting...
```

### ifenv (keyword macro)

AUTH: ENV

If environment variable is defined, execute expression

```
$ifenv(HOME,$env(HOME)) 
===
/home/username
```

### ifenv (keyword macro)

AUTH: ENV

If environment variable is defined, execute expression else execute another expression

```
$ifenvel(HOME,$env(HOME),No home is defined) 
===
% Home is most likely always defined
/home/username
```

### path

Join elements into a path.

```
$path($env(HOME),document)
$paths(a,b,c)
===
/home/simoncreek/document
a/b/c
```

### abs

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
Processor panicked, exiting...
```

### name

Get file name(last part) from input

```
$name(/home/test/Documents/info.txt)
===
info.txt
```

### parent

Get parent path from input

```
$parent(/home/test/Documents/info.txt)
===
/home/test/Documents
```

### let (keyword macro)

Macro ```bind```  is deprecated and will be removed in 2.0 version. Use ```let```instead.

Declares a new local macro. This macro is automatically clared after evalution
of the macro.

```
$define(test,a\_src a\_content=
$let+(source,$path(cache,$a\_src()))
$fileout(false,$source(),$a\_content())
)
$test+(temp,Hello World)
===
% Now ./cache/temp file contains string "Hello World"
% cannot reference "source" macro after macro execution
```

### static (keyword macro)

```global```is deprecated for intuitive naming. Global keyword will be removed
in 2.0 version. Use ```static``` instead.

Statically binds an expression that persists for the whole processing. Static
is useful when you don't need dynamic evaluation but statically bound value.
Because definition is evaluated on every call which might not be necessarily
efficient or not be an intended behaviour.

```
$define(test=$time())
$test()
$static(test=$time())
$test()
===
17:08:39 % This will yield different result according to time.
17:08:39 % This will always yield same result
```

Though, time will most likely print same thing for a single document
processing. Other operations might need consistent bound values.

### pipe

Pipe macro simply saves value to pipe. $-() returns piped value 
$-*() returns piped value in literal form.

```
$pipe(Value)
$-()
$*() 
===
Value
\*Value*\
```

### Repeat

Repeat given content for given times
```
$repeat(3,Content to be repeated
)
===
Content to be repeated
Content to be repeated
Content to be repeated

```
### array

Create comma separated array from given value. You can set custom delimiter as
second argument(default is single whitespace). You can also filter array with
regex expression in third argument.

```
$syscmd|^(ls)
$arr($-(),$nl())
$arr($-(),$nl(),\.sh$) // File that ends with .sh
===
auto.sh,Cargo.lock,Cargo.toml,oush
auto.sh
```

### foreach (keyword macro)

Loop around given value. Value is separated with commas. Thus values should be
always enclosed with double quotes. Iterated values are references with
```$:```.

```
$foreach(\*a,b,c*\,Value: $:
)
===
Value: a
Value: b
Value: c

```

### forloop (keyword macro)

Loop around given range. Value is separated with commas. Iterated values are
references with ```$:```.


Range is inclusive e.g. 1 and 3 means from 1 to 3.

```
$forloop(3,5,Number: $:
)
===
Number: 3
Number: 4
Number: 5

```
### eval

Eval evaluates expression. This macro(function) uses rust's evalexpr crate
[crate link](https://crates.io/crates/evalexpr). Therefore argument formula
follows evalexpr's syntax.

```
$eval(1+2)
$eval(0.1+0.2)
===
3
0.30000000000000004
```

### if (keyword macro)

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

### ifelse (keyword macro)

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

### ifdef (keyword macro)

If macro is defined then execute given expression.

```
$define(some=value)
$ifdef(some,Defined)
$undef(some)
$ifdef(some,Not defined)
===
Defined
```

### not

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

### syscmd

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

### sub

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

### tr

Tr translate characters to other characters. Utf8 characters work.

```
$tr(Given String,iSg,aOs)
===
Gaven Otrans
```

### len

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

### regex

Regex substitution and regex deletion gets source and additional arguments to
process regex operation. Second argument is regex expression. This use [regex
crate](https://crates.io/crates/regex).
```
$regex(Hello world,w.*?d,rust)
$regex(Hello World// TODO,//.*$,)
===
Hello rust
Hello World
```

### trim, chomp, comp

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

### nl

Simply print out "newline" characters. This newline respects formatter's
newline. Which is ```\r\n``` for windows and a ```\n``` in *nix systems by
default.

```
$nl()
===

% This is useful when you want to construct an output in one-liner
```

### lipsum

Lipsum creates a placehoder with given word counts. This requires features
"lipsum".

```
$lipsum(5)
===
Lorem ipsum dolor sit amet.
```

### time, date

Time and date prints current local time and date. This requires features
"chrono".

```
$time()
$date()
===
03:17:11
2021-08-20
```

### from

From creates formatted macro invocations with given csv values. The given macro
name doesn't need dollar sign prefix. This requires features **"csv"**.

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

### table

Table creates a formatted table from given csv values. Currently supported
formats are ```github```, ```wikitext``` and ```html```. This macro doesn't
pretty print but just make it readable from other programs. This requires
features **"csv"**.

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
