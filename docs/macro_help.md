MACRO SYNTAX:

	Macro syntax is dollar sign($). To invoke a macro put a dollar sign before
	a desired macro name.

	```
	$macro_name()
	```

	To define a macro, use a define macro. Argument is separated by
	whitespaces. Macro body should be separated with equal sign(=). Macro name
	or argument name should start as underscore or non numeric
	characters(UTF-8). 

	```
	$define(macro_name,arg1 arg2=Plain text $arg1() $arg2() after text)
	% macro name is macro name
	% arguments are arg1 and arg2 which can be invoked as 
	% $arg1() and $arg2() inside of a body
	```

	Comment syntax is percentage sign(%). Comment is disabled by default and
	should be enabled with --comment flag.

	```
	% I'm comment, default comment should start 
	% at the first chraacter of the line
	```

MACRO USAGE:

	This is an abbreivated version of usage. 

	Refer https://github.com/Simhyeon/r4d/blob/master/docs/macro_indices.md 
	for full usage.

	### define
	
	$define(name,a1 a2="$a1(),$a2()")
	
	### declare
	
	$declare(n1 n2 n3)
	
	### undef
	
	$undef(name)
	
	### rename

	$rename(len,length)
	
	### repl
	
	$repl(before,AFTER)
	
	### append

	$append(test, CASE)
	
	### pause (keyword macro)

	$pause(true)
	$pause(false)
	
	### include
	
	$include(src/content.rs)

	### read

	$read($a_src())
	
	### tempin, tempout, tempto
	
	$tempout(Hello world)
	$tempin()
	$tempto(out.json)
	
	### redir

	$redir(true)
	$redir(false)
	
	### fileout
	
	$fileout(true,file_name.txt,Hello World)
	$fileout(false,file_name.txt,This is appended)
	
	### env
	
	$env(HOME)
	
	### envset
	
	$envset(CUSTOM_VALUE,I'm new)
	
	### ifenv (keyword macro)
	
	$ifenv(HOME,$env(HOME)) 
	
	### ifenvel (keyword macro)
	
	$ifenvel(HOME,$env(HOME),No home is defined) 
	
	### path

	$path($env(HOME),document)
	
	### abs

	$abs(../../some_file.txt)
	
	### name

	$name(/home/test/Documents/info.txt)
	
	### parent

	$parent(/home/test/Documents/info.txt)
	
	### let (keyword macro)

	$define(test,a\_src a\_content=
		$let+(source,$path(cache,$a\_src()))
		$fileout(false,$source(),$a\_content())
	)
	
	### static (keyword macro)
	
	$static(test=$time())
	
	### pipe

	$pipe(Value)
	$-()
	$*() % Literal piped value
	
	### Repeat
	
	$repeat(3,Content to be repeated
	)
	
	### array

	$syscmd|^(ls)
	$arr($-(),$nl())
	$arr($-(),$nl(),\.sh$) // File that ends with .sh
	
	### foreach (keyword macro)

	$foreach(\*a,b,c*\,Value: $:
	)
	
	### forloop (keyword macro)

	$forloop(3,5,Number: $:
	)

	### eval

	$eval(1+2)
	$eval(0.1+0.2)
	
	### if (keyword macro)
	
	$if(true,TRUE)
	$if(false,False)
	
	### ifelse (keyword macro)
	
	$ifelse( true ,I'm true,I'm false)
	$ifelse( false ,I'm true,I'm false)
	
	### ifdef (keyword macro)
	
	$define(some=value)
	$ifdef(some,Defined)
	
	### not
	
	$not(true)
	$not(false)
	
	### syscmd
	
	$syscmd(uname -a) 
	$syscmd(ver)
	
	### sub

	$sub(1,5,123456789)
	
	### tr

	$tr(Given String,iSg,aOs)
	
	### len
	
	$len(Lorem ipsum dolor)
	
	### regex

	$regex(Hello world,w.*?d,rust)
	
	### trim, chomp, comp, triml

	$trim($value())
	$chomp($value())
	$comp($value())
	$triml($value())

	### wrap()

	$wrap(20,$value())
	
	### nl
	
	$nl()
	
	### lipsum
	
	$lipsum(5)
	
	### time, date

	$time()
	$date()
	
	### from
	
	$define(three,a1 a2 a3=1-$a1(), 2-$a2(), 3-$a3())
	$from+(three,
	a,b,c
	d,e,f
	)
	
	### table
	
	$table(github,\*a,b,c
	1,2,3
	4,5,6*\)
	$table(wikitext,\*a,b,c
	1,2,3
	4,5,6*\)
	$table(html,\*a,b,c
	1,2,3
	4,5,6*\)

	### flowcontrol

	$escape()

	$exit()

	### panic

	$panic()
