## What's a difference between custom macro and basic macros?

Basic macro is a macro like function pointer or closure. While custom macro is
a macro that doesn't include complicate logics but only gets expanded according
to given rules.

## Why the name basic?

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
