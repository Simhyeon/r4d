# DONE

[DONE]((./done/4_0.md))

# Todos

[todo](./todo.md)

# Changes

## Macro ergonomics

- Macro chain : This is much harder because many internal changes

## Hard+misc ones

* [ ] Implement array object.
* [ ] Projects performance

- Argument parsing to return a slice of values not a string would be good I guess?
    - This needs to implement cow manipulation and I'm... ok maybe later
- Utilize regex engine for fast parsing especially, define parsing. Possibly
whole parsing process, meh I don't think I can... Focus on define parsing.
- Refactor codes into multiple chunks of functions for better readability
- Use faster hashmap
- Currently lexing copies all chars into a versatile vector which may not
be the best idea. A bettter idea is to iterate and add slice from source.
But it is true that such implementation is more trivial to maintain and
extend with arguable performance boost.

* [ ] Make much more smaller binary file available as basic feature
    - Opt-lvel has huge effet (2.2M -> 1.8M)
    - codegen-units = 1 has a 0.1M gain ( increase compile time but it's small anyway )
    - upx is a holy banger (1.7M -> 500K ... Just wow...)

## Delayed, paused or stopped

* [-] Error and warning types
- Enable user to suppress specific types of warning
- Personally, I deicded it is not improtant or demanded feature. Thus discarded.

* [-] File feature with commands such as ls, pwd
- R4d is not a m4 replacement rather, it is an alternative for text processing.
R4d aims to be an booster of documentation not a build script. Thus extra
complexity for ls, pwd support are outrageous.

* [-] Custom keyword macro
- Every macro can be used as basic macro
- And I could not achieve what i wanted because I wanted optional parsing, which is hard to add as interface.
- However, to thin of combining if and function is actually impossible in plain programming languages. Of course there are no reasons why not for r4d? But I don't think it is very indeed a demand

* [-] Define regional macro -> Is it necessary? Or is it desirable?
- This maeks macro execution and debugging so much harder
- And also makes structure too complicated for small gain

* [-] Print macro information might be also useful
- However I think it needs a serious refactor of debugger logics.

* [-] Eval command in debug mode maybe useful? -> Yes but, it doesn't meet the point of debugging

* [-] Make parser separated
This is hard to make it right... for current status

Following is such hard to do cleanly, deterred
* [-] Lipsum variants
  - Lipsum width word limit (new lines after certain words)
  - Lipsum width width
  - Lipsum cache which utilizes cache files saved in tmp directory.
  is useful because calculating length and word limit is much more expensive
  simple random generation.
  - Lipsum with custom separator

### NOTE

1. About dnl and enl inside body
    - ENL or DNL doesn't work inside macro body because dnl is evaluated on every "line".
    - ENL is evaluated on "NONE" Cursor which menas enl is not evalued properly
    inside body. where lexor's cursor never goes to NONE state.

### How macro parsing works?

1. Get file buffer or stdin buffer
- It is deliberately decided to get a bufstream not a single string because
file or standard input maybe very big which is not so memory friendly and
partial processing will yield nothing if input was given as a huge chunk.
- Yet definition body processing doesn't read bufstream but a single chunk
of body for simplicity of error logging.
2. Iterate by lines
- This iteration doesn't use std's builtin lines() literator but use custom
iterator method which was blatantly copied from stack overflow. While std's
lines method always chop newline chracter, this was not ideal in r4d's
processing procedure.
3. Check if line include macro invocation
- Checking is simply done with character comparision of "$" sign existence.
4. Check if line include partial macro invocation
- This is a branch executed only when macro fragment is not empty, which means
current line is a part of long macro invocation.
5. Push lines to `chunk` until macro invocation is complete
- For 3 ~ 5 phase what so called "lexing" is introduced. Lexing has its
cursor(state machine) in its body so that lexor can define whether iterated
chracter is valid character for macro invocation or not. Invalid characters
will break the macro framgent and return original text.
6. Execute macro invocation and substitute with final result.
    - If macro doesn't exist, original text is returned
    - The order of macros are "local", "custom" and then "basic", this enables
user to override basic macro with custom macro.

### How basic macros work

Basic macros are simply a hashmap of functions with a string key. If processor
finds a macro fragment(chunk). It tries to evaluate the chunk and find sif
macro name is included in macro maps. After finding local and custom macros, if
basic macro name is found, macro call(function) is executed. Thus adding a
basic macro is as simple as creating function and insert a new hashmap item.

# Ditch feature signature, because it became too crucial.
