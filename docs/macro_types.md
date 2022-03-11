# TOC

- [Types of macro](#types-of-macro)
- [Local and global](#local-and-global)
- [Text and operation](#text-and-operation)
- [Hook macro](#hook-macro)

## Types of macro

There are basically two types of macros. Function macro and runtime macro.
Function macros are macros that trigger rust functions. Runtime macros are
macros that trigger text substitution that possibly include other function
macros.

Function macros are mostly built-in macros provided by r4d itself. However you
can also extend function macros with r4d's library interface.

There are special types of function macros that is called deterred macros.
Deterred macros are not evaluted before invocation and conditionally expanded
according to the macro's logic.

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

## Text and operation

Some macros are text macro which expands to text while some other operational
macros that changes a behaviour of processor.

For example; hygiene, pause or relay are operational macros. Keep in mind that
operational macros might not work as **procedurally** because operation macro
applies before text is retrieved from macro call.

Generic order is followed

- Parses text
- Apply operational macro 
- Possibly get expanded text from macro call

## Hook macro 

**Libary usage only**

Hook macro is a macro that is called automatically. There are two types of hook
macro. One is ```macro hook``` and the other is ```char hook```.

Macro hooks are called upon macro calls and char hooks are called upon char
encounters. However char hook is applied for plain texts which are not enclosed
by a macro call.

Hook macro should be registered before processing and can be enabled with
```hookon``` macro.

for example,

```rust
let processor = Processor::new();

processor.register_hook(
    HookType::Macro,            // Macro type
    :trigger_macro,             // Macro that triggers
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
