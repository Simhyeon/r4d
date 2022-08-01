# Notice

These internals are all based on 3.0 version.

# Table of Contents

- [R4d recursively finds and expands
macros](#r4d-recursively-finds-and-expands-macros)
- [Nature of macro types](#nature-of-macro-types)
- [Runtime macro](#Runtime-macro)
- [Function macro](#function-macro)
- [Deterred macro](#deterred-macro)
- [Errors](#errors)
		

# R4d recursively finds and expands macros

R4d's main logic is a method called ```process_buffer```. The method iterates
lines from a given buffer and checks if matching macro syntax is detected. If
texts are not part of macro syntax, r4d simply prints the line without any
processing.

When a macro's syntax is detected, r4d saves partial data as a macro fragment.
If the macro fragment becomes complete without any exit condition, then r4d
tries to expand the macro. R4d basically indexs macro name from internal
hashmap which contains all information about macros. If no such entry was
found, r4d yields error. After a macro name and a arguements was successfully
retrieved, r4d first expands macro's argument. And of course, the arguments can
have macro syntax inside. Therefore the expansion process is a recursive
procedure.

```txt
$macro_name(       -> On dollar character, a framgent start
    Text argment   -> Values are all saved into a framgent
)                  -> A fragment completes on ending parenthesis
```

Expanded arguments' direction differs by macro types. If a macro is a runtime
macro, the arguments are split by a length which is defined by a user, and then
maps each into a local macro with a name from parameters. For a function macro,
burden of argument spliting is transferred to each function because the length 
is not defined at constnat time. Deterred macro is also not different.

```r4d
$define(macro,a b c=$a() $b() $c())
$define(arg=1,2,3)
$macro($arg())

% Expanded arguments are mapped to parameters
% $arg() == 1,2,3
%           | | |
%           a b c
```

Unlike other macros local macro is not expanded on invocation, because local
macros are mapped with expanded arguments. This might not look reasonable in
some cases but very much plausible. If a local macro was expanded, then macro
arguments will consequently doubly expanded. 

# Nature of macro types

A runtime macro is technically an expressive and featureful format macro. A
runtime macro defines a final output as a macro body with punch holes. Those
punch holes, or namely other macros, are expanded on runtime, thus called a
runtime macro.

In contrast, function macros are higher wrapper around function pointers. R4d
is loaded with many built-in macros which means there are equal amount of
functions mapped to those macros. Since function macro is a rust function, it
can extensively benefit from rust's functionality or ecosystem.

A deterred macro, works similary in a sesne that it is mapped to a function
pointer. However it's internal logic is quite different. A deterred macro
prevents expansin of arguments and gladly bears a burden to expand by itself.
In exchange of complexity, deterred macro gains a power to optionally expand
arguments or even dynamic contents. 

# Macro expansion

## Runtime macro

After arguments are mapped to each parameters, a body of a runtime macro is
then expanded. Since body of definition is not expanded on declaration, local
macro is expanded properly. The expanded body is returned to invocated
position. Runtime macro's body is a type of ```String``` which means that a
runtime macro cannot return nothing but always a container with empty value.
For the reason, a runtime macro that seemingly returns nothing leaves a newline
in its place.

```txt
% Test leaves empty line
% while define leaves nothing in its place
$define(test=)
$test()
===

```

## Function macro

Function macro's expansion is totally dependent on mapped function's behaviour.
For example, regex macro splits arguments with comma and treat first argument
as expression, second argument as substitute text, and finally third argument
as a source text to process. Regex expression is compiled into a regex program,
or retrieved from cache if the expression was compiled before. And finally
function redirects each components to regex's replace methods. Technically
regex macro does the following pseudo code.

```rust
let (expr,sub,source) = split_and_get_arguments(&args);
let regex = try_get_cache_or_compile(&expr);
let result = regex.replace_all(source,sub)

return Some(result);
```

After the macro function does its job, it returns an option of string. Which
can be either ```Some(value)``` or ```None```. Therefore function macro can
return namely null value. In turn, function macro can leaves nothing behind.

```r4d
% Sequences of macros leaves literally nothing
$define(test=)
$clear()
$rename(test,TEST)
$undef(TEST)
===
```

Meanwhile, a standard argument split procedure strips literal quotes from given
arguments. So that a function macro can process arguments' pure form without
unncessary literal quotes. The same rule is applied to deterred macro.
Arguments are expanded and then stripped regardless of expansion order. (
Although there are special cases like que macro )

## Deterred macro

Deterred macro has more powers over simple function macros. Deterred can
capture context of a macro was invoked. Such ability enables deterred macro to
determine whether a macro is valid in a context of invocation nested level,
expand a dyanimc expression which is created inside of a function, and
retreive information about parent's local macro. There is no way for a function
macro to understand a local macro, while deterred macro can directly index,
add, and modify them. A macro like strip can even curve a default behaviour of
"expand & strip" and do "strip & expand". 

Deterred macro can decide an order of expansion. Every if macro variants are
deterred macro, because it needs to optionally expand macro arguments. ```If```
macro will epxand other arguments only when given conition is met. If the macro
was implemented as a function macro, the expression would have been executed
anyway which is not desirable for user experience. This is especially
troublesome when the expression includes operating macros such as include or
fileout.

As stated above, deterred macro can register a new local macro. ```For``` macro
variants gets a benefit from this functionality. ```For``` macro has a common
denominator of splitting contents. A rule varies by a macro, but all of them
creates an iterable object. After split, the macro registers a local macro
named ```:``` and expand a given body text. This is the reason why users can
use an undefined dynamic local macro inside for macro variants' body argument.
Think of it as a capture group of regular expressions.

Dynamic expression "can" be expanded by a function macro, but the macro cannot
capture a context of an invocation. Any expression can include local macros
which are scope specific and also not known to a function macro. As a result,
macros like exec or spread are deterred macro to enable a sane macro expansion. 

# Errors

There are two types errors in r4d. One is an unallowed logic error which is
decided by r4d processor. Such error wouldn't always panick a program but
either has a probability to or susceptible to unintended result. The other is
panicking error, which is mosty occured by internal usage of panickable
functions. Those errors are not handled by r4d because there are all sorts of
functions thus the erorr is simpled redirected with ? operator. Logic error
tries to express an error with details but panick errors sometimes can yield
errors that is not so helpful.
