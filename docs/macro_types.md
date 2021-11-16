# TOC

- [Custom and basic](#custom-and-basic)
- [Why the name basic](#why-the-name-basic)
- [What is a keyword macro?](#what-is-a-keyword-macro-)
- [Local and global](#local-and-global)

## Custom and basic

Basic macro is a macro like function pointer or closure. While custom macro is
a macro that doesn't include complicate logics but only gets expanded according
to given rules.

## Why the name basic

Well, it is because basic macro was not configurable at first and given as
default macros. But r4d has evolved and even basic macro is configurable and
can be disabled, so technically its name doesn't necessarily represents its
characteristic properly. In terms of non-configurable defaults, keyword macros
is a truely basic macros.

## What is a keyword macro?

Keyword macro is a subset of basic macros. The biggest difference is that
keyword macro has a specific rules similar to define macro which requires late
evaluation of arguments. So keyword macros cannot be constructed from a string
but from a vector. Therefore other macros' argument can be constructed
dynamically while keyword macro's argument should have comma separated forms.

e.g.

```r4d
$define(args=true,Expression)
$define(ifc,a_cond a_expr=$if($a_cond(),$a_expr()))
$if($args())
$ifc($args())
===
// With 'rad --nopanic'
error: Invalid argument
= if requires two arguments
 --> test:3:2
$if($args())
Expression
error: found 1 errors
```

In this case, custom macro ifc can handle dynamically constructed arguments
because its argument is expanded before macro's evaluation. While if's
arguments are first split into array and then evaluated.

One other important difference is that you **cannot undefine or override
keyword macros**. Even with empty macro map, keyword macro is always evaluated
if found any.

## Local and global

There are two macro types in terms of macro scope. Local macro and global
macro. Local macro is a macro that only persists for certain period or scope.
While global macro persists for the period of processing.

Local macro is created while macro is being expanded and shadows other macros.
For example, macro argument is local macro and always evaluated first.

```
$define(arg1=ARG1)
$define(macro,arg1 arg2=$arg1() + $arg2())
%                        |        |
%                        Theses are the local macros and argument macros
$arg1()
$macro(first, second)
% You cannot use local macro outside of the macro
$arg2()
===
ARG1
first +  second
Invalid macro name
= Failed to invoke a macro : "arg2"
```

The other way to utilize a local macro is to use ```let``` macro. Let macro
creates local macro that only persists for the macro that used a let macro.

```
$define(macro,arg1 arg=
$let(arg1,$arg1() is first)
$let(arg2,$arg1() is second)
% arg1 and arg2 is shadowed by new let binding
$arg1() 
$arg2() 
)
$macro(1,2)
===
1 is first
2 is second
```

Global macro is a macro that is not purged after macro execution. Defined
macros, basic or keyword macros are all global macros.

```
% These are all global macros
$define(name=NAME)
$name(/home/path)
```
