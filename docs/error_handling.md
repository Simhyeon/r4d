### Problem

This is not a public documnetation, but internal explanation purpose.

Currently of version "2.1.3" error handling is kind of inconsistent. Due to followings.

#### Problem of error modes

- Nopanic will panic in relay error or maybe some errors that I don't know of.
- Purge, Lenient, Strict, Nopanic are all in same category but not exactaly a same behaviour.
- Error handling code is quite hard to grasp and not consistent.

#### Problem of generic errors

- Currently r4d utilizes both log\_error and return error without separation.
- Sometimes log error but sometimes it doesn't.

#### Current implementaiton of error handling.

Currently r4d's course is divided into two branches.

- Runtime macros's evalution doesn't utilize '?' but branch it with match
pattern, and return EvalResult's variant.
	- EvalResult's variant properly managed purge and lenient call (And it was by design or at least I insisted it a feature)
- Function(Deterred) macro utilizes '?' and the yielded error is captured in lex\_branch\_end\_frag\_result\_err method
	- This error handled by nopanic

### Goal

- Make log\_error consistent. Always log error, user needs to know what went wrong.
	- Return error on function macros
	- Log\_error right on point when error happens other than function macros
- Make error behaviour intuitive. Difference between runtime and function macro is not to intuitive.
- Simlify error modes.

- Because processor building error can be different from other errors it should be captured.
