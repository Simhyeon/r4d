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
