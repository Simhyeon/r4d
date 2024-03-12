# Manual note ( This has be checked later n )

## Function Macro

- fileout: Should it have truncate as argument or as env
- Border : add a new env for border exception rules
- Cut and scut support `_` syntax -> Change argument type
- Change clear's manual after binary hygiene is updated
- dnl,enl logic check
- Refactor istype
- insulav and insulah
- docu macro after complicated runtime definition
- Hygiene
- Sortc
- Import 
- notat
- get-pipe
- relay series + halt
- source
- strict
- rangeby
- table...
- tr : with performance update
- exdent
- def + anon ( this is deterred )
- cindex
- hook

## Deterred

- Bufread
- readto
- tempin, raw mode
- append : Later with modification macros
- map seires

Actually finish deterred macro later...

## Misc

* Find \" and \\
* Remove examples
* Make names of Parameter intuitive
    - Literal pattern should be distinctive from regex pattern

* Replace tabs with 4 spaces

* Search keyword TODO
* Remove all `# Return`
* Some manuals lacks arguments name ( Especially optional ) and inconsistent naming.

# todo immediate

* [ ] New valuetype regex so that user knows which value should be requested.
    * [x] Added type with getter
    * [ ] Should be added to function signatures

* [ ] ArgParser allow empty input
    * [ ] Count

* [ ] Forin -> For inner : Iterate through inner calls ( increase by one )

* [ ] Possibly a new feature to allow permission for specific macro

* [ ] Add a new macro to `drop expression` + add doc for addexpr

* [ ] Currently arg_parser detect `$name(` to search macro invocation but this
  doesn't respect custom macro character rule

* [ ] Comment disrupts consume newline. Possibly due to newline Refer
  `get_pipe` manual

* [ ] Permission as macro signature.
    * [ ] Check if permission is coherent

* [ ] Change freeze melt to differnt terms
* [ ] Possibly sortc to sorte?
* [ ] Index should support _ syntax or not? because it is handy but makes
  indexing undefined from static perspective.
* [ ] Currently runtime map doesn't check auth

* [ ] Map series with arguments are very limited. it is impossible to insert
  space as arguments... can it be improved?

* [ ] Name alias

* [ ] Remove unnecessary pub keywords and make it private

* [ ] Is rangeby really necessary?

* [ ] --log is not useful in general cases... Paste expanded and split
  arguments into log message

* [ ] Update manuals so that you can fix bugs. Especially deterred macro

* [ ] Consider `strip` cases

* [ ] `Parse_chunk_arg` to return Cow<'a,str>

* [ ] Make error codes much more intuitive

* [ ] Check unnecessary ctext
* [ ] Check type incoherence

* [ ] FN `new_ext_macro(&mut self, ext: ExtMacroBuilder)` is currently
  disabled 

* [ ] Rearrange modules and struct and enums.

* [ ] Split by

- Debug all manuals
    1. Test english charcters
    2. Test korean characters
    3. Test invalid syntax, characters.
    4. Test macro argument's default attribute


#### bug

* [ ] On some cases, rad prints strange error : WTF is this error...?

```
-> Argument doesn't match argument type
```

* [ ] Exdent doesn't work well with argument type system...
* [ ] Mapn broken?
    -> Mostly due to `NUM_MATCH` this finds both + and - from first... HMM
    
* [ ] Loop is also buggy.

* [ ] BUG : Error inside file trigers nested error message
* [ ] Currently bufread series has a bug
```
thread 'main' panicked at src/logger.rs:542:31:
-> Looks like no tracker was created
```

* [ ] Map exression (mape) is completely broken... damn...
* [  ] Bug

```
$define(typefy,a_src=
    $forline=(
        $let(line_number,$a_LN())
        $let(line_src,$:())
        $logm(line_src),
        $a_src()
    )
)
Doesn't yield a_LN and $:() why is that?
```

* [ ] Bug : Peel removes following text after "to be peeled".

* Check PS.r4d manual because escape character is strange in the document.

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
    * [ ] Find similar cases

[bugs](./bugs_to_handle.md)

#### Documentation

* [ ] Notify users that trim input is applied after expansion.

* [ ] Notify that trim can remove empty newline
* [ ] Update repository documentations
    * [ ] macro indices
    * [ ] macro syntaxes
    * [ ] Hook macro
    * [ ] About extension and abscene of include macro
    * [ ] ETC...
    * [ ] Stream series and pipe rules

#### Features

* [ ] --eman option to print manual for environmnet
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

#### Env

* [ ] Enable frontal search for rotatei
* [ ] Env to apply `trim_each_lines` for trim-input
* [ ] Env for bordering ( exdentl )
* [ ] Env to auto precision for eval? maybe?
* [ ] Env allow empty count
* [ ] Env to retain newlines for strip series
* [ ] Env to verbose print for container
* [ ] Env cont pop no return
* [x] Env to bypass return validation

#### Macro ( macro )

* [ ] Fill macro
```
$fill([,],x,* [] ABC)
$fill([,],x,* [ ] ABC)
$fill([,],x,* [  ] ABC)
% With env RAD_FILL_REPEAT
$fill([,],xy,* [   ] ABC)
===
* [x] ABC
* [x] ABC
* [x ] ABC
* [xyxy] ABC
```
* [ ] Refactor qualify-value method
* [ ] Refactor list-directory-files
* [ ] Wrap as unicode charaters
* [ ] Macro body modification macro
    * [x] pop
    * [x] append
    * [x] prepend
    * [ ] Replace : Current implemnation only replaces runtime macro not local macro is it ok?
    -> Which modification method should be provided or not. hard question.

* [ ] Peelmap
* [ ] Insertat(index,target,source)

* [ ] For chunk
```
Iter through lines and aggregate regexed chunk and apply macro to it
$forchunk(start_regex,end_regex,macro_body,src)
```

* [ ] Add a new macro slice
    -> This splits string by pattern and slice them without separators
    -> Think of it as `cut` but returns range
```
$slice(pat,1,2,source)
```

* [x] Make alignby with complicated rules supportted
    * [ ] Rename macros that execute on lines that has no l suffix
    * [ ] Condl variant to respect leading tabs and spaces 
    -> Maybe this is a burden of pretty printer or env
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
#### ergonomics, misc

* [ ] Change order of Parameter for better piping
* [ ] Support multiple arguments for such desirable macros
    -> Eagerly find appropriate one
* [ ] How to include deterred order in signature?
* [ ] Add a warning message when user tries to repl function macro 

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

* [ ] Rearrange processor's lex branch method's arguments
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

### todo-previous

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

### LATER

* [ ] Test hook macro documentaion

### Temporary done

#### Return Type

* [x] Include return value for type signature
    * [x] Implenet enum table for enum type
        * [x] Currently about 10 items are left...
    * [x] Make a ergonomic builder pattern for enum table
* [x] Also apply enum table for return value

* [x] Subverge relay to relay and relay temp and relay file
* [x] Fixed a in-body comment bug

#### Others

* [x] Syscmd -> shell
* [x] Refactor ParralelRight 
* [x] Possibly change usage syntax -> Arg name rather than arg type
* [x] Find a way to display if optional is multiple or not
* [x] Re-enable eval feature ( default for binary but can be disabled )
* [x] Remove border and implement it as env for exdentl
* [x] Utilize proper optional arguments rather than split logics
* [x] Change signature for FMacroDesign so that level can be included
    * [x] Now level is merged to input

* [-] Macro name path might be inappropriate
    -> THere are hardly any alternative
* [x] Revl -> Reverse lines
* [x] Removed counter macro -> use mie instead
* [x] --allow should use adequate syntax comma rather than weird plus syntax
    -> Not it is split by comma
* [x] Is rename hijacking bad so that I need to handle or not?
    -> Add a new permission "DYN"
    -> Both rename and undef requires dyn because undef + def can simulate rename
* [x] Return type validation should also check no return -> Which was already
  done but I cleaned the code anyway
* [x] Fixed ranga index panic error
