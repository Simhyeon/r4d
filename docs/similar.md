# Add similarity algorithm for...

1. Invalid macro execution -> SUggest most similar macro name
2. Invalid macro manual -> Also SUggest most similar macro name

# TODO

* [ ] Learn about similar string algorithm
* [ ] Add a search algorithm for manual command

```
~Reference~
https://yassineelkhal.medium.com/the-complete-guide-to-string-similarity-algorithms-1290ad07c6b7
```

# Example

```shell
rad --man evaluation
--> There is no macro named as "evaluation". Did you mean "eval"?
```
This would be really helpful.

Or even better, create a new command search
```shell
rad --search evaluation
---
Macro name: eval
Short desc: blah blah blah

Macro name: evalk
Short desc: blah blah blah
---
```
