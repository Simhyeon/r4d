# (Previous) How would one implement format comment macro

Suppose I have an example like following

Hello world % This is comment
Hello Sui % This is comment

I want to make this example into next aligned one

Hello world % This is comment
Hello Sui   % This is comment

Frankly speaking, there is no such thing as easy way of doing things.

Let's do some crude yet functional proto-typing of logics.

1. Save lines into a vector.
2. Set value max width as default value (0)
3. Set container for line blank offset and pattern index
3. Find a value, pattern. If exists, update max width. If not, simply pass
5. After the whole iteration -> 
    1. Split a line into two &str ( first, pattern after )
    2. Lengthen first part with empty blanks
    3. Attach pattern after
    4. Print to text


# I realized aligning after pattern is such a hard thing to achieve

Because of CJK and emojis. I'm korean so I have nothing to say, but we need
them anyway. Therefore I need a different approach.

1. Find CJKs (This should be a well known problem.)

2. Double the count and add to max charcters

3. Find EMOJIS 

4. Double the count and add to max charcters

## CAVEAT

Emojis are quite complex and more complex then CJK. CJK is mostly 3 bytes.
(Except yet-hangul.) However emojis can be 4 btyes and consists of multiple
unicode scalar. Despite of CJK Characters.
