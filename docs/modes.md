### Strict related modes

There are various modes in processing 

First of all processor can be either **strict, lenient(purge), or nopanic**.

Default mode is strict mode. Within strict mode certain behaviours is not
allowed or acts differently.

- Overriding environment variables is an error
- Overriding existing macros is an error
- Failed executions of macros will interrupt a whole processing ( panics )
- Getting invalid environment arguments yields an error

Lenient mode is more flexible than strict mode.

- Overriding environment variables is not an error
- Overriding existing macros is not an error
- Failed execution of function macro will interrupt a processing but runtime
macros.
- Failure of runtime macro will yield unprocessed text into original position.
- Getting invalid environment arguments doesn't yield 

Purge is a special lenient mode that purges inexistent macros without any error
messages or left-over of macros.

```
% On lenient mode
this macro : $nono()
% On purge mode nothing is printed.
this macro : $nono()
===
$nono()
```

Nopanic simply prevents all panic at all with purge feature. But it will still
yield error log for function macro failures.

### Hygiene modes.

By default processing is not hygienic which means that macro's behaviour can
change according to external variants.

For example, following macro can fail according to inner macro's existence.

```
$define(unsafe=$some_random_macro())
% If some_random_macro exists if succeeds. If not, if fails.
```

**Macro hygiene** mode set every further runtime modification as
volatile(temporary). Within hygiene mode, runtime macros gets automatically
deleted after every macro call which resides in first level

**Input hygiene** mode clears every volatile macro after per input. This is
mostly useful in a context of library.

**Aseptic** mode disables runtime macro defintion at all. You cannot define any
runtime macro and only able to use pre-defined runtime macros.
