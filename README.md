### R4d (Rad)

R4d aims to be a modern m4 alternative, which means

- No trivial quotes for macro definition
- Explicit rule for macro definition and usage so that de facto underscore rule
is not necessary
- Easier binding with other programming languages
- Enable combination of file stream and stdout
- As expressive as current m4 macro processor

### Implementation

#### No library

Since r4d allows inline macro expansion and definition, explicit rule based
parsing is impossible. Plus, R4d yields not expanded string, which is not
typical in traditional parser framework.

### Syntax 

#### Macro definition

Definition is also a macro. However it requires specifid form not like other
macros

```
// Define with simple form
// First argument is macro name
// Second argument is whitespace separated macro's argument list
// Third argument is body to be substituted
// Body can express argument usage with macro usage
$define(name,arg1 arg2, $arg1() $arg2())
```

#### Macro inovokation

Prefix(default is $, dollar sign) can be changed by end user.
```
$define(macro_name,a1 a2, $a1() $a2())
$macro_name(arg1, arg2)
```

Macro can be invoked anywhere

```
My name is $macro_name(Simon, Creek).
```
converts to
```
My name is Simon Creek.
```

#### In-built macros(WIP)

- Include
- Foreach, for loop
