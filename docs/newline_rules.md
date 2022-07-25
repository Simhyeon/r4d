## What newlines?

While using a rad you might twick macros a little bit because the ouptut
somehow is not what you'd expect. You put some $nl() after macro codes and
simply spam trim attribute to make it expectable. And welp it is totally a sane
thing to do. However newlines within rad processing have **consistent rules**,
although those are often overlooked.

## Macro's return value is either nothing or a value with empty content

Think of a macro as a function. ( Though it really is ) Macro gets arguments
and processes body with given information and finally returns a value. Actually
every macro returns a rust type of ```Result<Option<String>>```. Now any
rustaceans would know the meaning of the title, but for those who are not, this
means macro returns a real text value or nothing. 

**When nothing is returned newline is consumed.** All function macros returns
nothing if it has no text to print. Therefore macros such as define, let or
static removes folloiwng newline. For example, 

```r4d
$define(test=Test)
===
```

Define leaves nothing in it's place. What if define returned a real value but
empty value? Let's simulate by wrapping define inside a macro.

```r4d
$define(my_define,a=$define(b=))
$my_define()
===

```

In this case runtime macro ```my_define``` has no contents in it other than
define. Not even spaces or newlines. So it's easy to think that ```my_define```
would return nothing. However ```my_define``` returns an empty string. A real
value with empty texts. Therefore it leaves an empty string or an empty line in
its place. Consequently **runtime macros never return "no value"** but at least
return an empty value.

Although you can change this behaviour with help of trim output macro
attribute. Refer, [docs](./macro_syntax.md) for detailed usage.

## Hidden newline character at the end of a line

You might encounter a case like this

```
$define(hmm=
	$let(a,b)
	$let(c,d)
)
$hmm()
===


```

You wanted to spread a macro definition for readability and thought ```hmm``` would
yield a single newline. Because let would consume folloiwng new lines. The
definition expanded result is technically same with folloiwng.

```
$define(this_is_empty=
)
$logm(this_is_empty)
===
% cosole
% log:
%  --> test:3:2
```

That is a single newline. Then why ```hmm``` prints two newlins? The short
answer is, **because there is a newline after**. But let us dive deeper. If we
substitute ```\n``` to ```\\n```, and assume define is eagerly expanded. The
processing steps would like following.

```
% Source
$define(hmm=\n	$let(a,b)\n	$let(c,d)\n)\n$hmm()\n
% 1. Expand fisrt let and consume newline
$define(hmm=\n	$let(c,d)\n)\n$hmm()\n
% 2. Expand second let and consume newline
$define(hmm=\n)\n$hmm()\n
% 3. Definition also consumes newline
$hmm()\n
% 4. Run a hmm macro function and paste a return value
\n\n
% 5. Ok now that is two newlines :)
```

Every line has a newline character in it's end. You just don't know because our
smart text editor handles newlines as if it were not there. Even the substitued
line had newline character after the last character ```\\n```. It is hard to
write simply null terminated string without trailing newline in text editor. (
And I'm not sure if it is even possible in some editors)
