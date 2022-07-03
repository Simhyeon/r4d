# 3.0.0-rc.4

- CHG : Apply new(1.62) clippy fix
- CHG : Ditch unnecessary "Some" arguments
- CHG : Regex syntax change
- CHG : Changed argument parsing behaviour frome lexor and arg parser
- FET : manual flag
- FET : New macros
	- find
	- findm
	- regexpr
- FET : New macro attribute
- CHG : Now define macro also detects trim and trim input attribute
- CHG : Removed cnl macro and fixed a newline bug
- CHG : Made formatter respect processor line ending
- ERG : Improved descriptions

# 3.0.0-rc.3

### Breaking

- Changed syntax of regex macro
- Ditched many Option parameters and made it intuitive

### Else

- CHG : Changed a parsing logic little bit
- CHG : Applied new clippy fix
- FET : Manual flag
- FET : RegexCache for better regex performance
- FET : New macros
	- Find
	- FIndm

# 3.0.0-rc.2

- ERG : Many rustdoc improvement
- FET : Extension macro configuration with script.rs file
- BUG : Exit status handling
- CHG : New template macro ```audit_auth```
- CHG : Moved from ```Vec<_>``` into ```&[]```

# 3.0.0-rc.1

- ERG : All documentations for built-in macros
- BUG : Forline macro fix

# 3.0.0-rc.0

[3.0 Changes](./3_0.md)

### ETC

- New macros : import, source, cnl, listdir
- Changed "enl" behaviour

# 2.1.3

Removed features are still included as empty placeholder for compatibility
which will be removed in 3.0

- BugFix : Hid unnecessary extra features from users
- BugFix : ExtMacroBuilder's export has been feature gated by storage,
- Ergono : Ditched avoidable dependencies
	- Thiserror
	- Csv
	- Lipsum
- Ergono : Remove feature gates for better maintainability
	- Storage

# 2.1.2

- New macros
- For loop nested mechanics with $:() macro
- Changed macro concepts
