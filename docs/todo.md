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

* [ ] Move macros from deterred to function that doesn't have to be deterred.

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

* [ ] Make a newtype pattern

* [ ] <Performance> Module soundness + publicity
    * [ ] Expose processor methods with impl arguments
    * [ ] Use internal concrete type for crate method 
    * [ ] Don't use expoed method for internal processor, if necessary make a
      internal only determined method
    * [ ] Use string::from for static str and `to_string` for display-able type
    * [ ] Remove unnecessary `to_string` calls
    * [ ] Recheck log_warning and log_error sanity
    * [ ] Currently removing unnecessary `to_string` with Into<String> 
        * [ ] Make a constructor for error type so that impl Into<String> can
          be utilized
    * [ ] Check publicity and modules
        * [ ] Add a new enum `LogLine`
        * [ ] Check the followings.
            * [ ] Tracktype publicity
    * [ ] There are too many fucking methods that receives str and then convert
      argument to string and I send Newly created string as reference.
        -> This is retarded as its finest form... Please remove all these stupid codes
    * [ ] Replace following pattern
    ```
        self.logger.start_new_tracker(TrackType::Input(
            input_name.unwrap_or("String".to_string()).to_string(),
        ));
        with 
        ===
        self.logger.start_input_record("String");
    ```

* [ ] First
    * [x] Add hygiene
    * [x] Add feel macro
    * [x] Add reverse-feel macro
    * [x] Make macro character fixed 
    * [x] Negation as argument not as string
    * [x] Make pipe a vector based
        * [x] Pipe input should be applied to `macro_input` rather than args
        * [x] Now change `arg_parser` behaviour lastly
    * [x] Test if pipeinput work for macros
        * [x] Runtime -> Still doesn't work...
        * [x] Function -> Empty arguments error... classic!
        * [x] Deterred
    * [x] Test pipeOutput
    * [x] New macro logp -> log pipe
    * [x] Change pipev signature from single value to multiple optional
    * [x] Simplify macro attribute usages
        * [x] Now trim input is `<` rather than ambiguous `=`
    * [x] Ditch optional-multiple and required auth from runtime macro
    * [x] Consider implemtnting return type for runtime macro ( ? )
    * [x] From invoke-runtime apply `to_arg` + negation logic + return type validation
    * [-] How about "=" rather than double "-" ?
        -> It's not bad in theory but bad for intuition, because case like
        `$mac=-()` can make confusion whether this macro intended string input
        or vector input. While `$mac--()` intentionally means overriding
        default behaviour.
    * [x] When a "runtime macro" that accepts zero arguments do receive them, the error
    * [x] CText doesn't work for runtime macros -> Fixed
    * [x] Check trim input behaviour of other macros
        -> Ok, trim input is applied per `get_arg` method
    * [x] "Length of a runtime or local macro" macro -> mlen
    * [O] ----------FROM DEV+REF----------
    * [x] Ditched strictpanic
        * [x] merge log and `log_no_line` 
        * [x] Don't eprint but rather return 
    * [x] Renew allowed comment character
    * [x] Macro definition validity check is was totally fine... I was fool
    * [O] ----------DONE----------
    * [ ] Fix bugs
    * [ ] Make an ergonomic getter for regex and make it forced to insert regex
      later.
    * [ ] I was not using `get_regex` none at all... wow
        -> Actually there was a reason for that... but I didn't document it.
        I exclusviely used `get_text` to stop side effects which is logical
        but... kinda frustrating that there is a regex type while it doesn't
        return proper type
        * [ ] Let's create a new type for regex called RadRegex;
        * [ ] Search `insert_or_something`
        * [ ] Function macro
        * [ ] Deterred macro
        * [ ] Pattern `p.insert_regex(reg.take()?)?;` can fail because this is
          only valid when `reg` is a newly created regex not referenced which
          is not the "whole" case.
    * [ ] Restart manual checking
        * [ ] No manuals for deterred macros
    * [ ] Test error messages
    * [ ] Check things for basics
    * [ ] Complete manual ( No adding macros... please )
    * [ ] Push to master
* [ ] Later
    * [ ] Use range over (usize,usize)
    * [ ] Complicated macro defintiion
        * [ ] Revert enum table for runtime macro
    * [ ] Improve cli ergonomics ( e.g. Shell ValueHint , enumeration values...)
    * [ ] New macros
    * [ ] Performance ... 
    * [ ] ExtMacroBuilder ... Extension etc...

* [ ] Consider implemtnting independent macro map struct and segregate from processor
* [ ] Consider implemtnting Logger struct and segregate from processor
* [ ] Consider implemtnting Debugger struct and segregate from processor

for e.g.

```rust
let mut map = MacroMap::new();
let mut logger = Logger::new();
let mut debugger: Debugger::new();
let proc = Process::new()
    .map(map)
    .logger(logger)
    .debugger(debugger);
```

* [ ] Possibly warn when zero input macro actually accepted some arguments

* [ ] Currently negation cannot logically work for runtime macro due to logic
  changes.

* [ ] Mediocre result for duplicate error logging... Make hash and skip if
  previous is same with current logging.

* [ ] Should runtime macro also respect empty return? like function or deterred?

* [ ] Currently arg_parser detect `$name(` to search macro invocation but this
  doesn't respect custom macro character rule

* [ ] Index should support _ syntax or not? because it is handy but makes
  indexing undefined from static perspective.
    -> Definitely but if range already supports? then... why not?

* [ ] Name alias

* [ ] Consider if `strip` methods are required
    * [ ] This is mostly related literal quote

* [ ] Make error codes much more intuitive

* [ ] Check if which macro should support empty input e.g. count
* [ ] Check unnecessary ctext
* [ ] Check type incoherence

* [ ] FN `new_ext_macro(&mut self, ext: ExtMacroBuilder)` is currently
  disabled 

* [ ] Find better method than levenshtein
* [ ] Rearrange modules and struct and enums.
    * [ ] Function should belong to processor not sub structs e.g. state, map
* [ ] Remove unnecessary pub keywords and make it private

* [ ] SplitBy macro ?

- Debug all manuals
    1. Test english charcters
    2. Test korean characters
    3. Test invalid syntax, characters.
    4. Test macro argument's default attribute

#### bug

* [ ] Exdent doesn't work well with argument type system...
    
* [ ] Loop can be buggy?. Not sure but can be buggy

* [ ] BUG : Error inside file triggers nested error message
    -> NOt every error but some errors. Serach `log_error`
    -> Remove strict error, make it returned not logged.
* [-] Currently bufread series has a bug
```
thread 'main' panicked at src/logger.rs:542:31:
-> Looks like no tracker was created
```
-> Failed to recreate.... god damn...

* [ ] Map exression (mape) is completely broken... damn...

* [ ] Bug : Peel removes following text after "to be peeled".

```
let val = wow(t) abc
===
let val = t
```

* Check PS.r4d manual because escape character is strange in the document.

* [ ] How come insula doesn't print any insulav or insulah for help message?
* [ ] Improve repl's error code
* [ ] Fix regex register shenenigans

---

 [ ] Fix deubbing feature bugs
    1. Add assertion information
        1. WHy it failed
        2. What was the value then,
    2. Diff
    3. Dryun
    4. Logging, Debugging
    5. You know what? Almost everything is bugged.

* [ ] Check CLI debugging options
    * [ ] Diff doesn't work at all
    * [ ] Dryun doesn't detect static macros...
    * [ ] Debugger panics from the start ;( Now it doesn't... like what is wrong with you?
    * [ ] Find similar cases

[bugs](./bugs_to_handle.md)

#### Documentation

* [ ] Notify users that trim input is applied after expansion.
* [ ] Notify users how mapn works ( + and - are all included )

* [ ] Notify that trim can remove empty newline
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

* [ ] --eman option to print manual for environmnet

* [ ] Complicated runtime macro definition
    * [ ] Auth, ret type etc...

* [ ] Possibly a new feature to allow permission for specific macro

* [ ] Add regex-file option
    -> Adding a complicated regex is fucking hard and somtimes very time
    consuming
    -> Adding multiple regexes are also tiring
    ->> However making a literal rule concrete is the first thing to come though.

* [ ] Consider implemtnting consecutive macro calls for sc and sl flags

* [ ] Panic message is kinda cringe... improve it.

* [ ] Update template macro...

#### Env

* [ ] Drain behaviour Currently -> `* [x]` -> `* [ ]` ( Insert single space )
* [ ] Fill expand ( [  ]  -> [xxxx])
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

* [ ] Change sortc to sorte?
* [ ] Forin -> For inner : Iterate through inner calls ( increase by one )

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

* [ ] Rename macros that execute on lines that has no l suffix

* [ ] Condl variant to respect leading tabs and spaces 
-> Maybe this is a burden of pretty printer or env

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

#### Performance

* [ ] Pattern such as `&["a","b"]` is kinda hard to put but this is less
  abmgious and saves compile time. While pattern like `["a", "b"]` has to be
  accepted with `IntoIterator<Item = AsRef<str>>` which increases compiles time
  but much cleaner for users. Think about what is better.

* [ ] Argerparser cursor can return vector of ranges not a total string.
  Currently cursor returns either single range for string which is comparably
  worse in performance.
* [ ] Return value variant : Slice -> Trim, range etc...
* [ ] `Parse_chunk_arg` to return cursors
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
* [x] ArgParser allow empty input
* [x] New valuetype regex so that user knows which value should be requested.
    * [x] Added type with getter
    * [x] Should be added to function signatures
* [x] Return valuetype from macro

* [x] Change freeze melt to differnt terms

* [-] Refactor list-directory-files : Dropped
* [x] New log line for log flag
* [x] Comment disrupts consume newline. Possibly due to newline Refer `get_pipe` manual -> Somehow fixed...? I don't know
* [x] Ditched rangeby
* [x] Improved Map arguments
* [x] Re-added no consume env
* [x] Flag to print all realted environmnet vairables : print-env
* [x] Mapn broken -> Mostly due to `NUM_MATCH` this finds both + and - from first
    -> THis has more pros thans cons, simply notify users that how mapn works
* [x] No such macro should be early returned
* [x] Logm is bugged : Fixed
* [x] BUG : Currently trim input doesn't work for deterred macro -> Fixed
* [x] Macro attribute `!!` for discard

* [x] Add a new macro to `drop expression` 
* [x] Remove regexcache and unify to register

