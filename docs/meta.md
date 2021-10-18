### Changed

### TODOs

* [x] Windows path sanity is broken shit...
* [ ] Make parser separated
* [ ] Make gui application with help of klask
- This requires nightly so different version of toolchain is used. 
* [ ] Create man page
* [ ] vimsnippet update
- List basic+keyword macros easily
* [ ] Improve projects performance
- Refactor codes into multiple chunks of functions for better readability
- Probably make a new model called debugger and segregate logic.

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
