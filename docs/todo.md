# Todo immediate

* [ ] Trim input !IMPORTANT
    * [ ] Skip expansion doesn't work for deterred macro

* [ ] Always keep in mind that slicel can fail

* [ ] Surr might be bugged? test later : Argparser is bugged
    -> This is possibly due to new argparser logic error
    when \) is supplied or namely literal quotes are bugged...
* [ ] Add environmnet variable for preventing negative index for slicel

* [ ] Log option shows unappended result before pipe append
```
 printf 'test' | rad -PL '$indent-(+)' --log
1:log
Name    = "indent"
Attr    =
Pipe input      : true
Pipe output     : false
Trim input      : false
Trim output     : false
Literal         : false
Negation        : false
Args    = "test"
---
+test%
```

* [x] Refactor slice to be much more singular
    -> Only two positive integer can be optimized, other than that everything
    has to be collected which is bit inefficient.

* [x] Branched that calls Collect directly assumed that lines were simple enurmation

-> Currently it is enumeration of tuple thus the structure should be considerd proeplery
Cehck if coherently

* [ ] Remove unnecessary head, tail, strip bullshits and integate into slice
  ,range, slicel -> Actually no, those are useful when you don't know the total
  length. Or... maybe not? -> Implement -index for range than it will be
  compltely replacable head, headl, tail, taill strip

* [ ] Add sliceu ( Slice unicode ) and make slice non allocating
* [ ] Support _ syntax for slice series
    * [x] Slicel
    * [ ] Slice

* [ ] Insulah to use : also
* [ ] Make a new macro for border?
```
% NExt is start line
// Text           // |

abcde
ddd
fff
gggg
==
// -------------------
// Text           // |
//                // |
abcde             // |
ddd               // |
fff               // |
gggg              // |
```

* [ ] Make an interal data structure for arrays? ( Table )
    -> Problem I want to split contents into an array but existing array needs
    to process the whole contents every time when it needs to index.
    ```
    $split-($comma*(),a,b,c)

    ```

* [ ] Change names that are inconsistent or incoherent
* [ ] Move regexpr to deterred macro
    -> Don't expand or parse second argument
    -> Use `split_once(',')` instead
* [ ] There are multiple macros that utilizes args to vec directory or simply
  utilize args.is-empty which might cause inconsistent behaviour. Check them.
    -> THis means such macro doesn't use "argparser" which has an side effect
* [ ] Check unnecessary `to_string`
    * [x] FunctionMap
    * [ ] DeterredMap
    * [ ] Other
* [ ] Capture to support capture group
* [ ] For chunk -> Also apply this logic to foldreg
```
Iter through lines and aggregate regexed chunk and apply macro to it
$forchunk(start_regex,end_regex,macro_body,src)
```
* [ ] Extract inner
```
let macro_name =  std::mem::take(&mut args[0]);
let mut formula = std::mem::take(&mut args[1]);
===
let macro_name =  args[0];
let mut formula = args[1];
```
* [ ] Split by
* [ ] Add a feature to use rope instead of simple string ( Crop crate )
    -> For example if skip expansion flag was given OR text size is bigger than
    1K ( which is a standard point where rope out-performs normal string )
        -> text size standard for crop usage should be supplied as arguments
        and saved to processor ` --rope 1000 ` means use crop from 1000 byte
        sizes.
* [ ] Check insula's logic throughly
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

* [x] Renamed things
    - Regex -> sub
    - Slice -> Range
    - sub   -> Slice
    - ifque => queif

* [ ] Consider implemtnting consecutive macro calls for sc and sl flags

#### BUG

* [ ] Inner panics on certain cenarios
```
$println^($inner({},1,$content~()))
```

for following

```
    pub(crate) fn map_file( args: &str, level: usize, p: &mut Processor,) -> RadResult<Option<String>> {
        if !Utils::is_granted("mapf", AuthType::FIN, p)? {
            return Ok(None);
        }
        let mut ap = ArgParser::new().no_strip();
        if let Some(args) = ap.args_with_len(args, 2) {
            ap.set_strip(true);
            let macro_src = p.parse_and_strip(&mut ap, level, "mapf", args[0].trim())?;
            let (macro_name, macro_arguments) = Utils::get_name_n_arguments(&macro_src, true)?;
            let file = BufReader::new(std::fs::File::open(p.parse_and_strip(
                &mut ap,
                level,
                "mapf",
                args[1].trim(),
            )?)?)
            .lines();

            let mut acc = String::new();
            for line in file {
                let line = line?;
                acc.push_str(
                    &p.execute_macro(
                        level,
                        "mapf",
                        macro_name,
                        &(macro_arguments.clone() + &line),
                    )?
                    .unwrap_or_default(),
                );
            }
            Ok(Some(acc))
        } else {
            Err(RadError::InvalidArgument(
                "mapf requires two arguments".to_owned(),
            ))
        }
    }
```

* [x] Bug: insulav and insulah was not stripping
* [x] Fixed a bug whree skip_expansion was not working
* [x] Syscmd is inconsistent
    * [x] quote enclosed value has to be sent separately. Currently arguments
    * [x] Rad inside rad squeezes output -> It was other bug


* [ ] Trim input is really... necessary. I mean it is required to do lots of
  things... but hey it is 4.0 and you really fucking needs it.
```
$stream(insulav)
\*
$assert(true,$istype( uint , 0 ))
$assert(false,$istype( uint , -1 ))
$assert(true,$istype( int , 0 ))
*\
$consume|()

$define=(sq,
a_ln a_lc
=
$ifelse($eval($a_ln() %  7 == 5),
$rotatel($comma*(),c,$a_lc()),
$a_lc()))
$forline-($sq($a_LN(),$:()))
=> This yield duplicated new lines
```
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

* [ ] Illustrate that insula macros are not pretty printer but, rather
  functional macro that creates sufficient spaces and newlines for following
  macro processes
* [ ] Changed rotate behaviour
    -> Think of center rotation as "ferris wheel"
* [ ] Update manuals
    * 24/177
* [ ] Update repository documentations
    * [ ] Macro indices
    * [ ] Macro syntaxes
    * [ ] Hook macro
    * [ ] About extension and abscene of include macro
    * [ ] ETC...
    * [ ] Stream series and pipe rules

#### Features

* [ ] Check if greedy argument's no-strip behaviour is ideal or not
* [x] Currently --sc arguments are sent as pipe which is not evaluated at all
  which might be unexpected behaviour
    -> Now sc and sl accepts arguments as real arguments not pipe input
* [x] Ditch evalexpr flag and include it as default
* [x] Search should be about searching. I don't know if something exists. It is no
   use when you only prints something just similar. How about showing lists if
   necessary?

#### Macro ( macro )

* [x] Add slicel
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
* [ ] Inner align? -> Stretch
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

* [x] use `get_split_arguments_or_error` for arg splitting
    * [x] FunctionMacroMap
    * [x] DeterredMacroMap
* [x] Ditch trim! macro because it is literally unnecessary
* [x] Implement inner macro
* [x] Update `args_with_len`
* [x] Argparser rerturn cow vector rather than string vector

* [ ] Macro to return cow rather than string? Is it that performant?
* [ ] Try removing unnecessary clone calls
* [-] Mie and pie insert_str is inefficient. -> Not so necessarily
    -> Push_str is also O(n) Sadly
* [ ] Check for alignby performance maybe duplicate
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
    * [ ] Early return
* [ ] Documentation
    * [ ] About escape rules + parenthesis rule
* [ ] Test
    * [ ] Test windows build
$todo_end()

### LATER

* [ ] Test hook macro documentaion

---------- ---------- ---------- ---------- ---------- ---------- ---------- 
### How I should handle todos

There are so many todos and I'm getting lost among them.

It's time to rearrange the things.

There are multiple types of todos classified as sorts

- Bug fix
- Documentations
- New macro, features
- Peformance improvement

There are todos that is classified as range

- Affect parsing, expansion
- Affect several macros
- Affect an macro
- Affect nothing

There are todos that is classified as difficulties

- Hard
- Easy

Bug fix is primary yet todo that needs parsing, expansion logic to be fixed are
hard and time consuming. And I cannot spend that much time yet or at least not
often.

### General rule of thumb

1. Update manuals first because I can both check bugs in macros itself and test
   processing logic's bug
2. Some immediate, critical bugs should be fixed asap
3. Bugs detected while writing manuals should be addressed first
4. Generic outage, bugs should be fixed later.
5. Manuals should be tested after generic bug fixes
6. New macros regardless of daily use should be implemented late
7. Peformance update should come as final

* Bug or unintended yet consistent behaviour should not be fixed at the time,
  but addressed later with much care.
    -> However this should be noted for later fix
