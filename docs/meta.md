### TODOs

* [x] Hide sandbox switch from end user experience
* [ ] Escape rule is very inconsistent
- In arg parsing it consumes and set ending parenthesis literal
- In define parsing it is treated as it is, a character ```\```

* [x] Add_line_number, clear local inside parse_lin
- Made builder pattern
* [x] Should move from returning self to returning mutable reference.
- Added backup methods for sandboxed form
  - File name backup works
  - Line number backup works
  - Local backup also works

* [ ] Utilize elog_panic to whether panic when given file doesn't exist or not

* [ ] Dogfood until 1.0

* [ ] Make proper documentation

* [x] Make library binding
  * [x] Make processor option setting more ergonomic with builder pattern
  * [x] Make code DRY -> Kinda done
    * [x] Processor -> parse method is too long
	* [x] lexor parts -> Actually this part is fine
	* [x] Arg parser is disastrous -> Little bit better
  * [x] Optional dependencies
  * [x] Only disclose necessary modules and structs

* [ ] Improve projects
  * [ ] Change module hierarchy
  * [ ] Change arguments naming
  * [ ] Make allocations less, especially string manipulation 

* [ ] New basic macros

### Deterred

* [-] Currently disabled escape with parenthesis for eaiser maintain
* [-] Failed invocation makes new line which is not so desirable

### How one should parse macro invocation?

While it is much easier to parse macro compared to other function definitions,
it is somewhat hard to do with stock libraries by itself since you have to
retain unparsed statements.

I'm thinking of multiple approaches but basic principle is to parse lines not a
whole file.

1. Get file buffer or stdin buffer
2. Iterate by lines
3. Check if line include macro invocation
4. If not, check if line include partial macro invocation
5. Push lines to `chunk` until macro invocation is complete
6. Execute macro invocation and substitue final result.

The main point is 3 and 4. How should I find whether invocation exists or not.

First approach was to utilize regex crate, while this works for partial
definition, it can't find full macro invocation because rust regex cannot find
balanced match (No subregex or recursive match). This is due to the fact that rust 
regex crate complies with pure regex standard.

My next approach will be pest. Pest has somewhat unfamailiar syntax but if used only for a line it would be fine.

### DONE

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
		- Thus "MacroMap"'s evaluation logic should be located in processor not in Macromap itself
	  * [x] Make nested invocation work
	    * [x] When definition includes nestedness -> This is evaluated on invocation
		* [x] When invocation includes nestedness -> This is evaluated with method name "evaluate"
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
- This needs sincere consideration because this binary targets alpine which is not necessarily easy to combine openssl library.
- Thus using using curl command through std::process::Command might be a global choice
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
