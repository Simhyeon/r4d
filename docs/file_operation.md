# File operation

File operation in r4d works slightly different by its behaivour. Some macros
works as bufread which means it reads file's content as consecutive lines
and send to writer without collecting. While some reads the file as big chunk
of string.

Basic work flow of file operation works like following.

```
Input -> Expander -> Writer -> Redirector
```

These are not official terms but used to illustrate internal logics.

## Include

Include macro works both way. If macro is in first level, include reads file as
bufread. But if include is called in nested context, it aggregates all data
into a result.

On a first level,

```r4d
% These are technically same operations
$include(file_name.txt)
$readin(file_name.txt)
```

But as nested call.

```
% Whole texts of file_name.txt is sent to outer macro.
$outer($include(file_name.txt))
```

This means that include macro collects all arguments until the "read" operation
finishes. After expander collects all string, include macro sends them to
writer which may be redirected to other variations.

## readin and readto

Readin and readto enforces bufread. This means that "read" text is not
collected, but gets sent to writer on line based. For example,

```r4d
% ASSUME that text.txt's content is "This comes after."
$define(nested,a_text= This should come before text $a_text())
$nested($readin(text.txt))
===
This comes after. This should come before text
```

This happens because while argument is being expanded, readin doesn't wait a
result but send file's content to writer. After readin sends all file contents
into a writer, then macros' body is evaluated and sent to writer. Therefore,
file's content was evaluated beforehand.

## Include with relay

There are times when you have to use relay pattern inside a file that you have
to include. However this will not always work by the level of macro's call
because of include macro's nature. ( Reminder, relay doesn't work inside
arguments.)

Let's assume that file ```inner.txt``` has a content as following

```txt
$declare(cont)
$relay(macro,cont)
1 2 3 4 5
$halt()
```

And we're using a index file with following contents.

```r4d
% On first level it works perfectly fine.
$include(inner.txt)
$cont()
===
1 2 3 4 5

```

This works as expected because include works as bufread. However, include
inside arguments process file content's as **arguments**

```
$if(true,$include(inner.txt))
$isempty($cont())
===
1 2 3 4 5

true
```

The text ```1 2 3 4 5``` was returned from if macro. And no texts were relayed
to cont macro. The reason relay didn't work as expected is because relay
doesn't work inside arguments. If you want to use relay inside a file, use
```readin``` to force bufread. Although this will print file's content before
your macro call evaluates, this technique is useful when you don't mind file's
whole content but only parts of it.

```
$if(true,$readin(inner.txt))
$isempty($cont())
===
false
```
