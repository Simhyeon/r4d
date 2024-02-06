# Todo immediate

* [ ] Possibly improve lexor...? -> I'm afraid :(

* [ ] Try to retain existing newlines as much as possible
* [ ] Add macro arguments for macro source

* [ ] Maintain consistencies over indexing and counting arguments
    -> Because some are 0 starting and some are 1 starting

* [ ] Sub works very strange

**Think Before adding a new macro so that can it be achieved as continuous macros**
e.g. inner a can be replaced with capture and inner combination

* [ ] Think about how failed operation should work
    - Macros that affect processor state shoulud be really careful when it can fail ( Lenient mode )

* [ ] Implement trim input 
    * [ ] Execute macro doesn't respect macro framgnet is it desirable? ->
      Kind of?
    * [ ] Expand also doesn't utilize attr yet. Is it ok?
* [ ] maybe `parse_chunk_args` can return cow instead of string

* [ ] Make a new slice macro for split and joining
* [ ] Always keep in mind that rangel can fail

* [ ] Add environmnet variable for preventing negative index for slicel(rangel)
* [ ] Add environmnet variable for insulah to customize split behaviour

* [ ] Change names that are inconsistent or incoherent
* [ ] Capture to support capture group
* [ ] For chunk
```
Iter through lines and aggregate regexed chunk and apply macro to it
$forchunk(start_regex,end_regex,macro_body,src)
```
* [ ] Split by
* [ ] Check insulav's logic throughly
* [ ] Escape rule is very outrageous
```
\\ -> \\
\ -> NONE WHAT?
```
* [ ] Check if all macros can handle empty arguments without panicking.
- Update manuals one by one
- Update all "None" variatns of manuals
- Add new macros that is immediately necessary for daily use
- Fix bugs that was found during manual update
    - Regardless of bug size and difficulties
    - Fix basic Lorem indexing

- Debug all manuals
    1. Test english charcters
    2. Test korean characters
    3. Test invalid syntax, characters.
    4. Test macro argument's default attribute

0. KEEP IN MIND : Escape rule is strange
    ```
    $assert(\,\) -> This doesn't invoked at all
    ```
    -> Check PS.r4d manual because escape character is strange in the document.

#### BUG

* [x] Inner panics on certain cenarios -> Failed to reproduce just in keep mind yeah...

* [x] Bug: insulav and insulah was not stripping
* [x] Fixed a bug whree skip_expansion was not working
* [x] Syscmd is inconsistent
    * [x] quote enclosed value has to be sent separately. Currently arguments
    * [x] Rad inside rad squeezes output -> It was other bug

* [ ] Check macros with single argument so that a function might not be
  analyzing shits.

* [x] Consider reverting changes for greedy and greedy strip

* [ ] How come insula doesn't print any insulav or insulah for help message?
* [ ] Improve repl's error code
* [ ] Not every funcion macro was treating zero width string as None.. It is
  shoking. Yet is it a "bug" that should be fixed or not?
    - I changed the behaviour of deterred macro and function macro to treate
    zero width string as None. Runtime and local macro should work as same.
    In this way, user knows why their macro has failed to malfunctioned.
    While making an experince of function macros comfortable.
    - However, giving an option to override is not necessarily a bad thing.
    Consider adding later.
* [x] Fixed where chars didn't work at all...
* [ ] Fix regex register shenenigans
 [ ] Fix deubbing feature bugs
    1. Add assertion information
        1. WHy it failed
        2. What was the value then,
    2. Diff
    3. Dryun
    4. Logging, Debugging
    5. You know what? Almost everything is bugged.

* [ ] Test all clap flags if they work as expected. -> DONE, except debugging
* [ ] Check CLI debugging options
    * [ ] Diff doesn't work at all
    * [ ] Dryun doesn't detect static macros...
    * [ ] Debugger panics from the start ;( Now it doesn't... like what is wrong with you?
    * [x] Fix require strict and require comment which doesn't respect vector rules -> auto.sh
    * [ ] Find similar cases
[bugs](./bugs_to_handle.md)

#### Documentation

* [ ] Notify that trim can remove empty newline
* [ ] Illustrate that insula macros are not pretty printer but, rather
  functional macro that creates sufficient spaces and newlines for following
  macro processes
* [ ] Changed rotate behaviour
    -> Think of center rotation as "ferris wheel"
* [ ] Update manuals
    * 24/177
* [ ] Update repository documentations
    * [ ] macro indices
    * [ ] macro syntaxes
    * [ ] Hook macro
    * [ ] About extension and abscene of include macro
    * [ ] ETC...
    * [ ] Stream series and pipe rules

#### Features

* [ ] Add a hygiene option for binary
* [ ] Add an option for Positive regulation which means auth related macros are
  only executed when it was allowed speicifically by user. Or simply wihout
  auth macros then
  -> This is to circumvent harsh hygiene rules and utilize runtieme macros

* [ ] Add regex-file option
    -> Adding a complicated regex is fucking hard and somtimes very time
    consuming
    -> Adding multiple regexes are also tiring
    ->> However making a literal rule concrete is the first thing to come though.


* [ ] Consider implemtnting consecutive macro calls for sc and sl flags
* [ ] Flag to print all realted environmnet vairables
* [ ] Panic message is kinda cringe... improve it.
* [ ] Update template macro...
* [ ] Check if greedy argument's no-strip behaviour is ideal or not
* [x] Make an interal data structure for arrays? ( Table ) 
    -> Implemented as cont macro

#### Macro ( macro )

* [x] Make a new macro for border?
* [x] Renamed indent to insertf
* [x] New macro insertr

* [ ] Add a new macro slice
    -> This splits string by pattern and slice them without separators
    -> Think of it as `cut` but returns range
```
$slice(pat,1,2,source)
```

* [x] Add rangel
* [x] Make alignby with complicated rules supportted
    * [ ] Notify users that align with comma will work strange
    * [ ] Rename macros that execute on lines that has no l suffix
    * [ ] Condl variant to respect leading tabs and spaces 
    -> Maybe this is a burden of pretty printer
* [x] Joinl macro
    * [ ] Add an environmnet variable to set sensible default for eliminating
      empty new lines. Or say, if something can be eaisly achieved by another
      macro there should be need to add sensible default behaviour. it is not
      consistent 
* [ ] Discard and print status macro
* [ ] Evalk formatting to be aesthetic
* [ ] TOC macro-script ( Not builtin but usage's example )

* [ ] Wrapl -> Wrap content by lines. == vim's "==" function
* [ ] Rotate concat -> Reverse of rotate macro
* [ ] Also add non evalexpr variant macro ( inc, dec )
* [ ] Pretty printer ( Json, toml etc... )
* [ ] Increaser by alphabets? ( Replacement for possible rer macro )
* [ ] Inner align? -> _$Stretch_
```
[a,b,c          ]
=
[a,     b,     c]
```
#### MISC, bug detecting, Ergonomics

* [ ] Should istype support string type? Which means non-digit in this case?
* [ ] Regcsv add skip parsing and skip extension maybe?
    -> Arg parser changed, so it might have been fixed pretty.
* [ ] Improve error messages for number related macros.
    e.g) strip series should indicate why index doesn't meet condition.
        -> Given content's length is ... and you gave index ...
* [ ] Check logging sanity as a whole for the time.
    * [ ] Stream related flags
    * [ ] Stream macro
    * [ ] SPread macro
* [ ] Check argument sanity of single argument macro
* [ ] Check if macro attribute is necessary for macro name input ( map, spread )

* [ ] Unicodwdith should be applied for aligin macro too
* [ ] Try using qsv rather than maintaining a wheel
* [ ] Make a color scheme option for color blindness
* [ ] Parsing and set error code of "~~ requires ~ arguments are super lame..."
        can it be much more DRY?
* [ ] Capture works on chunk based. Is capture by lines necessary? which works
  like ripgrep
* [ ] Consider implementing align super which applies consecutive alignby rules 
        -> e.g. first alignby ] and then by : and then # etc...
* [ ] Should textwrap respect unicode width?
* [ ] Rename rer? because rearrange is useful name and reordering numbers can be
        different name I guess
* [ ] Should rad support awk like operations?

#### Performance

* [ ] Become a no nester :).

* [ ] Add a feature to use rope instead of simple string ( Crop crate )
    -> For example if skip expansion flag was given OR text size is bigger than
    1K ( which is a standard point where rope out-performs normal string )
        -> text size standard for crop usage should be supplied as arguments
        and saved to processor ` --rope 1000 ` means use crop from 1000 byte
        sizes.

* [ ] There are multiple macros that utilizes args to vec directory or simply
  utilize args.is-empty which might cause inconsistent behaviour. Check them.
    -> THis means such macro doesn't use "argparser" which has an side effect
* [ ] Check unnecessary `to_string`
    * [x] FunctionMap
    * [ ] DeterredMap
    * [ ] Other

* [ ] Macro to return cow rather than string? Is it that performant?
* [ ] Try removing unnecessary clone calls
* [-] Mie and pie insert_str is inefficient. -> Not so necessarily
    -> Push_str is also O(n) Sadly
* [ ] Check for alignby performance maybe duplicate

* [x] Refactored `full_lines` so that unnecessary allocations happen
* [x] Remove unnecessary `lines` method to preserve line-ending

* [ ] Rer iteration cache to a concrete struct for better maintainability
* Think about ditching textwrap
* Inline small functions
    [src](https://matklad.github.io/2021/07/09/inline-in-rust.html)
* Remove `impl AsRef` as much as possible

* Change &args.to_string() into std::mem::take
    e.g.) let content = std::mem::take(&mut args[0]);
        -> function_map_impl:2374
* Use cow for performance improvement

* Remove a pattern such as ...
```

let mut lines = content.lines();
let line_count = lines.count();

if count > line_count: yatti yatta
```

THis is bad because count consumes. Error checking while iteration is better
but simply collecting is often faster.

* [x] Ditch wasm feature DONE -> Completely ditched
    -> However You still need to remove unnecessary features that was made for
    wasm target exclusviely. I sustained such codes with wasm keyword on comments

#### 4.0 IDEAS

* [ ] Should after and until macro support regex? because it doesn't for now

### TODO

$todo_start()
* [ ] Refactor
Totally change internal logic. Currently every text is processed as sequences.
Yet this approach has both pros and cons. Especially when you have to process a
file.
- To think about it, one of the power of r4d is stream based processing which
  makes concrete structure hard to be achieved... But shouldn't there be a
  hybrid way to do so?
* [ ] Bug fixes
    * [ ] Split_arguments should return Option not a result... Although this is
      somewaht breaking changes. Therefore I don't know if this is proper or
      not.
        - Why not create a new method which returns simply splitted array? and
          leave a method for correctly detecting a status
    * [ ] Currently user configured macro name is not "available" in log
      message.
      - Because proc macro doesn't support it as syntax
* [ ] Feature
* [ ] Documentation
    * [ ] About escape rules + parenthesis rule
* [ ] Test
    * [ ] Test windows build
$todo_end()

### LATER

* [ ] Test hook macro documentaion
