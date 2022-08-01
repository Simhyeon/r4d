### TODO
$todo_start()
* [x] New macros
* [x] Macro ergonomics
* [x] Library
* [ ] Bug fix
    * [ ] Currently use configured macro is not "available" in log message.
* [ ] Feature
    * [ ] Include data format pretty printer
* [x] Documentation
* [ ] Test
    * [ ] Test hook macro documentaion
    * [ ] Test windows build
$todo_end()

### Changes

- New macros
    * [x] Exist
- Changed Include behaviour
- Changed readto and readin behaviour

### Macro ergonomics

- Macro chain : This is much harder because many internal changes

#### Hard+misc ones

* [ ] Complete wasm

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

### DONE

* [x] BUG! : Fix unicode tail,head,strip: God damn it
* [x] New macro : $enl() remove right next newline
* [x] Error on duplicate table name
* [x] Add droptable macro
* [x] Redirect refactor with relay and hold macro

* [x] Possibly some mathematics related macros
    * [x] Min
    * [x] Max
    * [x] Ceil
    * [x] Floor
    * [x] Precision ( Floating number )
* [x] Some more text processing
    * [x] Cap -> captialize
    * [x] Low -> lower
    * [x] Num -> strip number part from given text
    * [x] Rev -> Reverse an array
    * [x] Eval with original formula support
    * [x] tarray & hms


* [x] New basic macros
    - head
    - headl
    - tail
    - taill
    - strip
    - stripl
    - Grep  ( Only print matched lines )
    - Index ( Get indexed value from array )
    - Sort  ( Sort array by values )
    - Sortl ( Sort lines by values )
    - Fold  ( Fold new lines to single line )
    - Foldl
    - Count ( Get count of array )
    - Countw( Get count of array )
    - Countl( Get count of lines )

* [x] Storage function
    * [x] update
    * [x] extract
* [x] New default macros
    * [x] Sep macro (keyword)
* [x] Indexing macros with "cindex" crate
* [x] Named pipe ( in a hashmap )
* [x] Pipe truncate option as builder pattern
- Default as none for compatibility
- From 2.0, default is true
* [x] Fix comment argument bug
* [x] DNL Macro
* [x] New basic macro
    * [x] Triml -> Trim line by line
    * [x] Wrap : But with additonal packages
    - Though I beleve it worth
    * [x] Panic macro
    * [x] Flow control macros
        * [x] Escape
        * [x] Exit

* [x] Improve signature ergonomics

* [x] Fixed unexported DiffOption and HookType re-export
* [-] Added help message before and after original help message

* [x] Make signature method
* [x] Use clap arg builder

* [x] Changed builder pattern to use move semantics rather than mutable reference
* [x] Debug sandboxing was not working

* [x] Bug : Name collision
- This was solved by clearing lower level local macros
- This was caused because nested macro was using previously declared local macro
- In a scenario apple
1 declared local macro "alpha" and nested macro 2 delcared a macro called "bravo"
This is totally fine and local macros will be all perished after 0th invocation.
- In a scenario banana
1 declared local macro "alpha" and nested macro 2 delcared a macro called "alpha"
This is also totally fine because lower level local macros are called first.
- However, in a scenario cacao
1 declared local macro alpha and nested macro 2 declared a macro called alpha.
But in this time 1 called another macro 2-1 which declard a macro called "alpha".
2-1's local macro "alpha" is treated as 2's local macro alpha and this triggered some nasty bug.

- The solution was quite straightforward. Remove all lower level macros after
every custom macro invocation. This might not be the most performant solution
but very cheap one.

* [x] Not necessarily a bug but improved
In debugging, argument was not processed in debug console. Which is somewhat
useful and not so useful. I simply added debugging feature's varaible and added
processed argument. Users can view both raw and processed arguments at the same time.

* [x] Debugging : Removed output text from promt, because if felt unnecessary
* [x] Macro attribute position fix

* [-] Eable useful basic macro customization by exposing processor state to end
user.
- You know what? This doens't look useful. Most of them are quite vulnerable to
undesired changes and not so desirable to end user usage.

* [x] Deprecating global in favor keyword macro ```static```
* [x] Deprecating bind in favor keyword macro ```let```
* [x] Declare macros
* [x] Custom special character
* [x] Comment type addition
* [x] Fixed a bug where assert result was not printed when there was no error at all.

* [x] Hook macro for macro and character
* [x] Hook macro should be temporarily disabled for specific cases
* [x] Static custom rules

* [x] Diff only changed option
* [x] Bug: Fixed default comment behaviour
* [x] Bug: Non sufficient macro invocation doesn't return whole string... wut..


* [x] Comment rule
* [x] New basic macros
    * [x] Ifenvel
    * [x] ifdefel
    * [x] abs
    * [x] arr
    * [x] envset
* [x] Newline consistency in chomp macro
* [x] Fixed a bug where unterminated macro was not properly evaluated

* [x] Merged into a single macro path becuase paths is redundant
* [x] Assert, nassert, fassert
* [x] Repl as keyword macro because replacement should not be expanded
* [x] Made error branch for invalid argument
* [x] Create unit test

* [x] Windows path sanity is broken shit...
* [x] Yield diff as error
* [x] Log as error
* [x] Segreated debugger module sueccesfully
* [x] Added a custom prompt log logic
* [x] Added a custom prompt log for from macro

* [x] Fixed keyword macros being written without chunk respect
* [x] Print macro attributes in logging
* [x] Refactor codes so that keyword and macro evaluation is merged.
* [x] Empty name is problematic or say inconsistent
* [x] Does removing piped value necessary? It doesn't give that much benefits, I think...
* [x] Change "from" syntax, a breaking change though, better choice for ergonomics.
* [x] Global bind which evaluates to value
* [x] Changed paths syntax
* [x] Overriding is prohibited in strict mode
* [x] Make strict by default
- Make lenient option rather
- Also apply to documents -> Not yet
* [x] Basic macro changes
    * [x] Make ifdef work as if and if else
    * [x] Make ifenv macro also as if and ifelse
* [x] New replace macro
* [x] Changed several cli options

* [x] Add comments for add custom rules
* [x] Make include respects currently set file
- For example, relative file is evaluated based upon currently read file's path
* [x] Made include input file is properly set, was actually a bug
* [x] No env value is currently panicking... which is not so desirable, I guess
* [x] Env values such as INPUT\_FILE or INUPT\_DIR would be usefule
- It has been decided as RAD_FILE and RAD_FILE_DIR
* [x] New basic macros
- Get parent
- Get filename
- Merge paths
* [x] Is warning for env is also harsh? Should be empty?
- Now this only yield on strict mode
* [x] Remove unnecessary hashamp clone
* [x] Make some macros a speical type
    * [x] Make if,ifelse,foreach,foloop as a keyword macro such as pause and define
    * [x] Create new struct keyword maps
* [x] Refactor sandbox environment
* [x] Add not macro

* [x] Make error messages much more accurate for various situations.
* [x] Add a diff tool for debugging
    * [x] Set diff original and diff processed when necessary
    * [x] Print diff in result if certain option was given => Diff option
* [x] Add conflicts with option for more ergonomic options

* [x] Segrate some structs into separate files
* [x] Add new error type permission denied as panicking
* [x] WriteOption::Stdout is weird in err context because it is always stderr
- Changed to WriteOption::Terminal
* [x] Panicking error red colored
* [x] Make basic macro failable not a panicking
- Make it an option
* [x] Add new basic macro with closure without knowing macrotype
* [x] Print permission
* [x] Divided io into fin and fout
* [x] Fixed a bug wherer dollar sign doesn't restart framgent check
* [-] Currently disabled escape with parenthesis for eaiser maintain
* [-] Escape rule is very inconsistent
- In arg parsing it consumes and set ending parenthesis literal
- In define parsing it is treated as it is, a character ```\```
- It costs more than its worth

* [x] Make colored optional
* [x] ~~Dry run~~ Discard run
- Problem is dry run doesn't prevent fileio fileout or redir
* [x] Authority option
- Disable IO,Syscmd,Shell by default
- And enable it by user input
- Create : AuthFlagStruct with stores array of u8 which represents u8 number of enum values.

* [x] BUG :::: Sys cmd fails to parse arguments... shit
- Simply bypassed with default greedy and space split
* [-] BUG :::: On debug mode, include triggers newly nested prompt...
- Well this might not be a bad idea to be frankly speaking
* [x] Make proper documentation for codes(docs.rs)
* [x] Consider adding custom basic macros
- This is very much easy so why not
* [x] New debug mode documentation
* [-] Check if parse_chunk_lines acts as intended in basic macros
* [x] Change logic of infinite loop warning, so that it onlys warns on body
expansion

* [-] Check if lexresult::Discard is ane behaviour
- This is ok because discard is only called when escape_nl has been set which
means it succeeded evaluation
* [x] Print line doesn't work as intended
* [x] Refactor from_yatti_yatta series into single method
* [x] Complete span command
- Remove unused caches
* [x] Make chunk_line chunk_char for more detailed debuggin and error logging
- Also parse_chunk calls freeze_number which makes error debugging hard to watch
- This was actually not so hard, surprisingly
* [x] Make interactive flag which toggles text wrapping
* [x] Make BR as not an error but a warning
* [x] Non printing macro do print empty space in debug mode
- $BR() evaluates to newline character fixed it
* [x] Log flag with debug flag yield strange output
- Line after macro evaluation is duplicated

* [x] Add debugger
  * [x] Lines option
  * [x] log option
  - Log every evaluation
  * [x] Debug command
    * [x] print, help
    * [x] step
  * [x] Breakpoint

* [x] If, ifelse is not so friendly with deterred macro
- Just remove it
* [x] Argument parsing is not properly done
- This was because I was adding new argument if there was any "blank character"
which menat consequent blank characters created more arguments
* [x] Warning system is broken in define parsing

* [x] Currently ifelse is inefficient
- Added deterred rule
* [x] Make library binding
  * [x] Make processor option setting more ergonomic with builder pattern
  * [x] Make code DRY -> Kinda done
    * [x] Processor -> parse method is too long
    * [x] lexor parts -> Actually this part is fine
    * [x] Arg parser is disastrous -> Little bit better
  * [x] Optional dependencies
  * [x] Only disclose necessary modules and structs

* [x] Add_line_number, clear local inside parse_lin
- Made builder pattern
* [x] Should move from returning self to returning mutable reference.
- Added backup methods for sandboxed form
  - File name backup works
  - Line number backup works
  - Local backup also works

* [x] Read from string method for lib usage
* [x] Hide sandbox switch from end user experience
* [x] Combination of stdin and file input
* [x] Add strict mode which is mutualy exclusive to purge mode
* [x] Temp refacotr a bit
- Made temp a file not a path
- Made temp redirect
* [x] Final render result -> Show warning and erros count
* [x] NameToArgs is not desirable
* [x] Make declartion more tolerable
* [x] Add local(bind) macro
* [x] Made error messages much easier to use.
* [x] Warn non existent include operation with error message
* [x] Improve foreach, forloop ergonomics
* [x] Make temp target change macro
* [x] Greedy is working not as it should
* [x] ENV macro
* [x] Path macro
* [x] Add literal attribute
* [x] Parent argument indicator

* [x] Enable all greedy by default option
* [x] Purge mode
* [x] Frozen file -> Bincode file
* [x] Greedy suffix which put all remainder to last element without splitting
- e.g. $pipe|+(1,2,3)

* [x] Unbalanced parenthesis warning in non fragment case.

* [x] Make expansion rule more consistent for basic macros
* [x] Literal input is stupid make, make comment-like literal rule instead
* [x] Improve macro ergonomics
* [-] Detect failed local macro definition
- This is impossible because definition can contains outer macro
* [x] Change iterator value naming -> $:
* [x] Name can be empty space which is problem
* [x] Refactor namespace rule
* [-] --Greedy double quote rules-- This is impossible or hard to make
* [x] Make error consistent so that failed invoke calways yield error message
* [x] Escape char doesn't work at all.. All escape characters all just printed as it is.
* [x] Line ending consistency
- Give user an option to use preferred line ending
* [-] Make syscmd call unsafe and allow only when sudo was given
- Maybe this is fine? Because some command should require auth anyway?
* [x] Make undef line is deleted
* [x] Error loggin
  * [x] Error message should indicate which line caused an error
  * [x] Silent option
  * [x] To stderr
  * [x] To file
* [-] Failed macro stops further evlaution.
- This is because parse chunk only executed only when macro is valid
* [-] Improve modularlity
- Define is not basic macro but reserved macro for now, Change this into basic
macro for better readability and maintainability.
Though it has some benefits, I don't think it is necessary to refactor.
* [-] Is nested parse chunk necessary? Maybe main parser is alreayd expanding all?
It was not... mostly because some macro can make another macro call...

* [x] Change from chunk write to buf writer
Currently every substituted text is saved to a single string variable, which is
problematic when ram size is not sufficient. Use buff writer to write given
string without saving to designated space.

* [x] God damn it, lines() consume all duplicate new lines
I googled a lot and shamlessly copied from stackoverflow, I Felt real needs to
learn rust much more.

* [x] Made defnition line is deleted

* [x] Enable user to override basic macro
* [x] Currently local macro is not released which is a bad idea as a final output.
* [x] Currently local macro is not perperly constructed when same macro invoked in single call
Using usize type level is not a bad idea, however it should be definite where
to add number and not

* [x] Read from file option
* [x] Output option

* [x] Make custom parser and lexor
  * [x] print out non macro text
  * [ ] Print remainder from lines that contains macro definition
    * [x] Complete register logic
      * [x] parse define macro's arguments
    * [x] Complete invoke logic
      * [x] Basic macro works for now
      * [x] Single macro in single line works
      * [x] Single macro in multiple lines
      * [x] Multiple macro in multiple lines
      * [x] Multiple macro in multiple fragmented lines
      * [x] Make custom macro works
        - Invocation should be also another
        - Thus "MacroMap"'s evaluation logic should be located in processor not
        in Macromap itself
      * [x] Make nested invocation work
        * [x] When definition includes nestedness -> This is evaluated on
        invocation
        * [x] When invocation includes nestedness -> This is evaluated with
        method name "evaluate"
  * [x] Print evaluated macro substitution
  * [x] Print failed macro
  * [x] Print a line which as multiple macros in a line
  * [x] Print a nested macro substitution

* [x] New basic macros
  * [x] Trime, chomp, compress no stripping
  * [x] Substring
  * [x] tr
  * [x] Pause macro option
  * [x] Write to temp file /tmp %TEMP%
  * [x] Literal attribute -> $test\(Literal text\)
  * [x] Make pipe rule
  * [x] Len
  * [x] Rename macro
  * [x] Csv table html format
  * [x] Define append (appdef)
  * [x] Text format
    * [x] CSV macro
    * [x] Data macro from data
      * [x] csv to markdown table
      * [x] csv to wikitext table
  * [x] Syscmd macro
  * [x] Time macro
  * [x] Undefine macro
- This needs sincere consideration because this binary targets alpine which is
not necessarily easy to combine openssl library.
- Thus using using curl command through std::process::Command might be a global
choice
  * [x] Include macro
  * [x] Repeat macro -> Same thing
  * [x] Foreach loop
  * [x] For loop -> Change by number
  * [x] If macro
  * [x] If define macro
  * [x] Remove extra new lines and spaces (Namely, "chomp" method)
  * [x] Random text -> Use lorem lipsum

* [x] Make direct subcommand option

- Buggy fixes, reserved for release notes

* [x] Fixed strange literal syntax
* [x] Removed double quotes syntax
* [x] Removed literal version of macro invocation
* [x] args_to_vec cannot parse string ```\**\``` starngely
* [x] Substring doesn't work with utf8
* [x] Foreach is broken
* [x] Changed define syntax

* [x] Decide concrete rule for versioning
* [x] Possibly relay and halt warning
* [x] Enable lib user to change write option on the way
* [x] Queue
* [x] Separte warning into two types
    * [x] System + program warning ( About security )
    * [x] Processing warning ( About execution sanity )
* [x] Merge fragmented option into single enum

From 2.0
* [x] Renamed basic to function
* [x] Renamed keyword to deterred
* [x] Created disgnated runtime macro map
* [x] Greedy as default behaviour and cannot be disabled becausew hy not
* [x] Removed closure rule
* [x] Pipe truncate as default
* [x] Made distinction between function macro and deterred macro much more consistent
* [x] Procedural macro for extension macro
* [x] Move deterred macros into function macros if possible
* [x] Hygienic processing
* [x] Relay halt as stack oriented not variable oriented
* [x] Rule files will get vector, not an option of vector
* [x] Provide auth checking in ext interface

---
For 2.1.2
* [x] Missig error message
* [x] Fixed wrong abs behaviour
* [x] Document macro
* [x] Document for macro builder
* [x] Warn user about unterminated input
* [x] Disabled read macro
* [x] Input stack bug
* [x] Include as raw option
* [x] Exec command
* [x] inplace eval
* [x] For loop nested mechanics with $:() macro
    * [x] This is breaking changes... Thus should be configured as feature until 3.0 release
* [x] Relocated function macros to deterred macro

---
For 2.1.3
* [x] Hid unnecessary extra features from users
* [x] ExtMacroBuilder's export has been feature gated by storage. What?
    - Now it's independently exported.
* [x] Ditch avoidable dependencies
    * [x] Thiserror
    * [x] Csv
    * [x] Lipsum
* [x] Remove features for better maintainability
    * [x] Storage

---
After 2.1.4

* [x] Rado
    * [x] Clap template
    * [x] Clap options
        * [x] Diff subcommand
        * [x] Edit subcommand
            * [x] Basics
            * [x] Make default rad_editor variant.
        * [x] Read subcommand rad,execute option
        * [x] Sync subcommand
            - Possibly rename later
        * [x] force subcommand
            * [x] read flag
        * [x] -o out option
        * [x] arguments option

---
3.0.0-rc.2

* [x] Hide processor and enhance Processor's documentation
* [x] Exit status handling : It exited the whole process without going further
sources
* [x] Audit auth template
* [x] Easily extend with script.rs file
* [x] Change from Vec<> into &[] if possible
* [x] Docs.rs documentation

---
3.0.0-rc.4

* [ ] New macros
    * [x] Escape blanks macro
    * [x] Grep and grepl variants separation
    * [x] Renamed arr to spilt
    * [x] Removed sep macro because, join works the same
    * [x] Removed queries macro
    * [x] Regexpr
    * [x] Unwrap
    * [x] Find
    * [x] Findm
    * [x] Input
    * [x] Temp
    * [x] Trimla ( Trim line amount )
    * [x] Indent ( Indent lines )
    * [x] Tab && space && empty
    * [x] read\_to read\_in
    * [x] join, joinl
    * [x] Number notation
    * [x] letr, staticr
    * [x] Counter macro
    * [x] Removed strip and stripl
    * [x] Align texts
* [ ] Macro ergonomics
    * [x] Renamed unwrap to strip
    * [x] Regex order change
    * [x] Changed tr order
    * [x] Newline can be repeated
    * [x] Changed fileout's argument order
    * [-] Tempout truncate option : No, but temp_to and new processor will
    delete temp file before writing contents
    * [x] Halt with boolean arguments so that, halt is queued by default
    * [x] For variant order change

* [x] Apply new(1.62) clippy fix
* [x] Ditch unnecessary "Some" arguments
* [x] Changed argument parsing behaviour frome lexor and arg parser
    * [x] Regex pattern doesn't go well with string literal "\* *\" syntax
    * [x] Should be represented as literal
* [x] Make a manual option (With signature option)
* [x] New macro attribute trim_input '='
* [x] Define macro respects trim_input
* [x] Removed cnl
* [x] Regex cache
* [x] Find possible inconsistent \n chracter usage
* [x] Trim performance with macro_rules
* [x] Changed parse_chunk_args logic and ditched parse_chunk_body
* [x] Queue is inconsistent (Queue execution timing was strange)
* [x] On parse chunk body: Unterminated string was not appended to remainder
* [x] Now comment can start in between with start type

* [x] Bug : Some macro didn't processed literal properly
* [x] Bug : Assert mode panicks on first error
* [x] Bug : Erro rmessaged cascaded as much as nested level
* [x] Bug : Exit yieled error and printed unreasonable erros when including multiple files
* [x] Modifed lex_branch_end_frag_eval_result_error to not print error on itself
* [x] Bug: Include containder had high priority over relay target
* [x] Bug: Fasssert set success as fail

* [x] Line number is strange on include. Although this persists
* [x] Enl inside macro body doesn't work... why?. Now it works

* [-] Logger doesn't work in nested context
    - I don't know but... it works now? how come...
* [-] Unbalanced parenthesis warning is repeated for no reason. This was
  because I was using it as content.

* [x] Now foreach and forline should get data as trimmed?
* [x] Trim output now consume new line if result is empty
* [x] Included signature and color as default into a binary because man flag is
critical
* [x] Now, silent's default value is any
* [x] New macros
    * [x] slice
    * [x] loge
    * [x] cmp
    * [x] ssplit
    * [x] istype
    * [x] iszero
    * [x] isempty
    * [x] ftime
    * [x] comma
    * [x] append with trailer
    * [x] chars
    * [x] squash
* [x] Feature
    * [x] Make deterred macro works like other macros
    * [x] Rado : Edit in place with io operation
* [x] Improve macro ergonomics
    * [x] Enable logm to print any local macros
    * [x] Append now also appends to local macro
    * [x] APpend is now a deterred macro
    * [x] No Breakpoint warning
    * [x] Changed from to spread
    * [x] Removed ieval because counter replaces it
    * [x] I changed queue to insert as no stripped.
* [x] Bug fix
    * [x] Literal rule is bugged ( Nested literal rule doesn't work at all )
    * [x] Setting an error option resetted a logger entirely.. why I did that?
    * [x] File operation was able to write to self
    * [x] Fixed consume newline waas not properly respected

* [x] BUG : Stdin yields error because input was not set
* [x] ERGO : Cmp is ambiguous, Change it to better name with useful variants
* [x] FET : Freeze refactor
    - Disable non-declare macros in freezing mode. ( Define or static )
    - Add new "set\_freeze\_mode" method
    - Freeze reuses out option
* [x] FET : Packaging
* [x] Documentation : 80 char cap for description + if variant refactor
* [x] Changed module name into common, relocated many common variants into
  separate ones
* [x] Now static macro is not expanded
* [x] Feature : Dry run

* [x] Pipe input macro attribute
* [x] Warn readin when relay is on. For sanity reason
* [x] Now relay and read to doesn't truncate a file
* [x] Now raw include doesn't pause but escape : More efficient + can handle
  possible breaking rad codes
* [x] Somehow container relaying priority was wrong ..?
* [x] Include call inside macro calls are not espcted behaviour
* [x] now index variants don't allocate into a separate vector but use iterator
  directly in cost of O(n) time complexity

* [x] New macro
    * [x] Dump
    * [x] indexl : Index line
    * [x] Sep : Separate
    * [x] until
    * [x] after
    * [x] map variants
    * [x] grepmap
    * [x] Capture
    * [x] Meta-processing related
        * [x] Require
        * [x] Strict
        * [x] Comment
    * [x] splitc : Maybe this can replace until or after?
        - No actually not, because splitc's user should know the exact form
        while until and after never fails thus enable dynamic input.
* [x] Negate macro attribute

