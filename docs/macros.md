#### Format

```
$macro_invocation(...)
===
Expanded text from macro
// As some comments
```

#### Basic(default) macros

All macros are case sensitive and should come with dollar sign prefix.

**define**

Define creates an custom macro. This macro is actually not a macro but special
function. Define cannot be renamed or undefined. Define macro cannot be
overriden too.

```
$define(name,a1 a2="$a1(),$a2()")
===
// Define doesn't print new line if it is a single input in the line
```
**undef**

Undef can undefine every macros including basic(default) macros. However
```define``` cannot be undefined.

```
$undef(name)
===
// Undef doesn't print new line if it is a single input in the line
```

**rename**

Rename can change the name of the macro. This applies both to basic and custom
macro. You cannot rename define.

```
$rename(len,length)
$length(I'm long)
===
8
```

**append**

Append append given string into the macro. Only
custom macro can be appended.

```
$define(test=TEST)
$append(test, CASE)
$test()
===
TEST CASE
```

**pause**

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

**include**

Include macro include given file and paste into the position. Included file's
contents are all expanded.

```
$include(src/content.rs)
===
// Content of src/content.rs is pasted in here
```

**temp**

Tempin gets content from temp file named ```rad.txt```. Macros within temp file
is also expanded. Tempout pushes content into current temp file. You can also
change the temp file with tempto.

Temp file is saved in ```%TEMP%``` in Windows and ```/tmp``` in *nix systmes.
```
$tempout(Hello world)
$tempin()
$tempto(out.json)
$tempout({"name":"simon creek"})
$tempin()
===
Hello world
{"name":"simon creek"}
```

**redir**

Redirect all input into a temp file.

```
$redir(true)
$foreach(\*1,2,3*\,Value: $:
)
1,2,3,4,5
$redir(false)
===
// Yield nothing regardless of -o option
// Content is saved to current temp file.
```

**fileout**

Fileout saves contents to the file. If truncate is false, non existent file
argument is panic behaviour.

```
$fileout(true,file_name.txt,Hello World)
$fileout(false,file_name.txt,This is appended)
===
```

**env**

Print environment variable. Wrong name yields error

```
$env(HOME)
===
/home/simoncreek
```

**path**

Merge two path into one.

```
$path($env(HOME),document)
===
/home/simoncreek/document
```

**bind**

Bind macro makes new local macro inside definition. This macro is automatically
clared after evalution of the macro.

```
$define(test,a\_src a\_content=
$bind+(source,$path(cache,$a\_src()))
$fileout(false,$source(),$a\_content())
)
$test+(temp,Hello World)
===
// Now ./cache/temp file contains string "Hello World"
```

**pipe, -, \* **

Pipe macro simply saves value to pipe. $-() returns piped value and clears
pipe. $*() returns piped value in literal form.

```
$pipe(Value)
$-()
$-() 
$pipe(Value)
$*()
$*() 
===
Value

\*Value*\
\**\
```

**Repeat**

Repeat given content for given times
```
$repeat(3,Content to be repeated
)
===
Content to be repeated
Content to be repeated
Content to be repeated

```

**foreach**

Loop around given value. Value is separated with commas. Thus values should be
always enclosed with double quotes.

```
$foreach(\*a,b,c*\,Value: $:
)
===
Value: a
Value: b
Value: c

```

**forloop**

Loop around given range. Value is separated with commas. 

Range is inclusive e.g. 1 and 3 means from 1 to 3.

```
$forloop(3,5,Number: $:
)
===
Number: 3
Number: 4
Number: 5

```

**forloop, foreach with nested macro usage**

Forloop and foreach's ```$:``` is a substituion rather than macro expansion.
And first of all rad doesn't create any form of AST from given source thus, $:
is expanded at the last stage.

```
$define(styles,a_styles=<style>
$foreach($a_styles*(),
$include($path($env(GDE_MODULE),$:.css)))
</style>
)
$styles(\*a,b,c*\)
=== 
// This fails because include tries to get path of $GDE_MODULE/$:.css
```
Thus enclose the body with literal rule which will deter evaluation for a time.

```
$define(styles,a_styles=<style>
$foreach($a_styles*(),
\*$include($path($env(GDE_MODULE),$:.css))*\)
</style>
)
$styles(\*a,b,c*\)
===
// This will execute macro of 
// $include($GDE_MODULE/a.css) 
// $include($GDE_MODULE/b.css)
// $include($GDE_MODULE/c.css)
```

**eval**

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

**ifelse**

Ifelse gets two branches and print out one according to given condition.

Text "true" and "false", non "0" integer and "0" are valid inputs. "true" and
"false" is case sensitive. 0 is false and any number other than 0 is true even
negative integer is valid input. Floating point number is not allowed.

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

**ifdef**

Check if given macro is defined or not.

```
$define(some=value)
$ifdef(some)
$undef(some)
$ifdef(some)
===
true
false
```

**syscmd**

Call system command, on unix system macro calls given command directly. While
windows call are mediated through ```cmd /C``` call.

```
$syscmd(uname -a) 
$syscmd(ver)
===
Linux

Microsoft Windows [Version 10......]

```

**sub**

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

**tr**

Tr translate characters to other characters. Utf8 characters work.

```
$tr(Given String,iSg,aOs)
===
Gaven Otrans
```

**len**

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

**regex**

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

**trim, chomp, comp**

```Trim``` removes preceding and trailing new lines, tabs and whitespaces from
given input. ```Chomp``` removed duplicate newlines from given input.
```Comp``` both trim and chomp given input

There are variatins with suffix "l" which yield literal output. ```triml,
chompl, compl```

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

**nl**

Simply print out "newline" characters. This newline respects formatter's
newline. Which is ```\r\n``` for windows and a ```\n``` in *nix systems by
default.

```
$nl()
===

// This is useful when you want to construct an output in one-liner
```

**lipsum**

Lipsum creates a placehoder with given word counts. This requires features
"lipsum".

```
$lipsum(5)
===
Lorem ipsum dolor sit amet.
```

**time, date**

Time and date prints current local time and date. This requires features
"chrono".

```
$time()
$date()
===
03:17:11
2021-08-20
```

**from**

From creates formatted macro invocations with given csv values. The given macro
name doesn't need dollar sign prefix. This requires features "csv".

```
$define(three,a1 a2 a3=1-$a1(), 2-$a2(), 3-$a3())
$from(\*a,b,c
d,e,f*\,three)
===
1-a, 2-b, 3-c
1-d, 2-e, 3-f
```

**table**

Table creates a formatted table from given csv values. Currently supported
formats are ```github```, ```wikitext``` and ```html```. This macro doesn't
pretty print but just make it readable from other programs. This requires
features "csv".

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
