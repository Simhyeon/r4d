### Strict related modes

Processor's error behaviour can be either **strict, lenient or purge**.

Default mode is strict mode. Within strict mode, certain behaviours is not
allowed. On strict mode,

- Overriding environment variables is an error
- Overriding existing macros is an error
- Failed executions of macros will interrupt a whole processing ( panics )
- Getting non-existent environment arguments yields a warning

Lenient mode is more flexible than strict mode.

- Overriding environment variables is not an error
- Overriding existing macro is not an error.
- Failed macros will be pasted as literal text.

Purge is a special lenient mode that purges non-existent macros without any
error messages or left-over of macros.

```
% On lenient mode
$nono()
% On purge mode nothing is printed.
$nono()
===
$nono()
```

### Hygiene modes.

By default processing is not hygienic, which means that macro's behaviour can
change according to external variance.

For example, following macro can fail according to inner macro's existence.

```
$define(unsafe=$some_random_macro())
$unsafe()
===
% If some_random_macro exists it succeeds. If not, if fails.
```

You can configure this behaviour through processor's method with following
variants.

**Macro hygiene** mode set every further runtime modification as
volatile(temporary). Within hygiene mode, runtime macros gets automatically
deleted after every macro call which resides in first level

**Input hygiene** mode clears every volatile macro after per input. This is
mostly useful in a context of library.

**Aseptic** mode disables runtime macro defintion. You cannot define any
runtime macro and only able to use pre-defined runtime macros.
