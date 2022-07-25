# Code manipulation

You can define and execute macros within a single file. But you can separate macro
codes for better maintainability.

The easiest way is to simply separate definitions and executions into a
separate r4d files. For example, put all define and static calls inside
```def.r4d``` and put all logic macros inside ```index.r4d```. Executing a
command ```rad def.r4d index.r4d``` will work as if they were a single file.

## Code freezing

The other way is to freeze macros inside a file. Macro freezing is conceptually
equal to m4's freezing. Rad processes macro definitions and save it into a
binary file. The frozen file can be melted anytime and can be used without
processing. 

```bash
# Freeze to a file
rad file_to_freeze --freeze -o lib.r4f

# Metl a file before processing a file
rad file_to_execute --melt lib.r4f
```

**Rules for freezing**

There are some rules applied to when macro is being freezed to a file. General
rule of thumb is that **optional definition is not allowed**. Consequently
following rules are applied.

- Only first level "define" is allowed.
- Deterred macro is not expanded.
- Every permission is closed.

# Packaging macro codes

You can merge macro codes into a statically packaged execution script with
```--package``` flag.

```bash
# Package into a single file
rad index.r4d --melt lib-license.r4f lib-i18n.r4f --compile -o bin.r4c

# Execute a file
# File that has .r4c extension is automatically interpreted as static script
rad bin.r4c

# You need to feed a flag "script" to interpret input as packaged script.
rad bin.r4d --script
```
