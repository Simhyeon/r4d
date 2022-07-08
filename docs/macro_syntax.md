# TOC

* [Macro definition](#macro-definition)
  * [Caveats](#caveats)
* [Macro invocation](#macro-invocation)
* [Literal rules](#literal-rules)
* [Comments](#comments)
* [Macro attributes](#macro-attributes)
* [Errors](#errors)
* [Break point](#break-point)

## Macro definition

Definition syntax is similar to macro invocation but requires a specific form
to sucessfully register the macro.

```
$define(name,arg1 arg2=$arg1() $arg2())
        |    |         |
Arg:    1st  2nd       3rd
```
**First argument** (Before first comma)

First argument is a macro name. Macro should start with an alphabet character
and following characters should be either alphanumeric or underscore.

**Second argument** (Before equal sign)

Second argument is macro's arguments. Macro argument also follows same rule
of naming. Multiple arguments can be declared and should be **separated by a
whitespace.**

**Third argument** (After equal sign)

Third argument is a macro body. Any text can be included in the macro body
while an unbalanced parenthesis will prevent processor's work. Currently there
is no way to include unbalanced parenthesis inside definition body. 

You can also simply bind the value without giving arguments. Which is mostly
similar to static macro.

```
$define(v_name=Simon creek)
% Which is same with
$static(v_name,Simon creek)
```

### Caveats

**Define is not evaluated on declaration**

Definition's body can include any macro invocation in itself, but wrong macro
use inside definition cannot be detected at the time of definition. By other
terms, defined macro is evaluated lazily.

```
$define(panik,kalm=$calm())
$panik()
$define(calm=KALM)
$panik()
===
error: Failed to invoke a macro : "calm"
 --> stdin:2:2
$calm()
KALM 
% After calm is defined, it prints out without error
```

**Define is evaluated on every call**

Because defined macro is evaluated on every invocation. This may not be a
desired behaviour. Use static macro if you want statically bound value. Static
macro eagerly evaluates arguments and assign the processed value to a macro.

```
$define(counter=0)
$define(print_counter=$counter())
$static(print_counter_as_static,$counter())
$append(counter,0000)
$print_counter()
$print_counter_as_static()
===
00000
0
```

## Macro invocation

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

Special macro ```$:()``` is used for iterated value.
```
$foreach(\*John,Simon,Jane*\,Name : $:()
)
$forloop(5,10,$:()th
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
non-escaped comma will change the number or content of arguments. If desirable
content includes unbalanced parentheses or commas, enclose the body with string
literal with the syntax of ```\* TEXT GOES HERE *\```, by the way literal syntax
inside macro body will printed as is. 

## Literal rules

Literal rules in r4d is unfortunately not straightforward at a first glance.
General rules are followings

- Literal text inside definition body is not stripped on execution
- Literal text as an argument is expanded and then stripped
	- Argument without macro enclosed with literal text is treated as is without
	breaking argument length
	- Argument with macro enclosed with literal text will treat the macro
	invocation as normal text without it expanded

In short, rad processes macros in given subprocesses.

- Expand expression from arguments
- Strip expanded arguments
- Bind arguments to local macros
- Expand a macro body

See an examples to better understand literal rules

### Macro body

```r4d
$define(test=\*$path(a,b)*\)
$test()
===
% Macro body is not stripped
\*$path(a,b)*\
```

### Literl inside arguments

**Literal without macros**

```r4d
$static(p1,$path(\*a*\,b,c))
$static(p2,$path(a,b,c))
$assert($p1(),$p2())
===
% This holds true because \*a*\ is tripped to be a value of "a"
% Thus path macro can process argument as if it was originally a "a,b,c"
```

**Literal with macros**

```
$static(p,a/b)
$static(p1,$path(\*$p()*\,b,c))
$static(p2,$path($p(),b,c))
$logm(p1)
$logm(p2)
$assert($p1(),$p2())
===
% p1's argument was stripped but not expanded
% log: $p()/b/c
%  --> test:4:2
%
% log: a/b/b/c
%  --> test:5:2 
% Final result of p1 and p2 is same to end user
```

**Literal passed as arguments**

```
$define(demo,a_first=$if($a_first(),TRUE))
$demo($not(false))
$demo(\*$not(false)*\)
===
TRUE
error: Invalid argument
= If requires either true/false or zero/nonzero integer but given "$not(false)"
 --> test:3:2~~
```

If the argument is passed as literal, local argument will be linked to stripped
but non-expanded value. which is ```$not(false)``` in this case. Since local
argument is not expanded, if receives strange value and execuion fails.

To prevent this error, minimize literal usage if possible. When passing array
use literal quote only when the values are constants. If you need to create a
dynamically created array, wrap it inside a macro and use literal attribute.

```r4d
$define=(
		arr_pass,
		a_arr a_it=
		$foreach=(
			$:() + $a_it()$nl(),
			$a_arr()
		)
)
$static(array,a,b,c,d,e,f,g)
$arr_pass^($array*(),@)
===
a + @
b + @
c + @
d + @
e + @
f + @
g + @
```

Simple process how this works

- **arr_pass**
	- Expand $array() -> ```a,b,c,d,e,f,g```
	- Try stripping, but nothing to do anyway
	- Wrap it inside literal (attirbute) -> ```\* a,b,c,d,e,f,g *\```
	- Bind the wrapped value to an argument ```a_arr```
- **foreach**
	- Expand $a_arr() -> ```\* a,b,c,d,e,f,g *\```
	- Strip a value -> ```a,b,c,d,e,f,g```
	- Bind the value to an argument ```a_body``` ( Refer --man foreach )

## Comments

Comment is disabled by default because comment character can intefere with
macro expansion and user expectance. You can enable comment mode with
```--comment``` flag.

There are three types of comment mode. Those are none,start and any. None is
the default and ```--comment``` is same with ```--comment start```.
```--comment any``` enables comments for any positions.

Default comment character is ```%```. If you have used LaTex before, it would
be familar. This can be configured with builder method.

```
% This is a valid comment on both start and any mode
Prior content goes here  % This is only valid comment on any mode.
```

### Macro attributes

**Trim output**

Trim output attribute ```^``` trims preceding and following newlines,
whitespaces from output.

```
$define(
	test
	=
	Hello
	World
)
$test()
$test^()
===

        Hello
        World

Hello
        World
```

**Trim input**

Trim input attribute ```=``` trims macro arguments by lines and also trim by
chunk. This is useful when you want to use a multiline complex text as
arguments but surplus blank spaces are unnecessary. Trim inputs power is mostly
centered on single argument mcros but other situations are also plausible.

Trim input can be applied to define macro and trimming is applied to macro
body.

```
% Needs trim output to remove newline that comes before "hello world"
$ifenv^=(HOME,
    hello world
    How are you?
    I'm fine, thanks. How's it going?
    yatti yatta
)
===
hello world
How are you?
I'm fine, thanks. How's it going?
yatti yatta
```

Since "trim input" trimes input not arguments, trimmed input can be different
from expectation.

e.g)
```
$macro_name=(
	first,
	second
)
===
% Arguments are passed as 
% first
% 
% second
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

A caveat with piped values

```
$eval|("test" == "test")
$define(result=$-())
$result()
$result()
===
true

```

Result calls pipe macro not saves piped value into itself, thus using result
second time will yield nothing.

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
% This is because foreach evaluate expression once more and "*\)" doesn't have
matching "(" character.
```

### Errors

Every error is panicking by default(strict mode), to make programs more stable
and expectable or because I'm too rusty person. You can disable strict mode
with lenient option ```-l or --lenient``` or puge option ```-p or --purge```.

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

**Position of character is not accurate somtime**

Welp this is because, r4d doesn't construct AST and processes macros with
stack. Thus position of characters can really vary.

Plus, macro expanded text can have totally different length, content from
original text. Thus tracking thoses offsets are not worth the hassel needed.

In conclusion, character in error or warning messages can be not correct but
might be an indicator, which is represented as arrow ```->```.

### Break point

```BR``` is reserved for debugging usage. You cannot override breakpoint.

```
$BR()
```

Using BR macro outside debug mode is not an error but warning.
