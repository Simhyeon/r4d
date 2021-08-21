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
function. Define cannot be renamed or undefined. Or even paused(Which will be
added later) Define macro cannot be overriden too.

```
$define(name,a1 a2,"$a1(),$a2()")
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

Pause literally pauses every macro execution except pause macro.

```
$pause(true)
$define(some,a,$a())
$eval(1 + 2)
$pause(false)
$define(some,a,$a())
$eval(1 + 2)
===

$define(some,a,$a())
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

Temp saves content to temp file named ```rad_temp.txt```. Temp file is saved in
```%TEMP%``` in windows and ```/tmp``` in non windows system. First argument is
whether to truncate the file content.

```
$temp(true,Hello world)
$include(/tmp/rad_temp.txt)
===
Hello world
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
$foreach("a,b,c",Value: $:
)
===
Value: a
Value: b
Value: c

```

**forloop**

Loop around given range. Value is separated with commas. Thus values should be
always enclosed with double quotes.

Range is inclusive e.g. "1,3" means from 1 to 3.

```
$forloop("3,5",Number: $:
)
===
Number: 3
Number: 4
Number: 5

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
$ifelse( true ,"I'm true","I'm false")
$ifelse( false ,"I'm true","I'm false")
$ifelse( 1 ,"I'm true","I'm false")
$ifelse( 0 ,"I'm true","I'm false")
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

Call system command, on unix system it calls directly. While windows call are
mediated through ```cmd /C``` call.

```
$syscmd("uname -a") 
$syscmd(ver)
===
Linux

Microsoft Windows [Version 10......]

```

**sub**

Sub gets substring from given input range. 

```
$sub("1,5",123456789)
$sub("2,",123456789)
$sub(",6",123456789)
===
2345
3456789
123456
```

**tr**

Tr translate characters to other characters.

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
$regex(Hello World// TODO,//.*$,"")
===
Hello rust
Hello World
```

**trim, chomp, comp**

```Trim``` removes preceding and trailing new lines, tabs and whitespaces from
given input. ```Chomp``` removed duplicate newlines from given input.
```Comp``` both trim and chomp given input

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

**lipsum**

Lipsum creates a placehoder with given word counts.

```
$lipsum(5)
===
Lorem ipsum dolor sit amet.
```

**time, date**

Time and date prints current local time and date.

```
$time()
$date()
===
03:17:11
2021-08-20
```

**from**

From creates formatted macro invocations with given csv values. The given macro
name doesn't need dollar sign prefix.

```
$define(three,a1 a2 a3,"1-$a1(), 2-$a2(), 3-$a3()")
$from("a,b,c
d,e,f",three)
===
1-a, 2-b, 3-c
1-d, 2-e, 3-f
```

**table**

Table creates a formatted table from given csv values. Currently supported
formats are ```github```, ```wikitext``` and ```html```. This macro doesn't
pretty print but just make it readable from other programs.

```
$table(github,"a,b,c
1,2,3
4,5,6")
$table(wikitext,"a,b,c
1,2,3
4,5,6")
$table(html,"a,b,c
1,2,3
4,5,6")
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
