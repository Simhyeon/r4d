# 3.1.1-rc1

- Upgraded misc crate versions
- Finally ditched lazy_static

# 3.1

* [x] Macros
    * [x] Strip macros
* [x] Macro ergonomics
    * [x] Dryrun as passthrough
    * [x] Log flag to respect pipe input + print full attributes
    * [x] Mapf trims file name
* [x] Bug fix
    * [x] Fixed trexter logger bug
    * [x] Fixed invlaid include directory
    * [x] You cannot safely use debug flag with pipe command. Added invalid error for stdin usage
    * [x] No such macro error is not properly printed
* [x] Feature
    * [x] Passthrough
    * [x] Expose update method for radstorage
    * [x] Enable literal parenthesis for body : $lp() $rp()

# 3.0.1 stable

- New macro anon for anonymous macro
- New macro scut for spaces cut
- Now comma inside parenthesis is not treated as delimiter.
- Strip has been merged with expand due to argument parsing logic change.
- Changed name splitc -> cut

# 3.0.0 ~~stable~~

- BUG : Fixed wrong initial input backtrace
- CHG : Include behaviour
- CHG : readto and readin behaviour
- ERG : Now path macro converts to platform specific path separator
- FET : New macros
    - Exist
    - Expand

# 3.0.0-rc.6

**rad**

- BUG : Stdin input panicked
- CHG : Now exit code is properly emmited on panic
- CHG : Module changes
- CHG : Now static macro is not expanded
- ERG : Ditched cmp and added multiple comparision macros
- ERG : Now documentation has 80 caps on description.
- ERG : Warn readin on relaying
- FET : Dry run
- FET : Freeze flag refactored
- FET : Packaging flag
- FET : Negate macro attribute
- FET : Pipe input macro attribute
- FET : New macros
    - dump
    - map variants : map, mapf, mapl
    - grepmap
    - meta-processing related
        - require
        - strict
        - comment
    - sep
    - after
    - until
    - capture
    - indexl
    - splitc 
    - gt, gte
    - lt, lte

**rado**

- FET : Package subcommand
- FET : Execute subcommand

# 3.0.0-rc.5

**rad**

- CHANGE : Refactored logger logics with crate trexter
- CHANGE : Solved debugging regression in terms of functionality
- CHANGE : Changed from's name to spread
- CHANGE : Deterred macros' expansion order is not consistent with function macros
- CHANGE : Removed ieval because counter replaces it
- CHANGE : Signature and color is included default into a binary feature
- ERGO : Append can get optional trailer argument
- ERGO : Append now also appends to local macro
- ERGO : Enable logm to print any local macros
- ERGO : No Breakpoint warning
- ERGO : Now foreach and forline get data as trimmed
- ERGO : Queue to be insert as no stripped.
- ERGO : Silent flag's default value is "any"
- ERGO : Trim output now consumes new line if result is empty
- FET : New macros
    - chars
    - cmp
    - comma
    - ftime
    - isempty
    - istype
    - iszero
    - loge
    - slice
    - squash
    - ssplit

- Bug fix
    - Nested literal rule was not properly stripped
    - Setting an error option resetted a logger entirely
    - File operation was able to write to self
    - Fixed consume newline was not properly respected

**rado**

- Edit in place flag

# 3.0.0-rc.4

- FET : New macros
    - Escape blanks
    - Grep && Grepl
    - strip ( differnt from previous )
    - Regexpr
    - Input
    - Temp
    - Trimla ( Trim line amount )
    - Indent ( Indent lines )
    - read\_to read\_in
    - join, joinl
    - notat
    - letr, staticr
    - counter
    - align
    - Tab && space && empty
- CHG : Macro ergonomics
    - For variatns order changed backwards
    - Static trims value
    - Halt is queued by default
    - Changed fileout's argument order
    - Renamed arr to spilt
    - Removed sep macro because
    - Removed queries macro
    - Removed strip and stripl
    - Removed cnl
- CHG : Changed argument parsing behaviour frome lexor and arg parser
- CHG : Made formatter respect processor line ending
- CHG : Now define macro also detects trim input attribute
- CHG : Rad now deletes temp file before start
- ERG : Improved descriptions a lot
- ERG : Now comment can start in between line with any type
- FET : METHOD > Set both comment and macro char at the same time
- FET : New macro attribute "="
- Bug : Assert mode panicked
- Bug : Error message cascaded as much as nested level
- Bug : Exit macro yieled error and broke from entier rad process
- Bug : Include's containder had high priority over relay target
- Bug : Fasssert treated success as fail and vice versa


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
