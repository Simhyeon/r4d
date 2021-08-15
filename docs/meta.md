### TODOs

* [ ] Make custom parser and lexor
  * [x] print out non macro text
  * [ ] Print remainder from lines that contains macro definition
    * [ ] Complete register logic
	  * [ ] parse define macro's arguments
	* [ ] Complete invoke logic
	  * [ ] Make print line work
  * [ ] Print evaluated macro substitution
  * [ ] Print failed macro  
  * [ ] Print a line which as multiple macros in a line
  * [ ] Print a nested macro substitution

* [x] Make direct subcommand option

* [ ] New basic macros
  * [ ] Time macro
  * [ ] Web request
- This needs serious consideration because this binary targets alpine which is not necessarily easy to combine openssl library.
  * [ ] CSV macro
    * [ ] csv query
    * [ ] csv to markdown table
    * [ ] csv to wikitext table
  * [ ] Include macro
  * [ ] For loop macro
  * [ ] Text format
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
