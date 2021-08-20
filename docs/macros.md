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

Define creates an custom macro.

```
$define(name,a1 a2,"$a1(),$a2()")
===
// Define doesn't print new line if it is a single input in the line
```
**undef**

Undef can undefine every macros including basic(default) macros. However ```define```
cannot be undefined.

```
$undef(name)
===
// Undef doesn't print new line if it is a single input in the line
```

**include**

Include macro include given file and paste into the position. Included file's
contents are all expanded.

```
$include(src/content.rs)
===
// Content of src/content.rs is pasted in here
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
$foreach("a,b,c",Value: $_
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
$forloop("3,5",Number: $_
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

Call system command, on unix system it calls ```sh -c``` and ```cmd /C``` on
windows system.

```
$syscmd("uname -a") 
$syscmd(ver)
===
Linux

Microsoft Windows [Version 10......]

```

**rsub, rdel**

Regex substitution and regex deletion gets source and additional arguments to
process regex operation. Second argument is regex expression. This use [regex
crate](https://crates.io/crates/regex).
```
$rsub(Hello world,w.*?d,rust)
$rdel(Hello World// TODO,//.*$)
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
formats are ```github``` and ```wikitext```. This macro doesn't pretty print
but just make it readable from other programs.

```
$table(github,"a,b,c
1,2,3
4,5,6")
$table(wikitext,"a,b,c
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
```