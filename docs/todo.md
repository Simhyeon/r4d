# Todo immediate

- Update manuals one by one
- Add new macros that is immediately necessary for daily use
- Fix bugs that was found during manual update
    - Regardless of bug size and difficulties

- Debug all manuals
    1. Test english charcters
    2. Test korean characters
    3. Test invalid syntax, characters.

0. KEEP IN MIND : Escape rule is strange
    ```
    $assert(\,\) -> This doesn't invoked at all
    ```
    -> Check PS.r4d manual because escape character is strange in the document.

#### BUG

* [ ] Fix regex register shenenigans
* [ ] Fix deubbing feature bugs
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
* [ ] Search should be about searching. I don't know if something exists. It is no
   use when you only prints something just similar. How about showing lists if
   necessary?

#### Macro

* [ ] Enable arguments by whitespaces for `map` variants
* [ ] TOC macro-script ( Not builtin but usage's example )
* [ ] Flat -> Flatten indented sub lines into a single one
```
$stream(flat)
let setter = Setter::new()
    .builder()
    .yeah();
$consume
===
let setter = Setter::new().builder().yeah();
```
-> This is simlar to vim J function but works on chunk.

* [ ] Flatreg -> Flatten by regular expression. Regexed line becomes main line
  that following lines are joined to
```
$define(wow,a_content=$flatreg(Self::,$a_content()))
$stream(wow)
Self::PermissionDenied(txt, atype) => format!(
"Permission denied for \"{0}\". Use a flag \"-a {1:?}\" to allow this macro.",
txt, atype
),
Self::StrictPanic => 
    "Every error is panicking in strict mode".to_string(),
$consume
===
Self::PermissionDenied(txt, atype) => format!( "Permission denied for \"{0}\". Use a flag \"-a {1:?}\" to allow this macro.", txt, atype),
Self::StrictPanic => "Every error is panicking in strict mode".to_string(),
```
* [ ] Wrapl -> Wrap content by lines. == vim's "==" function
* [ ] Sortli -> Sort list
```
$stream(sortli)
ABCD
ABCEE
    AAAA
AA
$consume
===
AA
ABCD
ABCEE
    AAAA
```
* [ ] Rotate concat -> Reverse of rotate macro
* [ ] No pipe truncate option for macro users.
* [ ] Also add non evalexpr variant macro ( inc, dec )
* [ ] Pretty printer ( Json, toml etc... )

#### MISC, bug detecting, Ergonomics

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

#### Peformance

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

* [ ] Ditch wasm feature DONE
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