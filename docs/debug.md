### How to debug?

#### Debug mode

Simply start debug mode with ```-d``` or ```--debug``` flag.

I highly recommend using only file input for debugging because stdin doesn't
work properly.

Piping doesn't work because pipe creates unterminated bufstream. This is
because debug mode opens stdin and tries to get content until eof, but there is
no eof. Please submit an issue if you know how to curve this behaviour.

Stdin requires EOF in the end. Which is Ctrl^D in linux and Ctrl^Z in windows.
Type as much text as you want and press EOF to end input stream.

```bash
# Debug mode
rad -d
rad --debug <FILE>

# Log mod, which prints all macro invocation info into terminal
# this will be explained later
rad --log

# Interactive mode, like a game you know.
# This disables text wrapping 
rad --debug -i
===
...
# After Some standard input and EOF
(stdin) : Default is next. Ctrl + c to exit.
>>
```

**Basic usage**

```
help       (h)     : Print this help
next       (n,\n)  : Next line
macro      (m)     : Next macro
step       (s)     : Next macro including nested
until      (u)     : Next macro but before invocation
continue   (c)     : Next break point
clear      (cl)    : Clear terminal
print      (p)     : Print variable

    - name (n)     : Print current macro name
    - arg  (a)     : Print current macro's argument (not necessarily complete)
    - text (t)     : Print current line text
    - line (l)     : Print current line number
    - span (s)     : Print span of lines until the line
```

### Example

**Original file**
```
Non macro texts are ignored unless newline
$define(test=Test)
$define(arg,a_arg=Arg is : $a_arg())
1
2
3
4
5
$BR() // This is a break point

$arg(Hello World)

$arg(
	$test()
)
End of file
```
Following is a sequences of user input and program output.
Text After ```//``` is a comment and should not be in included in real usage

```
(filename) : Default is next. Ctrl + c to exit.
>> macro      // == m
Non macro texts are ignored unless newline
(macro) : $define(test=Test)
>> print line // == p l
(output) : 2
>> print name // == p n
(output) : define
>> print arg  // == p a
(output) : test=Test
>> print text // == p t
(output) : $define(test=Test)
>> continue   // == c
1
2
3
4
5
(line) :
>> print line
(output) : 10
>>

(line) : $arg(Hello World)
>> print name
(output) :
>> print text
(output) : $arg(Hello World)
>> step
(macro) : $arg(Hello World)
>> print name
(output) : arg
>> print arg
(output) : Hello World
>> step
Arg is : Hello World

(step) :        $test()
>> step
(macro) : $arg(
>> print span // p s
(output) :
$arg(
        $test()
)
>>
Arg is :
        Test

(line) : End of file
>>
End of file
```

You can also clear termianl with command clear(cl in short).

#### Loggig

Logging flag sets processor to print every macro invocation's information.

You can save log to specific file with flag ```-e ErrorFile```.

e.g.
```
4:log
Name    = "ifdef"
Attr    =
Greedy  : false
Pipe    : false
Literal : false
Trimmed : false
Args    = "define,Define is defined of course"
---
```

#### Diff

Diff flag prints difference between source input and processed output

You can save diff to specific file with flag ```-e ErrorFile```.

d.g.
```
DIFF :
- $define(author=SimonCreek)
- $define(title=R4d demo)
  ---
- title : $title()
- author : $author()
+ title : R4d demo
+ author : SimonCreek
  ---
- My name is $author() and I made r4d to make macros can be used within various
- forms of texts. This article was written in $date() $time().
+ My name is SimonCreek and I made r4d to make macros can be used within various
+ forms of texts. This article was written in 2021-10-03 03:05:26.

- $if($ifdef(test), This should be only printed when I'm testing not in release)

  This is some important table automatically formatted according to environment
  variable.

- $table($env(TABLE_FORM),\*H1,H2,H3
- a,b,c
- d,e,f*\)
+ |H1|H2|H3|
+ |-|-|-|
+ |a|b|c|
+ |d|e|f|

- I'm out of idea and I need some texts, $lipsum(15)
+ I'm out of idea and I need some texts, Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore.
```
