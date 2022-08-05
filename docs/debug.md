# TOC

- [How to debug](#how-to-debug)
- [Backtrace](#backtrace)
- [Log macro](#log-macro)
- [Dry run](#dry-run)
- [Debug mode](#debug-mode)
- [Example](#example)
- [Logging](#logging)
- [Diff](#diff)
- [Assertion](#assertion)

### How to debug

### Backtrace

Set environment variable RAD\_BACKTRACE to see error log information.

```
$envset(RAD_BACKTRACE,true)
$define(demo=$sep($cut($nl(),3,1$nl()2)))
$demo()
===
error: Invalid argument
= Index out of range. Given index is "3" but array length is "2"
 --> [INPUT = test]:2:2 >> (MACRO = demo):0:2 >> (MACRO = sep):0:2
```

### Log macro

The easiest way is to use a log macro. Simply call a log macro with a value you
want to check and logger will print the message for you.

```r4d
$counter(ct)
$counter(ct)
$define(check=$log(Counter at the time is : $ct()))
$check()
===
% This is printed on console
% log: Counter at the time is : 2
%  --> test:4:2~~
```

Or you can use logm macro to print unexpanded body of a given macro. Function
acro or deterred macro cannot be logged because it has no concept of body.

```r4d
$define(cont=)
$append(cont,\*$path(a,b,c)*\)
$append(cont,\*$evalk(1 + 2 )*\)
$cont()
% When user want to check the original body
$logm(cont)
===
a/b/c
1 + 2 = 3
% Printed on console
% log: $path(a,b,c)
% $evalk(1 + 2)
%  --> test:5:2
```

### Dry run

You can dry run macros for checking if macro expressions are mostly in sane
shape. In dry run mode, every function macro is silently ignored and processor
checks macro's body on declaration. Also every invalid macro name error will be
interpreted as warning because function macro is not expanded.

Dry run with ```--dryrun``` flag.

```r4d
% Typo in local macro usage
$define(Test,a_1 a_2=$a1() $a2() $fileout(Insufficient arguments))
% Typo in macro name
$test()
===
warning: Invalid macro name
= No such macro name : "a1"
 --> [INPUT = demo]:1:2 >> (MACRO = Test):1:2
warning: Invalid macro name
= No such macro name : "a2"
 --> [INPUT = demo]:1:2 >> (MACRO = Test):1:8
warning: Invalid macro name
= No such macro name : "test"
 --> [INPUT = demo]:2:2
```

#### Debug mode

Start a debug mode with ```-d``` or ```--debug``` flag.

I highly recommend using only file input for debugging because stdin doesn't
work properly.

Piping doesn't work because pipe creates unterminated bufstream. This is
because debug mode opens stdin and tries to get content until eof, but there is
no eof. Please submit an issue if you know how to curve this behaviour.

You can set $BP() macro inside a file to set a breakpoint for debugging.

```bash
# Debug mode
rad -d
rad --debug <FILE>

# Interactive mode
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
step       (s)     : Next macro including nested ( Currently very buggy don't use )
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
% This is a break point
$BR()

$arg(Hello World)

$arg(
    $test()
)
End of file
```
Following is a sequences of user input and program output.
Text After ```//``` is usaed as a comment for demonstration purpose and should
not be in included in real usage.

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

#### Logging

Logging flag sets processor to print every macro invocation's information.

You can save log to specific file with flag ```-e ErrorFile```.

e.g.
```
5:log
Name    = "evalk"
Attr    =
Greedy  : false
Pipe    : false
Literal : false
Trimmed : false
Args    = "1 + 2 "
---
```

#### Diff

Diff flag, ```--diff``` , prints difference between source input and processed output

You can save diff to specific file with flag ```-e ErrorFile```.

d.g.
```
DIFF :
- $define(author=Simon Creek)
- $define(title=R4d demo)
  ---
- title  : $title()
- author : $author()
+ title  : R4d demo
+ author : Simon Creek
  ---
- My name is $author() and I made r4d to make macros can be used within various
- forms of texts. This article was written in $date() $time().
+ My name is Simon Creek and I made r4d to make macros can be used within various
+ forms of texts. This article was written in 2022-07-08 22:12:21.

- $ifdef(test, This should be only printed when I'm testing not in release)$dnl()
  This is some important table automatically formatted according to environment
  variable.

- $regcsv(addr,$include(addr.csv))$dnl()
- id,first name,last name,address
- 1,John,doe,AA 1234
- 2,Janet,doner,BB 4566
- 3,Hevay,jojo,CC 8790
- $static(
-     queried,
-     $query(
-         SELECT id,'first name',address
-         FROM addr where 'first name' = John
-     )
- )$dnl()
- % Comments are disabled by default for better compatibility
- % TABLE_FORM == github
- $table($env(TABLE_FORM),$queried())
+ |1|John|AA 1234|
+ |-|-|-|

- $wrap(40,$lipsum(15))
+ Lorem ipsum dolor sit amet consectetur
+ adipiscing elit. In rhoncus sapien
+ iaculis sapien congue a

- Evaluation : $prec($eval( $num(0.1second) + $num(0.2sekunde)),2)
- Evaluation : $evalk( 1 + 2 )
+ Evaluation : 0.30
+ Evaluation :  1 + 2 = 3
```

#### Assertion

Sometimes you want an assertion for debugging. R4d comes with several assertion
macros.

Basic usages are followed.

```
% Every assertion yields error on fail
% Passes when lvalue == rvalue
$assert($test(),Test)
% Passes when lvalue != rvalue
$nassert($test(),Test)
% Passes when inner expression panicks
$fassert($eval(a + b))
```

You can use assertion mode with ```--assert``` flag which prevents assertion
panics and prints assertion result after processing.

```
% Ran with rad <INPUT> --assert
$assert(test,test)
$assert(test,test1)
$nassert(test,test)
$nassert(test,test1)
$fassert(Test)
$assert($eval(a + b))
===
assert fail -> test:2:2
assert fail -> test:3:2
assert fail -> test:5:2
error: found 5 errors

Assert
SUCCESS : 2
FAIL: 3
```
