# Problems

- Step inside macro definition panicks
- I fixed it somehow by reverting parse chunk body
- However I cannot get improved information about other things
- THis is because logger lines are inappropriate

# Approach

- I made a separate crate named tracks( To be renamed later ) which tracks text
propagation.
- I can now get tracing infomation about text semantic location

# Yet to be solved

- I need to merge tracks appropriately to display information properly
- This is also pivotal to make debugging right

# Next approach I should take

- Tweak usages of append\_track method and merge methods.

# Desired goal

## elog should print a line and charater number of invoked call after dollar sign

```
123456
---
$mac()

```

- Currently it prints 1,6, but it should print 1,2
- This is because, logger uses line and charater information of **Full track**
  which points to current cursor position. ( Because full track merges all track numbers )

## Arg parse should print inner information not a macro invocation order

```
123456789012
$mac(
    $inner()
    )
```

- On previous ok elog, error or inner macro printed as if it were in 1,2 which
  is same with mac's position. This is acceptible, but can be improved.
- On current implementation, it prints 3 which is 1 + 2. One from mac's
  position and 2 from inner's position. This is due to Argument going
  backwards. To solve this you have to merge paths carefully. 

How to merge path varis, but merged path should be, mac's position + inner's
position calculated by new tracker. In other words, new tracker's full track +
parent tracker's full tracks without last track.

Since evaluate always called before merge\_path. It will be safely assumed
that, range operation can be safely merged.

Sub-ranging tracks do work ( with high possibility of panicking though ), but
the current problem is that. Parse chunks args should create a new track with 0
value.

* [x] Check is processed consistency
    * [x] Empty name : THis doesn't print erorr but simply panick
    * [x] Comment exit
    * [x] Exit frag
    * [x] Restart : char number is broken
* [ ] I put frag.is processed everywhere, I need to check those are necessary
* [x] Currently logger uses constant LINE_ENDING, is this ok?
    - Since this is not formatted to output. It is theoretically ok, but might
      not what user would expect
* [x] A bug... is that currently, define always consumes newlines... what? No
  it was just that, logm consumed result... like wtf.

* [x] Strict error code with line numbers are tedious. Can it be changed?
* [x] Logger struct

* [ ] Clear processor's re-export of logger methods

# Debugger

* [x] M command works
* [x] N command works
