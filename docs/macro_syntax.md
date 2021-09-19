#### Macro definition

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
of naming. Multiple arguments can be declared and should be separated by a
whitespace.

**Third argument** (After equal sign)

Third argument is a macro body. Any text can be included in the macro body
while an unbalanced parenthesis will prevent processor's work. Currently there
is no way to include unbalanced parenthesis inside definition body. 

You can also simply bind the value without giving arguments.

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

**Escape characters**

Defnition body can include any text except unbalanced parenthesis. Even escape
character is saved into body as it is.

However escape characters are escaped in arguments. Use double escape chraacter
to use literal escape character.

```
$define(test,a_arg=Inside arg
$a_arg()
while body's escape is '\')
$test(First escape is : '\'
Second escape is '\\')
===
Inside arg
First escape is : ''
Second escape is '\'
while body's escape is '\'
```

#### Macro inovocation

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
non-escaped comma will change the number or content of arguments. If desirable
content includes unbalanced parentheses or commas, enclose the body with string
literal with the syntax of ```\* TEXT GOES HERE *\```, by the way literal
inside macro body will printed as is. 
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
// Easier alternative is greddy attribute in this case
$repeat+(2,I'm,comma,separated)
===
I'm,comma,separated
I'm,comma,separated
// Result is same
```
### Doublely evaluated macros

Some macros' arguments are evaluated twice such as ```foreach, forloop, if and
ifelse```.

Try enclose it with literal to prevent from unexpected behaviour.

e.g.
```
$if(true,\*$macro()*\)
$ifelse(true,\*$macro()*\,\*$macro()*\)
$foreach(\*1,2,3*\,\*$:*\)
$forloop(1,3,\*$:*\)
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
	=                                        // This new line
	$bind+(cond,$eval($a_expr()))            // Whitespaces before "$bind"
	$bind+(true_path,$path(cache,$a_path())) // Whitespaces before "$bind"
	$bind+(false_path,$path(cache,index.md)) // Whitespaces before "$bind"
	$ifelse(                                 // Whitespaces before "$ifelse"
		$cond(),
		$include($true_path()),
		$include($false_path())
	)                                        // This new line
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
