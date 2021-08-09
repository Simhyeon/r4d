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

I tried pest, however I didn't like the limitation of the syntax. Plus, other
parser generator libraries recommend using custom parser for programming
languages. 

The reason stock library doesn't fit well comes from the expressiveness and
complexity of m4 macro processor. Technical difficulties are followed,

- M4 syntax allows string literal in its file which is not allowed in many
programming languages
- M4 allows macro definition in any location of given input, even among string literal
- M4 allows shell execution in any location of given input.

Thus there are too much caveats to with exsiting library that can differentiate
string literal and macro definitio and macro usages at the same time.

### Syntax (This is a plan, not the real specification)

#### Macro definition

Definition is also a macro.

Definition should start from the first of the line

```
// Define with json form
$define({name: "", args: [""], contents: [""]})
// Define with simple form
$define("",[""],[""])
```

#### Macro inovokation

Prefix(default is $, dollar sign) can be changed by end user.
```
$macro_name(arg1, arg2)
```

Macro can be invoked anywhere

```
My name is $macro_name(Simon, Creek).
```

#### In-built macros

- Include
- Foreach, for loop
