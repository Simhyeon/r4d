### TODOs

Currently, it looks like macro rule is working.

* [ ] Improve modularlity
Define is not basic macro but reserved macro, so...

* [ ] Enable user to override basic macro 
Simply make custom macro search comes earlier than basic macro
* [ ] Currently invalid macro definition doesn't yield proper error message
* [ ] Currently local macro is not released which is a bad idea as a final output.

* [ ] Read from file option
* [ ] Output option
* [ ] Err redirection option

* [ ] New basic macros
  * [ ] Syscmd macro
  * [ ] Time macro
  * [ ] Web request
- This needs sincere consideration because this binary targets alpine which is not necessarily easy to combine openssl library.
  * [ ] Include macro
  * [ ] For loop(repeat) macro
  * [ ] Text format
    * [ ] CSV macro
      * [ ] csv query
      * [ ] csv to markdown table
      * [ ] csv to wikitext table
  * [x] Remove extra new lines and spaces (Namely, "chomp" method)
  * [x] Random text -> Use lorem lipsum

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

* [x] Make direct subcommand option
