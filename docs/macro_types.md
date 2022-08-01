# TOC

- [Types of macro](#types-of-macro)
- [Local and global](#local-and-global)
- [Text and operation](#text-and-operation)
- [Hook macro](#hook-macro)

## Types of macro

There are technically two types of macros. A function macro and a runtime macro.
Function macros are macros that trigger rust functions. Runtime macros are
macros that trigger text substitution that possibly include other function
macros. Both macro types are syntactically same but has difference in that
tokenized arguments are mapped to argumens in runtime macro and function macro
can manipulate argumens maually or even interrupt such tokenizing behaviour.

Function macros are mostly built-in macros provided by r4d itself. However you
can also extend function macros with r4d's library interface. [How to
extend](./ext.md)

There is a special type of function macro that is called deterred macro.
Arguments of function macros are eagerly expanded while deterred macro
interrupt such expansion and can control the entire behaviour.

Not every macro that markded as deterred deters an argument expansion. If you
want to know the exact order of expansion refer a manual with ```--man``` flag.

## Local and global

There are two macro types in terms of macro scope. Local macro and global
macro. Local macro is a macro that only persists for certain period or scope.
While global macro can be accessed within whole processing.

Local macro consists of two cases. First is automatically bound by processor
and second is set by user. When processor processes texts, macro's argumens are
expanded, tokenized and then mapped to local arguments.

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
creates local macro that only persists for scope of the macro that used a let
macro.

```
$define=(
    macro,
    a_arg1 a_arg2
    =
    $let(arg1,$a_arg1() is first)
    $let(arg2,$a_arg2() is second)
    % arg1 and arg2 is shadowed by new let binding
    $arg1()
    $arg2()
)
$macro^(1,2)
===
1 is first
2 is second
```

**Local macro is not expanded**

The most important trait of local macro is that **local macro is not
expanded**. Processor simply returns indexed body. Therefore if a local macro
has it's content as a form of macro expressions, it will be printed as if it
were escaped.

```
$define(demo,a_boolean=$if($a_boolean(),It is true))
$demo(\*$not(false)*\)
===
error: Invalid argument
= If requires either true/false or zero/nonzero integer but given "$not(false)"
 --> test:2:2~~
```

When "demo" was invked, it had bound ```\*$not(false)*\``` into an argument
```a_boolean```. The binding is technically same with let syntax. Let expands
the target expression and assign it to a macro.

In this case, text ```$not(false)``` was bound to a local macro ```a_boolean```
and inner macro ```if``` received it as a literal text not a expanded value of
```true```.

On the other hand, global macros are macros that can be invoked from anywhere.
Built-in macros, runtime macros are all global macros.

```
% These are all global macros
$define(name=NAME)
$name(/home/path)
```

## Text and operation

Some macros are text macro which expands to text while some are operational
macros that changes a behaviour of processor.

For example; hygiene, pause or relay are operational macros. Keep in mind that
operational macros might not work as **procedurally** because operation macro
applies before text is retrieved from macro call.

## Hook macro

**Libary usage only**

Hook macro is a macro that is called automatically. There are two types of hook
macro. One is a ```macro hook``` and the other is ```char hook```.

Macro hooks are called upon macro calls and char hooks are called upon char
encounters. However char hook is only applied for plain texts which are not
enclosed by a macro call.

Hook macro should be registered before processing and can be enabled with
```hookon``` macro.

for example,

```rust
let processor = Processor::new();

processor.register_hook(
    HookType::Macro,            // Macro type
    trigger_macro,              // Macro that triggers
    "hook_div",                 // Macro to be executed
    1,                          // target count
    false                       // Resetable
);
processor.register_hook(
    HookType::Char,             // Macro type
    '#',                        // Char that triggers
    "icon",                     // Macro to be executed
    2,                          // target count
    true                        // Resetable
);
```

to enable this hook macro

```r4d
$define(trigger_macro=Trigger)
$define(hook_div=
<div>I'm created</div>)
$define(icon= <i class="header-i"></i>)
$hookon(macro,trigger_macro)
$hookon(char,#)
$trigger_macro()
## I'm second header
## I'm another second header
===
Trigger
<div>I'm created</div>
## <i class="header-i"></i> I'm second header
## <i class="header-i"></i> I'm another second header
```
